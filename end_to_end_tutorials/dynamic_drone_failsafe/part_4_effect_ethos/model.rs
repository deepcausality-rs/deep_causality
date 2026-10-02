/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fail-safe controller, as six stages of one causal process: read the telemetry and the
//! sensor frame, fuse the frame and the drone's own state into the ground context, judge each patch
//! from the context, detect the faults with a collection of causaloids, assess the urgency after
//! the contingency and emergency procedures pilots follow, and turn it into a maneuver with the
//! fail-safe machine, which flies only what the Effect Ethos permits. The checks the Ethos's norms
//! run are here too.

use super::constants::*;
use super::model_config::{Detector, landing_state, look_state};
use super::model_context::{
    Candidate, FailsafeMachine, FailsafeProcess, FailsafeState, Faults, Frame, Fusion, Ground,
    GroundContext, GroundEthos, GroundNode, LandingTarget, Maneuver, Plan, PlanStatus, Proposal,
    Review, Ruling, Situation, Urgency,
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
use dynamic_drone_failsafe::{Command, Drone, FloatType, Guidance, Quantity, Telemetry, Terrain};
use std::collections::{BTreeMap, HashMap};

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

/// Stage 5: assesses the urgency after the contingency and emergency procedures pilots follow. A
/// battery with no more endurance than an emergency allows is an emergency: land as soon as
/// possible. A lost fix or link, or a low battery, is a contingency: hold, wait out the recovery
/// window, then land as soon as practicable.
/// With no fault the mission continues. An emergency, and a last resort, last for the rest of the
/// flight. Each change goes to the log with what the drone does about it.
pub fn assess(
    value: CausalEffect<Faults>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
    time_s: usize,
) -> FailsafeProcess<Urgency> {
    let Some(faults) = value.into_value() else {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    };
    let Some(context) = ctx else {
        return failed(CausalityError::MissingContext(), state, None);
    };
    let emergency = match battery_critical(&context) {
        Ok(critical) => critical,
        Err(err) => return failed(err, state, Some(context)),
    };
    let found = if emergency {
        Urgency::Possible
    } else if faults.gnss_lost || faults.link_lost || faults.battery_critical {
        Urgency::Practicable
    } else {
        Urgency::Routine
    };
    let urgency = match state.urgency {
        Urgency::Possible | Urgency::LastResort => state.urgency,
        _ => found,
    };
    let mut logs = EffectLog::new();
    if urgency != state.urgency {
        match urgency {
            Urgency::Routine => {
                logs.add_entry(&format!(
                    "t={time_s} s: The satellite fix and the command link are back. The drone resumes its mission."
                ));
                let landing = state
                    .waiting_since_s
                    .is_some_and(|since| time_s >= since + RECOVERY_WINDOW_S)
                    && state.plan.is_some_and(|p| p.status == PlanStatus::Approved);
                if !landing {
                    state.plan = None;
                }
                state.waiting_since_s = None;
            }
            Urgency::Practicable => {
                logs.add_entry(&format!(
                    "t={time_s} s: Contingency: {}. The battery is healthy, so the drone holds over the ground and waits up to {RECOVERY_WINDOW_S} s for recovery.",
                    lost(&faults)
                ));
                state.waiting_since_s = Some(time_s);
            }
            _ => {
                logs.add_entry(&format!(
                    "t={time_s} s: Emergency: {}. The drone lands as soon as possible and may be ditched; it may not endanger a person.",
                    if state.cell_failed_at_s.is_some() {
                        "a cell has failed"
                    } else {
                        "the battery is nearly empty"
                    }
                ));
                state.plan = None;
                state.waiting_since_s = None;
            }
        }
        state.urgency = urgency;
    } else if urgency == Urgency::Practicable
        && state.waiting_since_s == Some(time_s.saturating_sub(RECOVERY_WINDOW_S))
    {
        logs.add_entry(&format!(
            "t={time_s} s: No recovery after {RECOVERY_WINDOW_S} s. The drone lands as soon as practicable."
        ));
    }
    FailsafeProcess::new(Ok(CausalEffect::value(urgency)), state, Some(context), logs)
}

/// Stage 6: the fail-safe machine turns the urgency into a maneuver, and flies only what the
/// Effect Ethos permits. Each second the Ethos reviews the plan in force again. When the drone
/// must land and no plan is in force, the machine puts every candidate to the Ethos and flies the
/// cheapest permitted one, nearest first among equal costs. In a contingency a landing may stand
/// while the drone looks; in an emergency with nothing permitted the last-resort norms apply, so the
/// drone never holds until it falls. Every round, plan and change of the state running goes to the
/// log.
pub fn act(
    value: CausalEffect<Urgency>,
    mut state: FailsafeState,
    ctx: Option<GroundContext>,
    machine: &FailsafeMachine,
    ethos: &GroundEthos,
    time_s: usize,
) -> FailsafeProcess<Maneuver> {
    if value.into_value().is_none() {
        return failed(CausalityError::ValueNotAvailable(), state, ctx);
    }
    let Some(context) = ctx else {
        return failed(CausalityError::MissingContext(), state, None);
    };
    let mut logs = EffectLog::new();
    if let Err(err) = plan(&context, &mut state, machine, ethos, time_s, &mut logs) {
        return failed(err, state, Some(context));
    }

    let mut maneuver = Maneuver::None;
    let mut running = None;
    for id in [HOLD_STATE, LAND_STATE, LOOK_STATE] {
        match run(machine, id, situation(&state, time_s), &mut logs) {
            Ok(Maneuver::None) => {}
            Ok(m) => {
                maneuver = m;
                let version = if id == HOLD_STATE {
                    1
                } else {
                    state.plan_version
                };
                running = Some((id, version));
            }
            Err(err) => return failed(err, state, Some(context)),
        }
    }
    if maneuver == Maneuver::None && state.urgency != Urgency::Routine {
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

/// Flies the drone for one second: a maneuver over the ground by its camera and LiDAR, the mission
/// when nothing is wrong, or a hover where it is when nothing is permitted yet.
pub fn steer(drone: &mut Drone, terrain: &Terrain, maneuver: Maneuver, urgency: Urgency) {
    match maneuver {
        Maneuver::HoldOver { x, y } | Maneuver::Explore { x, y } => {
            drone.guide(Guidance::HoldOver { x, y }, terrain)
        }
        Maneuver::LookOver { x, y } => drone.guide(
            Guidance::DescendOver {
                x,
                y,
                agl_m: LOOK_HEIGHT_M,
            },
            terrain,
        ),
        Maneuver::LandOn { x, y } => drone.guide(Guidance::LandOn { x, y }, terrain),
        Maneuver::None if urgency == Urgency::Routine => drone.step(Command::Continue, terrain),
        Maneuver::None | Maneuver::ChooseTarget => {
            let (x, y) = drone.position();
            drone.guide(Guidance::HoldOver { x, y }, terrain)
        }
    }
}

/// The clearance the Effect Ethos demands between a touchdown and any person, in m: the base
/// clearance, widened by the drone's position error and a gust.
pub fn clearance_m(context: &GroundContext) -> Result<FloatType, CausalityError> {
    Ok(PERSON_CLEARANCE_M + point_value(context, POSITION_ERROR_ID)? + GUST_MARGIN_M)
}

/// Whether, on any seen patch within `radius_m` of `patch`, a person is more likely present than
/// not. The judge names a person on its map only at [`CONFIDENCE`]; for safety the Effect Ethos
/// counts one at [`PERSON_LIKELY`], and rules one out only at [`PERSON_RULED_OUT`].
pub fn person_within(
    context: &GroundContext,
    patch: (i64, i64),
    radius_m: FloatType,
) -> Result<bool, CausalityError> {
    let land_c = point_value(context, LAND_TEMPERATURE_ID)?;
    for near in patches_within(patch, radius_m) {
        if !seen(context, near) {
            continue;
        }
        let likely = person_belief(context, near, land_c)?
            .probability_exceeds(
                &SampleSession::seeded(JUDGE_SEED),
                PERSON_LIKELY,
                CONFIDENCE,
                TEST_EPSILON,
                MAX_SAMPLES,
            )
            .map_err(uncertain_error)?;
        if likely {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The not-ruled-out check: whether, on any patch within the clearance of `patch`, a person is
/// not ruled out at [`PERSON_RULED_OUT`]. Unseen ground rules out no one, and neither does ground
/// that may lie under canopy, which hides a person from both sensors.
pub fn person_not_ruled_out(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    let land_c = point_value(context, LAND_TEMPERATURE_ID)?;
    for near in patches_within(patch, clearance_m(context)?) {
        if !seen(context, near) {
            return Ok(true);
        }
        let open = !belief(context, near, Quantity::Protrusion)?.greater_than(CANOPY_LOWEST_M);
        if !holds(&open)? {
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

/// The path check: whether flying straight from the drone to `patch` at the drone's height would
/// cross the canopy without the margin, or cross unseen ground below the unseen ceiling. The patch
/// itself is the drone norm's to judge.
pub fn path_crosses_trees(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    let from = (
        point_value(context, DRONE_ACROSS_ID)?,
        point_value(context, DRONE_ALONG_ID)?,
    );
    let agl = point_value(context, DRONE_AGL_ID)?;
    let to = patch_centre(patch);
    let step = PATCH_SIDE_M / (ONE + ONE);
    let steps = lower(Real::ceil(distance(from, to) / step)) as usize;
    let mut checked = Vec::new();
    for k in 0..=steps {
        let f = if steps == 0 {
            ZERO
        } else {
            lift_usize::<FloatType>(k) / lift_usize::<FloatType>(steps)
        };
        let at = patch_of((from.0 + (to.0 - from.0) * f, from.1 + (to.1 - from.1) * f));
        if at == patch || checked.contains(&at) {
            continue;
        }
        checked.push(at);
        if !seen(context, at) {
            if agl < UNSEEN_CEILING_M {
                return Ok(true);
            }
            continue;
        }
        let clear = belief(context, at, Quantity::Protrusion)?.less_than(agl - CANOPY_MARGIN_M);
        if !holds(&clear)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// The battery check: whether flying to `patch` and descending onto it takes longer than the
/// battery lasts, less the reserve.
pub fn battery_falls_short(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    Ok(time_to_land_s(context, patch)? > point_value(context, ENDURANCE_ID)? - RESERVE_S)
}

/// The drone check: whether the drone would not survive touching down on `patch`, which it
/// survives only on seen open ground that is, at the stated confidence, not steep and has a
/// surface.
pub fn drone_would_be_lost(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    if !seen(context, patch) {
        return Ok(true);
    }
    let steep = belief(context, patch, Quantity::Slope)?.greater_than(TIP_OVER_DEG);
    let no_return = belief(context, patch, Quantity::Returns)?.less_than(NO_RETURN_BELOW);
    let trees = belief(context, patch, Quantity::Protrusion)?.greater_than(CANOPY_LOWEST_M);
    Ok(!holds(&!(steep | no_return | trees))?)
}

/// The edge check: whether a gust could put the drone on steep ground beside `patch`: any seen
/// neighbouring patch that is not, at the stated confidence, flat. Unseen neighbours are the
/// not-ruled-out norm's to judge, and a look settles both.
pub fn beside_steep_ground(
    context: &GroundContext,
    patch: (i64, i64),
) -> Result<bool, CausalityError> {
    for (di, dj) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        let near = (patch.0 + di, patch.1 + dj);
        if !seen(context, near) {
            continue;
        }
        let flat = belief(context, near, Quantity::Slope)?.less_than(TIP_OVER_DEG);
        if !holds(&flat)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether the battery is critical: no more endurance is left than an emergency allows.
pub fn battery_critical(context: &GroundContext) -> Result<bool, CausalityError> {
    Ok(point_value(context, ENDURANCE_ID)? <= EMERGENCY_ENDURANCE_S)
}

/// How the Effect Ethos rules on a proposal against `context`, under the norms of `urgency`.
pub fn ruling(
    ethos: &GroundEthos,
    context: &GroundContext,
    kind: Ground,
    candidate: Candidate,
    urgency: Urgency,
) -> Result<Ruling, CausalityError> {
    let review = review(ethos, context, candidate, urgency)?;
    let why = match &review {
        Review::Rejected(norms) => reasons(context, norms)?,
        Review::Look(norms) => reasons(context, norms)?,
        Review::Approved(_) => String::new(),
    };
    Ok(Ruling {
        kind,
        candidate,
        review,
        why,
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

/// The distance from a point to the nearest patch the controller judged to hold a person, in m.
/// `None` when it has seen no one.
pub fn nearest_person_seen_m(
    state: &FailsafeState,
    at: (FloatType, FloatType),
) -> Option<FloatType> {
    nearest_judged(state, Ground::Person, at).map(|(_, centre)| distance(centre, at))
}

/// Reviews the plan in force with the Effect Ethos and, when the choose state fires and a round is
/// due, puts the candidates to it.
fn plan(
    context: &GroundContext,
    state: &mut FailsafeState,
    machine: &FailsafeMachine,
    ethos: &GroundEthos,
    time_s: usize,
    logs: &mut EffectLog,
) -> Result<(), CausalityError> {
    if let Some(plan) = state.plan
        && plan.status == PlanStatus::Explore
    {
        if distance(plan.target.centre, state.position) < PATCH_SIDE_M
            || time_s >= plan.look_until_s
        {
            logs.add_entry(&format!(
                "t={time_s} s: The drone has looked over new ground. Choosing again."
            ));
            state.plan = None;
        }
    } else if let Some(plan) = state.plan
        && let Some(patch) = plan.target.patch
    {
        let candidate = Candidate {
            proposal: plan.proposal,
            patch,
            centre: plan.target.centre,
        };
        let at = format!(
            "the patch at {:.0} m across and {:.0} m along",
            lower(plan.target.centre.0),
            lower(plan.target.centre.1)
        );
        let status = match review(ethos, context, candidate, plan.urgency)? {
            Review::Approved(_) => {
                if plan.status == PlanStatus::Look {
                    logs.add_entry(&format!(
                        "t={time_s} s: The Effect Ethos permits {at}: a person is now ruled out within {:.0} m of it.",
                        lower(clearance_m(context)?)
                    ));
                }
                Some(PlanStatus::Approved)
            }
            Review::Look(_)
                if state.urgency == Urgency::Practicable && time_s >= plan.look_until_s =>
            {
                logs.add_entry(&format!(
                    "t={time_s} s: After {LOOK_LIMIT_S} s of looking, a person is still not ruled out near {at}. It is set aside. Choosing again."
                ));
                state.set_aside.insert(patch);
                None
            }
            Review::Look(_) if state.urgency == Urgency::Practicable => Some(PlanStatus::Look),
            Review::Look(_) => {
                logs.add_entry(&format!(
                    "t={time_s} s: There is no time left to look at {at}. Choosing again."
                ));
                None
            }
            Review::Rejected(norms) => {
                logs.add_entry(&format!(
                    "t={time_s} s: The Effect Ethos withdrew {at}: {}. Choosing again.",
                    reasons(context, &norms)?
                ));
                None
            }
        };
        state.plan = status.map(|status| Plan {
            status,
            since_s: if status == plan.status {
                plan.since_s
            } else {
                time_s
            },
            ..plan
        });
    }
    let due = state
        .failed_round_s
        .is_none_or(|at| time_s >= at + REASSESS_S || state.urgency >= Urgency::Possible);
    if due && run(machine, CHOOSE_STATE, situation(state, time_s), logs)? == Maneuver::ChooseTarget
    {
        choose(context, state, machine, ethos, time_s, logs)?;
    }
    Ok(())
}

/// One round: puts every candidate to the Effect Ethos and puts the cheapest one in force, sooner
/// first among equal costs. In a contingency a landing that waits only on a look competes too, at no
/// harm cost and with the look's time added, so a near patch worth a look beats a far one known
/// already; with none of either, the drone explores the nearest unseen ground. In an emergency with
/// none permitted, the last-resort norms apply and the round runs again.
fn choose(
    context: &GroundContext,
    state: &mut FailsafeState,
    machine: &FailsafeMachine,
    ethos: &GroundEthos,
    time_s: usize,
    logs: &mut EffectLog,
) -> Result<(), CausalityError> {
    let urgency = state.urgency;
    let mut best: Option<(i64, FloatType, Candidate, PlanStatus)> = None;
    let mut consider = |cost: i64, time: FloatType, candidate: Candidate, status: PlanStatus| {
        if best.is_none_or(|(c, t, _, _)| (cost, time) < (c, t)) {
            best = Some((cost, time, candidate, status));
        }
    };
    let mut forbidden: BTreeMap<String, usize> = BTreeMap::new();
    let candidates = candidates(state, urgency);
    for candidate in &candidates {
        let time = time_to_land_s(context, candidate.patch)?;
        let review = review(ethos, context, *candidate, urgency)?;
        if matches!(review, Review::Look(_)) && urgency == Urgency::Practicable {
            consider(0, time + LOOK_ALLOWANCE_S, *candidate, PlanStatus::Look);
        }
        match review {
            Review::Approved(cost) => consider(cost, time, *candidate, PlanStatus::Approved),
            Review::Look(norms) | Review::Rejected(norms) => {
                for norm in &norms {
                    *state.rejections.entry(*norm).or_default() += 1;
                }
                *forbidden.entry(reasons(context, &norms)?).or_default() += 1;
            }
        }
    }
    logs.add_entry(&format!(
        "t={time_s} s: {} proposals put to the Effect Ethos under the {} norms: {} permitted.",
        candidates.len(),
        urgency_name(urgency),
        candidates.len() - forbidden.values().sum::<usize>(),
    ));
    let chosen = best.map(|(cost, _, candidate, status)| (candidate, status, cost));
    for (why, count) in &forbidden {
        logs.add_entry(&format!("t={time_s} s:   {count} forbidden: {why}."));
    }

    if let Some((candidate, status, cost)) = chosen {
        install(state, machine, candidate, status, urgency, time_s)?;
        logs.add_entry(&format!(
            "t={time_s} s: Chosen: {} the patch {:.0} m away, at {:.0} m across and {:.0} m along; {}. Land and look states replaced by version {}.",
            verb(candidate.proposal),
            lower(distance(candidate.centre, state.position)),
            lower(candidate.centre.0),
            lower(candidate.centre.1),
            if status == PlanStatus::Look {
                "pending a look".to_string()
            } else {
                format!("harm cost {cost}")
            },
            state.plan_version,
        ));
        state.failed_round_s = None;
        return Ok(());
    }
    if urgency == Urgency::Possible {
        logs.add_entry(&format!(
            "t={time_s} s: Last resort: nothing is permitted under the emergency norms. The person bans yield to costs, so the drone comes down where it endangers the fewest, farthest from people."
        ));
        state.urgency = Urgency::LastResort;
        return choose(context, state, machine, ethos, time_s, logs);
    }
    if let Some(frontier) = nearest_unseen(context, state) {
        let candidate = Candidate {
            proposal: Proposal::Land,
            patch: frontier,
            centre: patch_centre(frontier),
        };
        let reason = if path_crosses_trees(context, frontier)? {
            None
        } else {
            Some(candidate)
        };
        if let Some(candidate) = reason {
            install(
                state,
                machine,
                candidate,
                PlanStatus::Explore,
                urgency,
                time_s,
            )?;
            logs.add_entry(&format!(
                "t={time_s} s: Nothing is permitted or worth a look here. The drone flies to unseen ground {:.0} m away, at {:.0} m across and {:.0} m along, to look for a landing site.",
                lower(distance(candidate.centre, state.position)),
                lower(candidate.centre.0),
                lower(candidate.centre.1),
            ));
            state.failed_round_s = None;
            return Ok(());
        }
    }
    logs.add_entry(&format!(
        "t={time_s} s: Nothing is permitted yet; the drone holds over the ground and proposes again in {REASSESS_S} s."
    ));
    state.failed_round_s = Some(time_s);
    Ok(())
}

/// Puts a plan in force: replaces the land and look states with a new version that flies to its
/// patch.
fn install(
    state: &mut FailsafeState,
    machine: &FailsafeMachine,
    candidate: Candidate,
    status: PlanStatus,
    urgency: Urgency,
    time_s: usize,
) -> Result<(), CausalityError> {
    let target = LandingTarget {
        patch: Some(candidate.patch),
        centre: candidate.centre,
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
        proposal: candidate.proposal,
        status,
        urgency,
        since_s: time_s,
        look_until_s: time_s
            + lower(Real::ceil(
                distance(candidate.centre, state.position) / APPROACH_SPEED_M_S,
            )) as usize
            + LOOK_LIMIT_S,
    });
    Ok(())
}

/// The candidates of a round: landings on the safe patches nearest the drone, except those a look
/// could not clear in a contingency, and, in an emergency, ditchings on the nearest other ground and
/// on the patch below the drone.
pub fn candidates(state: &FailsafeState, urgency: Urgency) -> Vec<Candidate> {
    let nearest = |kinds: &[Ground], proposal: Proposal, most: usize| {
        let mut found: Vec<Candidate> = state
            .ground
            .iter()
            .filter(|(_, g)| kinds.contains(g))
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
        found.truncate(most);
        found
    };
    let mut all = nearest(&[Ground::Safe], Proposal::Land, MAX_LANDINGS);
    if urgency == Urgency::Practicable {
        all.retain(|c| !state.set_aside.contains(&c.patch));
    }
    if urgency >= Urgency::Possible {
        all.extend(nearest(
            &[Ground::Unsure, Ground::Steep, Ground::Water, Ground::Trees],
            Proposal::Ditch,
            MAX_DITCHINGS,
        ));
        let below = patch_of(state.position);
        if state.ground.get(&below) != Some(&Ground::Safe)
            && let Some(f) = state.fusion.get(&below)
            && !all.iter().any(|c| c.patch == below)
        {
            all.push(Candidate {
                proposal: Proposal::Ditch,
                patch: below,
                centre: f.centre,
            });
        }
    }
    all
}

/// The tags whose norms judge a proposal at `urgency`.
fn tags(urgency: Urgency) -> &'static [&'static str] {
    match urgency {
        Urgency::Routine | Urgency::Practicable => &[LANDING_TAG],
        Urgency::Possible => &[LANDING_TAG, EMERGENCY_TAG],
        Urgency::LastResort => &[LANDING_TAG, EMERGENCY_TAG, LAST_RESORT_TAG],
    }
}

/// The Effect Ethos's verdict on one proposal under the norms of `urgency`. A proposal forbidden by
/// the not-ruled-out norm, alone or with the edge norm, may stand while the drone looks: the look
/// shows the ground around the patch, where a flat patch clear of people may lie.
pub fn review(
    ethos: &GroundEthos,
    context: &GroundContext,
    candidate: Candidate,
    urgency: Urgency,
) -> Result<Review, CausalityError> {
    let parameters = HashMap::from([
        (
            "across".to_string(),
            ActionParameterValue::Integer(candidate.patch.0),
        ),
        (
            "along".to_string(),
            ActionParameterValue::Integer(candidate.patch.1),
        ),
    ]);
    let name = match candidate.proposal {
        Proposal::Land => "land",
        Proposal::Ditch => "ditch",
    };
    let action = ProposedAction::new(0, name.to_string(), parameters);
    let verdict = ethos
        .evaluate_action(&action, context, tags(urgency))
        .map_err(|err| CausalityError::DeonticError(err.to_string()))?;
    match verdict.outcome() {
        TeloidModal::Optional(cost) => return Ok(Review::Approved(cost)),
        TeloidModal::Obligatory => return Ok(Review::Approved(0)),
        TeloidModal::Impermissible => {}
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
    let look_settles = norms.contains(&NORM_NOT_RULED_OUT)
        && norms
            .iter()
            .all(|n| *n == NORM_NOT_RULED_OUT || *n == NORM_EDGE);
    Ok(if look_settles {
        Review::Look(norms)
    } else {
        Review::Rejected(norms)
    })
}

/// Why the norms forbid a proposal, as one clause per norm.
fn reasons(context: &GroundContext, norms: &[TeloidID]) -> Result<String, CausalityError> {
    let clearance = lower(clearance_m(context)?);
    let agl = lower(point_value(context, DRONE_AGL_ID)?);
    Ok(norms
        .iter()
        .map(|norm| match *norm {
            NORM_PERSON => format!("a person is within {clearance:.0} m of it"),
            NORM_NOT_RULED_OUT => {
                format!("a person is not yet ruled out within {clearance:.0} m of it")
            }
            NORM_PATH => {
                format!("the flight there at {agl:.0} m would cross trees or unseen ground")
            }
            NORM_BATTERY => format!(
                "the battery cannot get the drone there and down with {:.0} s to spare",
                lower(RESERVE_S)
            ),
            NORM_DRONE => "the drone would not survive the touchdown".to_string(),
            NORM_EDGE => "a gust could put the drone on steep ground beside it".to_string(),
            _ => format!("norm {norm}"),
        })
        .collect::<Vec<_>>()
        .join("; "))
}

/// How long flying to `patch` and descending onto it takes, in s, at the drone's descent rate.
fn time_to_land_s(context: &GroundContext, patch: (i64, i64)) -> Result<FloatType, CausalityError> {
    let at = (
        point_value(context, DRONE_ACROSS_ID)?,
        point_value(context, DRONE_ALONG_ID)?,
    );
    let descent = point_value(context, DESCENT_RATE_ID)?;
    Ok(distance(patch_centre(patch), at) / APPROACH_SPEED_M_S
        + point_value(context, DRONE_AGL_ID)? / descent)
}

/// What the faults have taken from the drone, in words.
fn lost(faults: &Faults) -> &'static str {
    match (faults.gnss_lost, faults.link_lost) {
        (true, true) => "the satellite fix and the command link are lost",
        (true, false) => "the satellite fix is lost",
        (false, true) => "the command link is lost",
        (false, false) => "the battery is low",
    }
}

fn urgency_name(urgency: Urgency) -> &'static str {
    match urgency {
        Urgency::Routine | Urgency::Practicable => "contingency",
        Urgency::Possible => "emergency",
        Urgency::LastResort => "last-resort",
    }
}

fn verb(proposal: Proposal) -> &'static str {
    match proposal {
        Proposal::Land => "land on",
        Proposal::Ditch => "ditch the drone on",
    }
}

/// The situation the fail-safe machine reads.
fn situation(state: &FailsafeState, time_s: usize) -> Situation {
    Situation {
        urgency: state.urgency,
        waiting: state.urgency == Urgency::Practicable
            && state
                .waiting_since_s
                .is_some_and(|since| time_s < since + RECOVERY_WINDOW_S),
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
/// finds a surface, a bump as tall as a person and no taller.
fn person_belief(
    context: &GroundContext,
    patch: (i64, i64),
    land_c: FloatType,
) -> Result<UncertainBool<FloatType>, CausalityError> {
    let no_return = belief(context, patch, Quantity::Returns)?.less_than(NO_RETURN_BELOW);
    let bump = belief(context, patch, Quantity::Protrusion)?;
    Ok(
        belief(context, patch, Quantity::HotSpot)?.greater_than(land_c + PERSON_EXCESS_C)
            | (!no_return & bump.greater_than(PERSON_BUMP_M) & bump.less_than(PERSON_TALLEST_M)),
    )
}

/// The unseen patch nearest the drone that borders seen ground, farther than the drone sees from
/// where it hovers: the edge of what it knows.
fn nearest_unseen(context: &GroundContext, state: &FailsafeState) -> Option<(i64, i64)> {
    let below = patch_of(state.position);
    state
        .fusion
        .keys()
        .flat_map(|&(i, j)| [(i - 1, j), (i + 1, j), (i, j - 1), (i, j + 1)])
        .filter(|patch| !seen(context, *patch) && *patch != below)
        .min_by(|a, b| {
            distance(patch_centre(*a), state.position)
                .partial_cmp(&distance(patch_centre(*b), state.position))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// The patches whose centres lie within `radius_m` of `patch`'s centre, `patch` included.
fn patches_within(patch: (i64, i64), radius_m: FloatType) -> Vec<(i64, i64)> {
    let reach = lower(Real::ceil(radius_m / PATCH_SIDE_M)) as i64;
    let centre = patch_centre(patch);
    (-reach..=reach)
        .flat_map(|di| (-reach..=reach).map(move |dj| (patch.0 + di, patch.1 + dj)))
        .filter(|near| distance(patch_centre(*near), centre) <= radius_m)
        .collect()
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

/// The patch containing a point.
fn patch_of(at: (FloatType, FloatType)) -> (i64, i64) {
    (
        lower(Real::floor(at.0 / PATCH_SIDE_M)) as i64,
        lower(Real::floor(at.1 / PATCH_SIDE_M)) as i64,
    )
}

/// Writes the drone's own state into the context: where the frames place it, its height, how far
/// off it may land from a patch it tracks, how long its battery lasts and how fast it descends. A
/// one-second sag in the lowest cell is a failed cell: the battery then lasts a fixed time and the
/// drone descends at the emergency rate. Otherwise the battery lasts as long as the lowest cell takes
/// to drain to empty.
fn record_drone(
    context: &mut GroundContext,
    state: &mut FailsafeState,
    frame: &Frame,
) -> Result<(), ContextIndexError> {
    let t = &frame.telemetry;
    let cell_v = t.min_cell_v();
    if state.cell_failed_at_s.is_none()
        && state.last_cell_v.is_some_and(|v| v - cell_v > CELL_SAG_V)
    {
        state.cell_failed_at_s = Some(t.time_s());
    }
    state.last_cell_v = Some(cell_v);
    let (endurance, descent) = match state.cell_failed_at_s {
        Some(at) => (
            CRITICAL_ENDURANCE_S - lift_usize::<FloatType>(t.time_s() - at),
            EMERGENCY_DESCENT_M_S,
        ),
        None => (
            (cell_v - CELL_EMPTY_V) / CELL_DRAIN_V_PER_S,
            LANDING_DESCENT_M_S,
        ),
    };
    for (id, value) in [
        (DRONE_ACROSS_ID, frame.position.0),
        (DRONE_ALONG_ID, frame.position.1),
        (DRONE_AGL_ID, t.altitude_agl_m()),
        (POSITION_ERROR_ID, LANDING_ACCURACY_M),
        (ENDURANCE_ID, endurance),
        (DESCENT_RATE_ID, descent),
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
    let clock = DiscreteTime::new(CLOCK_ID, TimeScale::Second, frame.telemetry.clock_s());
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
/// neither steep, nor a person, nor under canopy, and the LiDAR found a surface on it. Otherwise
/// the first hazard that holds with that confidence names it, people first; a patch with none is
/// unsure.
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
    let trees = read(Quantity::Protrusion)?.greater_than(CANOPY_LOWEST_M);

    if holds(&!(steep.clone() | no_return | person.clone() | trees.clone()))? {
        return Ok(Ground::Safe);
    }
    for (ground, hazard) in [
        (Ground::Person, person),
        (Ground::Trees, trees),
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

/// The context identifier of a patch's node: slot 0 is its space node, slots 1 to 5 its data
/// nodes. Patch indices stay within a thousand of zero, so the identifiers never meet the clock's.
fn node_id(patch: (i64, i64), slot: u64) -> ContextoidId {
    let (i, j) = patch;
    (((i + 1000) as u64) * 10_000 + (j + 1000) as u64) * 8 + slot
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
