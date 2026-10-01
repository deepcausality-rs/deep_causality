/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fail-safe controller, as six stages of one causal process: read the telemetry and the
//! sensor frame, fuse the frame and the drone's own state into the ground context, judge each patch
//! from the context, detect the faults with a collection of causaloids, decide the fail-safe on the
//! standard ladder, and turn it into a maneuver with the fail-safe machine, which flies only what
//! the Effect Ethos approves. The checks the Ethos's norms run are here too.

use crate::constants::*;
use crate::model_config::{Detector, landing_state, look_state};
use crate::model_types::{
    Candidate, FailsafeMachine, FailsafeProcess, FailsafeState, Faults, Frame, Fusion, Ground,
    GroundContext, GroundEthos, GroundNode, LandingTarget, Maneuver, Plan, PlanStatus, Proposal,
    Review, Ruling, Situation,
};
use deep_causality::{
    ActionParameterValue, AggregateLogic, CausalEffect, CausalityError, EffectLog, LogAddEntry,
    LogAppend, MonadicCausable, MonadicCausableCollection, PropagatingEffect, ProposedAction,
};
use deep_causality_algebra::Real;
use deep_causality_context::{
    ContextIndexError, Contextoid, ContextoidId, ContextoidType, ContextuableGraph, Datable,
    DiscreteTime, EuclideanSpace, RelationKind, Temporal, TimeScale, UncertainData,
};
use deep_causality_ethos::{DeonticInferable, TeloidID, TeloidModal};
use deep_causality_num::{lift_i64, lift_usize, lower};
use deep_causality_uncertain::{SampleSession, Uncertain, UncertainBool, UncertainError};
use dynamic_drone_failsafe::{Command, FLIGHT_START_HOUR, FloatType, Quantity, Telemetry};
use std::collections::HashMap;

/// Stage 1: takes this second's telemetry and sensor frame into the process and records the
/// telemetry in the log.
pub fn sense(
    frame: Frame,
    state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<Frame> {
    let t = frame.telemetry;
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "t={} s: Telemetry: {:.0} satellites, HDOP {:.1}, {:.0} % packet loss, lowest cell {:.2} V, {:.0} m above ground.",
        t.time_s(),
        lower(t.satellites()),
        lower(t.hdop()),
        lower(t.link_loss_pct()),
        lower(t.min_cell_v()),
        lower(t.altitude_agl_m()),
    ));
    FailsafeProcess::new(Ok(CausalEffect::value(frame)), state, ctx, logs)
}

/// Stage 2: records where the drone is, sets the context's clock, fuses the frame into the context
/// and writes the drone's own state into it. Each patch's five quantities are
/// fused over every frame that has seen the patch, each reading weighted by its inverse variance,
/// and the context node of each quantity holds the fused value as a normal distribution.
pub fn perceive(
    value: CausalEffect<Frame>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<Frame> {
    let Some(frame) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let Some(mut context) = ctx else {
        return failed(CausalityError::MissingContext(), state, None);
    };
    state.position = frame.position;
    match fuse(&mut context, &mut state, &frame)
        .and_then(|()| record_drone(&mut context, &mut state, &frame))
    {
        Ok(()) => FailsafeProcess::new(
            Ok(CausalEffect::value(frame)),
            state,
            Some(context),
            EffectLog::new(),
        ),
        Err(err) => failed(graph_error(err), state, Some(context)),
    }
}

/// Stage 3: judges every patch in the frame from what the context holds about it. The clock node
/// says whether it is night, which decides what a thermal contrast means: water is warmer than the
/// land at night and cooler by day. Writes the land temperature into the context and records how
/// much of the ground below is safe.
pub fn judge(
    value: CausalEffect<Frame>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<Telemetry> {
    let Some(frame) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let Some(mut context) = ctx else {
        return failed(CausalityError::MissingContext(), state, None);
    };
    let night = match is_night(&context) {
        Ok(night) => night,
        Err(err) => return failed(err, state, Some(context)),
    };
    if let Some(land_c) = land_temperature_c(&state, &frame) {
        state.land_temperature_c = Some(land_c);
    }
    let Some(land_c) = state.land_temperature_c else {
        return failed(CausalityError::ValueNotAvailable(), state, Some(context));
    };
    if let Err(err) = set_point(&mut context, LAND_TEMPERATURE_ID, land_c) {
        return failed(graph_error(err), state, Some(context));
    }
    for reading in &frame.readings {
        match judge_patch(&context, reading.patch(), land_c, night) {
            Ok(ground) => {
                state.ground.insert(reading.patch(), ground);
            }
            Err(err) => return failed(err, state, Some(context)),
        }
    }
    let count = |g: Ground| {
        frame
            .readings
            .iter()
            .filter(|r| state.ground.get(&r.patch()) == Some(&g))
            .count()
    };
    let mut logs = EffectLog::new();
    logs.add_entry(&format!(
        "t={} s: Ground: of {} patches below, {} safe, {} too steep, {} water, {} with a person, {} unsure.",
        frame.telemetry.time_s(),
        frame.readings.len(),
        count(Ground::Safe),
        count(Ground::Steep),
        count(Ground::Water),
        count(Ground::Person),
        count(Ground::Unsure),
    ));
    FailsafeProcess::new(
        Ok(CausalEffect::value(frame.telemetry)),
        state,
        Some(context),
        logs,
    )
}

/// Stage 4: the detector collection says whether anything is wrong; only then does each detector
/// say what. A lost fix and a lost link are confirmed only once they have persisted. Each fault
/// that is confirmed or clears is recorded in the log.
pub fn detect(
    value: CausalEffect<Telemetry>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
    detectors: &[Detector],
) -> FailsafeProcess<Faults> {
    let Some(telemetry) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let input = PropagatingEffect::pure(telemetry);
    let (outcome, _, _, mut logs) = detectors
        .evaluate_collection(&input, &AggregateLogic::Any, None)
        .into_parts();
    let anything_wrong = match outcome {
        Ok(effect) => effect.into_value() == Some(true),
        Err(err) => return failed(err, state, ctx),
    };

    let mut fired = [false; 4];
    if anything_wrong {
        for (slot, detector) in fired.iter_mut().zip(detectors) {
            match detector.evaluate(&input).into_parts().0 {
                Ok(effect) => *slot = effect.into_value() == Some(true),
                Err(err) => return failed(err, state, ctx),
            }
        }
    }
    let [degraded, no_fix, link, battery] = fired;
    state.no_fix_for_s = if no_fix { state.no_fix_for_s + 1 } else { 0 };
    state.link_down_for_s = if link { state.link_down_for_s + 1 } else { 0 };
    let faults = Faults {
        gnss_degraded: degraded,
        gnss_lost: state.no_fix_for_s >= GNSS_CONFIRM_S,
        link_lost: state.link_down_for_s >= LINK_CONFIRM_S,
        battery_critical: battery,
    };

    let before = state.faults;
    let changes = [
        (
            before.gnss_degraded,
            faults.gnss_degraded,
            format!(
                "Satellite fix degraded: {:.0} satellites, HDOP {:.1}.",
                lower(telemetry.satellites()),
                lower(telemetry.hdop())
            ),
            "Satellite fix no longer degraded.".to_string(),
        ),
        (
            before.gnss_lost,
            faults.gnss_lost,
            format!("Satellite fix lost: none for {GNSS_CONFIRM_S} s."),
            "Satellite fix restored.".to_string(),
        ),
        (
            before.link_lost,
            faults.link_lost,
            format!(
                "Command link lost: {:.0} % of packets dropped for {LINK_CONFIRM_S} s.",
                lower(telemetry.link_loss_pct())
            ),
            "Command link restored.".to_string(),
        ),
        (
            before.battery_critical,
            faults.battery_critical,
            format!(
                "Battery critical: one cell at {:.2} V.",
                lower(telemetry.min_cell_v())
            ),
            "Battery no longer critical.".to_string(),
        ),
    ];
    for (was, is, raised, cleared) in changes {
        if is && !was {
            logs.add_entry(&format!("t={} s: {raised}", telemetry.time_s()));
        } else if was && !is {
            logs.add_entry(&format!("t={} s: {cleared}", telemetry.time_s()));
        }
    }
    state.faults = faults;
    FailsafeProcess::new(Ok(CausalEffect::value(faults)), state, ctx, logs)
}

/// Stage 5: the fail-safe ladder of part 1, unchanged. Each step up the ladder is recorded in the
/// log with its reason.
pub fn decide(
    value: CausalEffect<Faults>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
    time_s: usize,
) -> FailsafeProcess<Command> {
    let Some(faults) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let (proposed, reason) = if faults.battery_critical {
        (Command::LandNow, "land now: the battery is critical.")
    } else if faults.gnss_lost && faults.link_lost {
        (
            Command::LandNow,
            "land now: with no satellite fix and no link, the drone can neither navigate nor receive orders.",
        )
    } else if faults.gnss_lost {
        (
            Command::Hold,
            "hold: no satellite fix, but the link is up, so wait for the fix or the operator.",
        )
    } else if faults.link_lost {
        (
            Command::ReturnHome,
            "return home: the link is lost, but the fix holds.",
        )
    } else {
        (Command::Continue, "continue: no fault.")
    };
    let mut logs = EffectLog::new();
    if severity(proposed) > severity(state.failsafe) {
        logs.add_entry(&format!("t={time_s} s: Fail-safe set to {reason}"));
        state.failsafe = proposed;
    }
    let failsafe = state.failsafe;
    FailsafeProcess::new(Ok(CausalEffect::value(failsafe)), state, ctx, logs)
}

/// Stage 6: the fail-safe machine turns the ladder's command into a maneuver, and flies only what
/// the Effect Ethos approves. Each second the Ethos reviews the plan in force again. When no plan
/// is in force and the ladder says land now, the machine proposes landings on the safe patches
/// nearest the drone, then ditchings on the nearest water, until the Ethos approves one or lets it
/// stand pending a look at unseen ground. Each plan that stands replaces the land and look states
/// with a new version. Every proposal, verdict and change of the state running goes to the log.
pub fn act(
    value: CausalEffect<Command>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
    machine: &FailsafeMachine,
    ethos: &GroundEthos,
    time_s: usize,
) -> FailsafeProcess<Maneuver> {
    let Some(failsafe) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let Some(context) = ctx else {
        return failed(CausalityError::MissingContext(), state, None);
    };
    let mut logs = EffectLog::new();
    if let Err(err) = plan(
        &context, &mut state, machine, ethos, failsafe, time_s, &mut logs,
    ) {
        return failed(err, state, Some(context));
    }

    let mut maneuver = Maneuver::None;
    let mut running = None;
    for id in [HOLD_STATE, HOME_STATE, LAND_STATE, LOOK_STATE] {
        match run(machine, id, situation(&state, failsafe), &mut logs) {
            Ok(Maneuver::None) => {}
            Ok(m) => {
                maneuver = m;
                let version = if id == LAND_STATE || id == LOOK_STATE {
                    state.plan_version
                } else {
                    1
                };
                running = Some((id, version));
            }
            Err(err) => return failed(err, state, Some(context)),
        }
    }
    if maneuver == Maneuver::None && failsafe == Command::LandNow {
        maneuver = Maneuver::ChooseTarget;
    }
    if running != state.running {
        if let Some((id, version)) = running {
            logs.add_entry(&format!(
                "t={time_s} s: Fail-safe machine runs \"{}\", state {id}, version {version}.",
                state_name(id)
            ));
        }
        state.running = running;
    }
    FailsafeProcess::new(
        Ok(CausalEffect::value(maneuver)),
        state,
        Some(context),
        logs,
    )
}

/// The clearance the Effect Ethos demands between a touchdown and any person, in m: the base
/// clearance, widened by the drone's position error and a gust.
pub fn clearance_m(context: &GroundContext) -> Result<FloatType, CausalityError> {
    Ok(PERSON_CLEARANCE_M + point_value(context, POSITION_ERROR_ID)? + GUST_MARGIN_M)
}

/// The person norm's check: whether, on any seen patch within the clearance of `patch`, a person
/// is judged present at [`CONFIDENCE`], the confidence the judge names a person at.
pub fn person_within_clearance(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    let land_c = point_value(context, LAND_TEMPERATURE_ID)?;
    for near in patches_within(context, patch)? {
        if seen(context, near) && holds(&person_belief(context, near, land_c)?)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The not-ruled-out norm's check: whether, on any patch within the clearance of `patch`, a person
/// is not ruled out at [`PERSON_RULED_OUT`]. Unseen ground rules out no one; seen ground rules a
/// person out once enough frames agree. Flying over the patch and looking settles both.
pub fn person_not_ruled_out(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    let land_c = point_value(context, LAND_TEMPERATURE_ID)?;
    for near in patches_within(context, patch)? {
        if !seen(context, near) {
            return Ok(true);
        }
        let no_person = !person_belief(context, near, land_c)?;
        let ruled_out = no_person
            .probability_exceeds(
                &SampleSession::seeded(JUDGE_SEED),
                PERSON_RULED_OUT,
                PERSON_RULED_OUT,
                PERSON_TEST_EPSILON,
                PERSON_TEST_SAMPLES,
            )
            .map_err(uncertain_error)?;
        if !ruled_out {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The battery norm's check: whether flying to `patch` and descending onto it takes longer than the
/// battery lasts, less the reserve. A critical battery descends at the emergency rate.
pub fn battery_falls_short(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    let at = (
        point_value(context, DRONE_ACROSS_ID)?,
        point_value(context, DRONE_ALONG_ID)?,
    );
    let agl = point_value(context, DRONE_AGL_ID)?;
    let endurance = point_value(context, ENDURANCE_ID)?;
    let descent = if endurance <= CRITICAL_ENDURANCE_S {
        EMERGENCY_DESCENT_M_S
    } else {
        LANDING_DESCENT_M_S
    };
    let needed = distance(patch_centre(patch), at) / APPROACH_SPEED_M_S + agl / descent;
    Ok(needed > endurance - RESERVE_S)
}

/// The drone norm's check: whether the drone would not survive touching down on `patch`, which it
/// survives only on seen ground that is, at the stated confidence, not steep and has a surface.
pub fn drone_would_be_lost(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    if !seen(context, patch) {
        return Ok(true);
    }
    let steep = belief(context, patch, Quantity::Slope)?.greater_than(TIP_OVER_DEG);
    let no_return = belief(context, patch, Quantity::Returns)?.less_than(NO_RETURN_BELOW);
    Ok(!holds(&!(steep | no_return))?)
}

/// Reviews the plan in force with the Effect Ethos and, when no plan is in force and the choose
/// state fires, puts new proposals to it.
fn plan(
    context: &GroundContext,
    state: &mut FailsafeState,
    machine: &FailsafeMachine,
    ethos: &GroundEthos,
    failsafe: Command,
    time_s: usize,
    logs: &mut EffectLog,
) -> Result<(), CausalityError> {
    if let Some(plan) = state.plan
        && let Some(patch) = plan.target.patch
    {
        let status = match review(ethos, context, patch, plan.proposal)? {
            Review::Approved => Some(PlanStatus::Approved),
            Review::Look => Some(PlanStatus::Look),
            Review::Rejected(norms) => {
                logs.add_entry(&format!(
                    "t={time_s} s: The Effect Ethos withdrew the patch at {:.0} m across and {:.0} m along: {}. Choosing again.",
                    lower(plan.target.centre.0),
                    lower(plan.target.centre.1),
                    reasons(context, &norms)?,
                ));
                None
            }
        };
        match status {
            Some(status) => {
                if status == PlanStatus::Approved && plan.status != PlanStatus::Approved {
                    logs.add_entry(&format!(
                        "t={time_s} s: The Effect Ethos approves the patch at {:.0} m across and {:.0} m along: a person is now ruled out within {:.0} m of it, and no other norm forbids it.",
                        lower(plan.target.centre.0),
                        lower(plan.target.centre.1),
                        lower(clearance_m(context)?),
                    ));
                }
                state.plan = Some(Plan { status, ..plan });
            }
            None => state.plan = None,
        }
    }
    if run(machine, CHOOSE_STATE, situation(state, failsafe), logs)? == Maneuver::ChooseTarget {
        propose(context, state, machine, ethos, time_s, logs)?;
    }
    Ok(())
}

/// Puts proposals to the Effect Ethos, nearest first, until one stands. A round in which none
/// stands is logged once; the drone holds over the ground and the machine proposes again each
/// second.
fn propose(
    context: &GroundContext,
    state: &mut FailsafeState,
    machine: &FailsafeMachine,
    ethos: &GroundEthos,
    time_s: usize,
    logs: &mut EffectLog,
) -> Result<(), CausalityError> {
    let clearance = clearance_m(context)?;
    let mut entries = Vec::new();
    let mut rejected = Vec::new();
    for (n, candidate) in candidates(state).into_iter().enumerate() {
        let Candidate {
            proposal,
            patch,
            centre,
        } = candidate;
        let what = format!(
            "t={time_s} s: Proposal {}: {} the patch {:.0} m away, at {:.0} m across and {:.0} m along.",
            n + 1,
            match proposal {
                Proposal::Land => "land on",
                Proposal::Ditch => "ditch the drone on",
            },
            lower(distance(centre, state.position)),
            lower(centre.0),
            lower(centre.1),
        );
        let status = match review(ethos, context, patch, proposal)? {
            Review::Approved => {
                entries.push(format!("{what} Approved."));
                PlanStatus::Approved
            }
            Review::Look => {
                entries.push(format!(
                    "{what} Not yet: a person is not yet ruled out within {:.0} m of it, so the drone flies over it to look.",
                    lower(clearance)
                ));
                PlanStatus::Look
            }
            Review::Rejected(norms) => {
                entries.push(format!("{what} Rejected: {}.", reasons(context, &norms)?));
                rejected.extend(norms);
                continue;
            }
        };
        let target = LandingTarget {
            patch: Some(patch),
            centre,
        };
        state.plan_version += 1;
        for (state_action, name) in [
            (landing_state(state.plan_version, target), "land"),
            (look_state(state.plan_version, target), "look"),
        ] {
            machine.update_single_state(state_action).map_err(|err| {
                CausalityError::ActionError(format!("replacing the {name} state: {err}"))
            })?;
        }
        state.plan = Some(Plan {
            target,
            proposal,
            status,
        });
        entries.push(format!(
            "t={time_s} s: Land and look states replaced by version {}.",
            state.plan_version
        ));
        break;
    }
    if state.plan.is_none() {
        if state.round_failed {
            return Ok(());
        }
        entries.push(format!(
            "t={time_s} s: No proposal passes the Effect Ethos; the drone holds over the ground."
        ));
        state.round_failed = true;
    } else {
        state.round_failed = false;
    }
    for norm in rejected {
        *state.rejections.entry(norm).or_default() += 1;
    }
    for entry in entries {
        logs.add_entry(&entry);
    }
    Ok(())
}

/// How the Effect Ethos rules on a proposal against `context`. The drone norm counts as defeated
/// when its check holds but it does not forbid the proposal: the sacrifice norm overrode it.
pub fn ruling(
    ethos: &GroundEthos,
    context: &GroundContext,
    kind: Ground,
    candidate: Candidate,
) -> Result<Ruling, CausalityError> {
    let review = review(ethos, context, candidate.patch, candidate.proposal)?;
    let norms = match &review {
        Review::Approved => Vec::new(),
        Review::Look => vec![NORM_NOT_RULED_OUT],
        Review::Rejected(norms) => norms.clone(),
    };
    Ok(Ruling {
        kind,
        candidate,
        why: reasons(context, &norms)?,
        drone_norm_defeated: drone_would_be_lost(context, candidate.patch)?
            && !norms.contains(&NORM_DRONE),
        review,
    })
}

/// The patch nearest `at` that the controller judged as `kind`, with its centre.
pub fn nearest_judged(
    state: &FailsafeState,
    kind: Ground,
    at: (FloatType, FloatType),
) -> Option<((i64, i64), (FloatType, FloatType))> {
    state
        .ground
        .iter()
        .filter(|(_, g)| **g == kind)
        .filter_map(|(patch, _)| state.fusion.get(patch).map(|f| (*patch, f.centre)))
        .min_by(|a, b| {
            distance(a.1, at)
                .partial_cmp(&distance(b.1, at))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// The proposals the machine puts to the Effect Ethos: landings on the safe patches nearest the
/// drone, then ditchings on the nearest water, at most [`MAX_PROPOSALS`] of each.
fn candidates(state: &FailsafeState) -> Vec<Candidate> {
    let nearest = |kind: Ground, proposal: Proposal| {
        let mut found: Vec<Candidate> = state
            .ground
            .iter()
            .filter(|(_, g)| **g == kind)
            .filter_map(|(patch, _)| {
                state.fusion.get(patch).map(|f| Candidate {
                    proposal,
                    patch: *patch,
                    centre: f.centre,
                })
            })
            .collect();
        found.sort_by(|a, b| {
            distance(a.centre, state.position)
                .partial_cmp(&distance(b.centre, state.position))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        found.truncate(MAX_PROPOSALS);
        found
    };
    let mut all = nearest(Ground::Safe, Proposal::Land);
    all.extend(nearest(Ground::Water, Proposal::Ditch));
    all
}

/// The Effect Ethos's verdict on one proposal. A proposal forbidden by the not-ruled-out norm alone
/// may stand while the drone looks.
fn review(
    ethos: &GroundEthos,
    context: &GroundContext,
    patch: (i64, i64),
    proposal: Proposal,
) -> Result<Review, CausalityError> {
    let parameters = HashMap::from([
        ("across".to_string(), ActionParameterValue::Integer(patch.0)),
        ("along".to_string(), ActionParameterValue::Integer(patch.1)),
    ]);
    let name = match proposal {
        Proposal::Land => "land",
        Proposal::Ditch => "ditch",
    };
    let action = ProposedAction::new(0, name.to_string(), parameters);
    let verdict = ethos
        .evaluate_action(&action, context, &[LANDING_TAG])
        .map_err(|err| CausalityError::DeonticError(err.to_string()))?;
    if verdict.outcome() != TeloidModal::Impermissible {
        return Ok(Review::Approved);
    }
    let mut norms: Vec<TeloidID> = verdict
        .justification()
        .iter()
        .copied()
        .filter(|id| {
            ethos
                .get_norm(*id)
                .is_some_and(|norm| norm.modality() == TeloidModal::Impermissible)
        })
        .collect();
    norms.sort_unstable();
    Ok(if norms == [NORM_NOT_RULED_OUT] {
        Review::Look
    } else {
        Review::Rejected(norms)
    })
}

/// Why the norms forbid a proposal, as one clause per norm.
fn reasons(context: &GroundContext, norms: &[TeloidID]) -> Result<String, CausalityError> {
    let clearance = lower(clearance_m(context)?);
    Ok(norms
        .iter()
        .map(|norm| match *norm {
            NORM_PERSON => format!("a person is within {clearance:.0} m of it"),
            NORM_NOT_RULED_OUT => {
                format!("a person is not yet ruled out within {clearance:.0} m of it")
            }
            NORM_BATTERY => format!(
                "the battery cannot get the drone there and down with {:.0} s to spare",
                lower(RESERVE_S)
            ),
            NORM_DRONE => "the drone would not survive the touchdown".to_string(),
            _ => format!("norm {norm}"),
        })
        .collect::<Vec<_>>()
        .join("; "))
}

/// The distance from a point to the nearest patch the controller judged to hold a person, in m.
/// `None` when it has seen no one.
pub fn nearest_person_seen_m(
    state: &FailsafeState,
    at: (FloatType, FloatType),
) -> Option<FloatType> {
    state
        .ground
        .iter()
        .filter(|(_, g)| **g == Ground::Person)
        .filter_map(|(patch, _)| state.fusion.get(patch))
        .map(|f| distance(f.centre, at))
        .fold(None, |nearest, d| match nearest {
            Some(n) if n <= d => Some(n),
            _ => Some(d),
        })
}

/// The situation the fail-safe machine reads: the ladder's command, the drone's position, and
/// where the plan in force stands.
fn situation(state: &FailsafeState, failsafe: Command) -> Situation {
    Situation {
        failsafe,
        position: state.position,
        plan: state.plan.map_or(PlanStatus::None, |p| p.status),
    }
}

/// Evaluates one state of the machine and appends its log.
fn run(
    machine: &FailsafeMachine,
    id: usize,
    situation: Situation,
    logs: &mut EffectLog,
) -> Result<Maneuver, CausalityError> {
    let effect = machine
        .eval_single_state(id, &PropagatingEffect::pure(situation))
        .map_err(|err| CausalityError::ActionError(err.to_string()))?;
    let (outcome, _, _, mut entries) = effect.into_parts();
    logs.append(&mut entries);
    Ok(outcome?.into_value().unwrap_or_default())
}

/// The belief that a person stands on a patch: a hot spot well above the land, or, where the LiDAR
/// finds a surface, a bump as tall as a person.
fn person_belief(
    context: &GroundContext,
    patch: (i64, i64),
    land_c: FloatType,
) -> Result<UncertainBool<FloatType>, CausalityError> {
    let no_return = belief(context, patch, Quantity::Returns)?.less_than(NO_RETURN_BELOW);
    Ok(
        belief(context, patch, Quantity::HotSpot)?.greater_than(land_c + PERSON_EXCESS_C)
            | (!no_return
                & belief(context, patch, Quantity::Protrusion)?.greater_than(PERSON_BUMP_M)),
    )
}

/// The patches whose centres lie within the clearance of `patch`'s centre, `patch` included.
fn patches_within(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<Vec<(i64, i64)>, CausalityError> {
    let radius = clearance_m(context)?;
    let reach = lower(Real::ceil(radius / PATCH_SIDE_M)) as i64;
    let centre = patch_centre(patch);
    Ok((-reach..=reach)
        .flat_map(|di| (-reach..=reach).map(move |dj| (patch.0 + di, patch.1 + dj)))
        .filter(|near| distance(patch_centre(*near), centre) <= radius)
        .collect())
}

/// Whether the camera and LiDAR have seen a patch.
fn seen(context: &GroundContext, patch: (i64, i64)) -> bool {
    context.get_node_index_by_id(node_id(patch, 0)).is_some()
}

/// The centre of a patch, in m across and along.
fn patch_centre(patch: (i64, i64)) -> (FloatType, FloatType) {
    let half = PATCH_SIDE_M / (ONE + ONE);
    (
        lift_i64::<FloatType>(patch.0) * PATCH_SIDE_M + half,
        lift_i64::<FloatType>(patch.1) * PATCH_SIDE_M + half,
    )
}

/// Writes the drone's own state into the context: where the frames place it, its height, how far
/// that place may be off, and how long its battery lasts.
fn record_drone(
    context: &mut GroundContext,
    state: &mut FailsafeState,
    frame: &Frame,
) -> Result<(), ContextIndexError> {
    let t = &frame.telemetry;
    if t.min_cell_v() < CRITICAL_CELL_V && state.critical_since_s.is_none() {
        state.critical_since_s = Some(t.time_s());
    }
    let endurance = match state.critical_since_s {
        Some(since) => CRITICAL_ENDURANCE_S - lift_usize::<FloatType>(t.time_s() - since),
        None => (t.min_cell_v() - CRITICAL_CELL_V) / CELL_DRAIN_V_PER_S + CRITICAL_ENDURANCE_S,
    };
    let position_error = POSITION_DRIFT_M_PER_S * lift_usize::<FloatType>(state.no_fix_for_s);
    for (id, value) in [
        (DRONE_ACROSS_ID, frame.position.0),
        (DRONE_ALONG_ID, frame.position.1),
        (DRONE_AGL_ID, t.altitude_agl_m()),
        (POSITION_ERROR_ID, position_error),
        (ENDURANCE_ID, endurance),
    ] {
        set_point(context, id, value)?;
    }
    Ok(())
}

/// Sets a data node of the drone's own state to a known value.
fn set_point(
    context: &mut GroundContext,
    id: ContextoidId,
    value: FloatType,
) -> Result<(), ContextIndexError> {
    context.update_node(
        id,
        Contextoid::new(
            id,
            ContextoidType::Datoid(UncertainData::new(id, Uncertain::point(value))),
        ),
    )
}

/// The value of a data node of the drone's own state.
fn point_value(context: &GroundContext, id: ContextoidId) -> Result<FloatType, CausalityError> {
    let node = context
        .get_node_index_by_id(id)
        .and_then(|index| context.get_node(index))
        .ok_or(CausalityError::MissingContext())?;
    match node.vertex_type() {
        ContextoidType::Datoid(data) => data
            .get_data()
            .sample_at(&SampleSession::seeded(JUDGE_SEED), 0)
            .map_err(uncertain_error),
        _ => Err(CausalityError::MissingContext()),
    }
}

fn distance(a: (FloatType, FloatType), b: (FloatType, FloatType)) -> FloatType {
    let (dx, dy) = (a.0 - b.0, a.1 - b.1);
    Real::sqrt(dx * dx + dy * dy)
}

fn state_name(id: usize) -> &'static str {
    match id {
        CHOOSE_STATE => "choose a landing patch",
        HOLD_STATE => "hold over the ground",
        HOME_STATE => "fly home",
        LAND_STATE => "land on the approved patch",
        _ => "fly over the patch and look",
    }
}

/// Fuses one frame into the context and the fusion sums. A patch seen for the first time gets a
/// space node at its centre and one data node per quantity, each linked to the space node.
fn fuse(
    context: &mut GroundContext,
    state: &mut FailsafeState,
    frame: &Frame,
) -> Result<(), ContextIndexError> {
    let clock = DiscreteTime::new(CLOCK_ID, TimeScale::Second, clock_s(&frame.telemetry));
    context.update_node(
        CLOCK_ID,
        Contextoid::new(CLOCK_ID, ContextoidType::Tempoid(clock)),
    )?;
    for reading in &frame.readings {
        let patch = reading.patch();
        let first_sight = !state.fusion.contains_key(&patch);
        let fusion = state.fusion.entry(patch).or_default();
        fusion.centre = reading.centre();
        for q in Quantity::ALL {
            let k = q as usize;
            let weight = ONE / (reading.sigma(q) * reading.sigma(q));
            fusion.precision[k] += weight;
            fusion.weighted[k] += weight * reading.value(q);
        }
        let fused = *fusion;
        if first_sight {
            let (x, y) = reading.centre();
            let space_id = node_id(patch, 0);
            let space = context.add_node(Contextoid::new(
                space_id,
                ContextoidType::Spaceoid(EuclideanSpace::new(space_id, x, y, ZERO)),
            ))?;
            for q in Quantity::ALL {
                let id = node_id(patch, q as u64 + 1);
                let data = context.add_node(data_node(id, &fused, q))?;
                context.add_edge(space, data, RelationKind::Spatial)?;
            }
        } else {
            for q in Quantity::ALL {
                let id = node_id(patch, q as u64 + 1);
                context.update_node(id, data_node(id, &fused, q))?;
            }
        }
    }
    Ok(())
}

/// A data node holding the fused value of one quantity as a normal distribution.
fn data_node(id: ContextoidId, fusion: &Fusion, q: Quantity) -> GroundNode {
    let k = q as usize;
    let mean = fusion.weighted[k] / fusion.precision[k];
    let sigma = Real::sqrt(ONE / fusion.precision[k]);
    Contextoid::new(
        id,
        ContextoidType::Datoid(UncertainData::new(id, Uncertain::normal(mean, sigma))),
    )
}

/// What the context says about one patch. A patch is safe when, with the stated confidence, it is
/// neither steep, nor a person, and the LiDAR found a surface on it. Otherwise the first hazard
/// that holds with that confidence names it, people first; a patch with none is unsure.
fn judge_patch(
    context: &GroundContext,
    patch: (i64, i64),
    land_c: FloatType,
    night: bool,
) -> Result<Ground, CausalityError> {
    let read = |q: Quantity| belief(context, patch, q);
    let steep = read(Quantity::Slope)?.greater_than(TIP_OVER_DEG);
    let no_return = read(Quantity::Returns)?.less_than(NO_RETURN_BELOW);
    let contrast = if night {
        read(Quantity::MeanTemperature)?.greater_than(land_c + WATER_CONTRAST_C)
    } else {
        read(Quantity::MeanTemperature)?.less_than(land_c - WATER_CONTRAST_C)
    };
    let water = no_return.clone() & contrast;
    let person = person_belief(context, patch, land_c)?;

    if holds(&!(steep.clone() | no_return | person.clone()))? {
        return Ok(Ground::Safe);
    }
    for (ground, hazard) in [
        (Ground::Person, person),
        (Ground::Water, water),
        (Ground::Steep, steep),
    ] {
        if holds(&hazard)? {
            return Ok(ground);
        }
    }
    Ok(Ground::Unsure)
}

/// Whether an uncertain condition holds with probability above [`CONFIDENCE`], decided by a
/// sequential test that stops drawing once the evidence settles it. The draws are seeded, so every
/// run judges alike.
fn holds(condition: &UncertainBool<FloatType>) -> Result<bool, CausalityError> {
    condition
        .probability_exceeds(
            &SampleSession::seeded(JUDGE_SEED),
            CONFIDENCE,
            CONFIDENCE,
            TEST_EPSILON,
            MAX_SAMPLES,
        )
        .map_err(uncertain_error)
}

/// The fused value of one quantity of one patch, as the context holds it.
fn belief(
    context: &GroundContext,
    patch: (i64, i64),
    q: Quantity,
) -> Result<Uncertain<FloatType>, CausalityError> {
    let node = context
        .get_node_index_by_id(node_id(patch, q as u64 + 1))
        .and_then(|index| context.get_node(index))
        .ok_or(CausalityError::MissingContext())?;
    match node.vertex_type() {
        ContextoidType::Datoid(data) => Ok(data.get_data()),
        _ => Err(CausalityError::MissingContext()),
    }
}

/// Whether the context's clock reads night.
fn is_night(context: &GroundContext) -> Result<bool, CausalityError> {
    let node = context
        .get_node_index_by_id(CLOCK_ID)
        .and_then(|index| context.get_node(index))
        .ok_or(CausalityError::MissingContext())?;
    let ContextoidType::Tempoid(clock) = node.vertex_type() else {
        return Err(CausalityError::MissingContext());
    };
    let hour = clock.time_unit() / SECONDS_PER_HOUR % HOURS_PER_DAY;
    Ok(!(DAWN_HOUR..DUSK_HOUR).contains(&hour))
}

/// The temperature of the land in this frame: the median fused temperature of the patches the
/// LiDAR found a surface on. `None` when the frame holds no land.
fn land_temperature_c(state: &FailsafeState, frame: &Frame) -> Option<FloatType> {
    let returns = Quantity::Returns as usize;
    let temperature = Quantity::MeanTemperature as usize;
    let mut land: Vec<FloatType> = frame
        .readings
        .iter()
        .filter_map(|r| state.fusion.get(&r.patch()))
        .filter(|f| f.weighted[returns] / f.precision[returns] >= NO_RETURN_BELOW)
        .map(|f| f.weighted[temperature] / f.precision[temperature])
        .collect();
    land.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    land.get(land.len() / 2).copied()
}

/// The local time of day of a telemetry sample, in seconds since midnight.
fn clock_s(telemetry: &Telemetry) -> u64 {
    lower(FLIGHT_START_HOUR) as u64 * SECONDS_PER_HOUR + telemetry.time_s() as u64
}

/// The context identifier of a patch's node: slot 0 is its space node, slots 1 to 5 its data
/// nodes. Patch indices stay within a thousand of zero, so the identifiers never meet the clock's.
fn node_id(patch: (i64, i64), slot: u64) -> ContextoidId {
    let (i, j) = patch;
    (((i + 1000) as u64) * 10_000 + (j + 1000) as u64) * 8 + slot
}

fn severity(command: Command) -> u8 {
    match command {
        Command::Continue => 0,
        Command::Hold => 1,
        Command::ReturnHome => 2,
        Command::LandNow => 3,
    }
}

fn graph_error(err: ContextIndexError) -> CausalityError {
    CausalityError::GraphError(err.to_string())
}

fn uncertain_error(err: UncertainError) -> CausalityError {
    CausalityError::UncertainError(err.to_string())
}

fn failed<V>(
    err: CausalityError,
    state: FailsafeState,
    ctx: Option<GroundContext>,
) -> FailsafeProcess<V> {
    FailsafeProcess::new(Err(err), state, ctx, EffectLog::new())
}
