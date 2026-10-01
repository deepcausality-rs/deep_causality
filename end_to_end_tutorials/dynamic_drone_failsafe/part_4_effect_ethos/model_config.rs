/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fault detectors, one causaloid per fault; the ground context the controller starts with;
//! the fail-safe machine, one causal state per maneuver; and the Effect Ethos, one norm per rule a
//! landing must pass.

use crate::constants::*;
use crate::model;
use crate::model_types::{
    FailsafeMachine, GroundContext, GroundEthos, LandingTarget, Maneuver, PlanStatus, Situation,
};
use deep_causality::{
    ActionError, ActionParameterValue, CSM, CausalAction, CausalEffect, CausalState, Causaloid,
    EffectLog, PropagatingEffect, PropagatingProcess, ProposedAction,
};
use deep_causality_context::{
    ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, DiscreteTime, TimeScale,
    UncertainData,
};
use deep_causality_ethos::{DeonticError, EffectEthos, TeloidModal};
use deep_causality_uncertain::Uncertain;
use dynamic_drone_failsafe::{Command, Telemetry};

pub type Detector = Causaloid<Telemetry, bool, (), ()>;

/// The fault detectors, in the order of their identifiers.
pub fn detectors() -> Vec<Detector> {
    vec![
        Causaloid::new(1, gnss_degraded, "satellite positioning degraded"),
        Causaloid::new(2, gnss_no_fix, "satellite positioning without a fix"),
        Causaloid::new(3, link_down, "command link down"),
        Causaloid::new(4, battery_critical, "battery critical"),
    ]
}

fn gnss_degraded(t: Telemetry) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(t.satellites() < DEGRADED_SATELLITES || t.hdop() > DEGRADED_HDOP)
}

fn gnss_no_fix(t: Telemetry) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(t.satellites() < NO_FIX_SATELLITES)
}

fn link_down(t: Telemetry) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(t.link_loss_pct() > LINK_DOWN_LOSS_PCT)
}

fn battery_critical(t: Telemetry) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(t.min_cell_v() < CRITICAL_CELL_V)
}

/// A ground context with the clock, set to midnight, and the six nodes of the drone's own state,
/// set to zero. The controller sets them each second and adds the patches as the camera and LiDAR
/// see them.
pub fn ground_context() -> Result<GroundContext, ContextIndexError> {
    let mut context =
        GroundContext::with_capacity(GROUND_CONTEXT_ID, "ground below the drone", 4096);
    context.add_node(Contextoid::new(
        CLOCK_ID,
        ContextoidType::Tempoid(DiscreteTime::new(CLOCK_ID, TimeScale::Second, 0)),
    ))?;
    for id in [
        DRONE_ACROSS_ID,
        DRONE_ALONG_ID,
        DRONE_AGL_ID,
        POSITION_ERROR_ID,
        ENDURANCE_ID,
        LAND_TEMPERATURE_ID,
    ] {
        context.add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(UncertainData::new(id, Uncertain::point(ZERO))),
        ))?;
    }
    Ok(context)
}

/// The fail-safe machine. Each state's causaloid reads the situation and returns a maneuver; the
/// machine fires the state's action when the maneuver is not `None`. The land and look states
/// start at version 1 with no patch; the controller replaces both with a new version each time the
/// Effect Ethos lets a plan stand.
pub fn failsafe_machine() -> FailsafeMachine {
    let choose = machine_state(CHOOSE_STATE, choose_a_patch, "choose a landing patch");
    let hold = machine_state(HOLD_STATE, hold_over_the_ground, "hold over the ground");
    let home = machine_state(HOME_STATE, fly_home, "fly home");
    let land = landing_state(1, LandingTarget::default());
    let look = look_state(1, LandingTarget::default());
    CSM::new(&[
        (&choose.0, &choose.1),
        (&hold.0, &hold.1),
        (&home.0, &home.1),
        (&land.0, &land.1),
        (&look.0, &look.1),
    ])
}

/// The landing state at a version, landing on `target`. Its causaloid holds the target as its
/// context.
pub fn landing_state(
    version: usize,
    target: LandingTarget,
) -> (
    CausalState<Situation, Maneuver, LandingTarget>,
    CausalAction,
) {
    target_state(
        LAND_STATE,
        version,
        target,
        land_on_the_patch,
        "land on the approved patch",
    )
}

/// The look state at a version, flying over `target` at height. Its causaloid holds the target as
/// its context.
pub fn look_state(
    version: usize,
    target: LandingTarget,
) -> (
    CausalState<Situation, Maneuver, LandingTarget>,
    CausalAction,
) {
    target_state(
        LOOK_STATE,
        version,
        target,
        fly_over_and_look,
        "fly over the patch and look",
    )
}

/// The Effect Ethos. Every norm reads the proposal and the ground context:
///
/// | Norm | Forbids a proposal when | Priority |
/// |------|-------------------------|----------|
/// | person | a person is judged present, at 95 %, on any patch within the clearance | 4 |
/// | not ruled out | a person is not ruled out, at 99 %, on every patch within the clearance; unseen ground rules out no one | 4 |
/// | battery | the battery cannot get the drone there and down with reserve | 3 |
/// | drone | the drone would not survive the touchdown | 2 |
///
/// The sacrifice norm permits ditching the drone, and defeats the drone norm by its higher
/// priority. Nothing defeats the two person norms: losing the drone is acceptable, harming a person
/// is not. The landing norm permits every proposal no other norm forbids.
pub fn effect_ethos() -> Result<GroundEthos, DeonticError> {
    let mut ethos = EffectEthos::new()
        .add_deterministic_norm(
            NORM_LANDING,
            "land",
            &[LANDING_TAG],
            any_proposal,
            TeloidModal::Optional(0),
            1,
            1,
            1,
        )?
        .add_deterministic_norm(
            NORM_PERSON,
            "land",
            &[LANDING_TAG],
            person_within_clearance,
            TeloidModal::Impermissible,
            1,
            1,
            4,
        )?
        .add_deterministic_norm(
            NORM_NOT_RULED_OUT,
            "land",
            &[LANDING_TAG],
            person_not_ruled_out,
            TeloidModal::Impermissible,
            1,
            1,
            4,
        )?
        .add_deterministic_norm(
            NORM_BATTERY,
            "land",
            &[LANDING_TAG],
            battery_falls_short,
            TeloidModal::Impermissible,
            1,
            1,
            3,
        )?
        .add_deterministic_norm(
            NORM_DRONE,
            "land",
            &[LANDING_TAG],
            drone_would_be_lost,
            TeloidModal::Impermissible,
            1,
            1,
            2,
        )?
        .add_deterministic_norm(
            NORM_SACRIFICE,
            "ditch",
            &[LANDING_TAG],
            ditching_proposed,
            TeloidModal::Optional(0),
            1,
            1,
            3,
        )?
        .link_defeasance(NORM_SACRIFICE, NORM_DRONE)?;
    ethos.verify_graph()?;
    Ok(ethos)
}

fn machine_state(
    id: usize,
    causal_fn: fn(Situation) -> PropagatingEffect<Maneuver>,
    description: &'static str,
) -> (
    CausalState<Situation, Maneuver, LandingTarget>,
    CausalAction,
) {
    (
        CausalState::new(
            id,
            1,
            PropagatingEffect::pure(Situation::default()),
            Causaloid::new(id as u64, causal_fn, description),
            None,
        ),
        CausalAction::new(hand_over, description, 1),
    )
}

type TargetFn = fn(
    CausalEffect<Situation>,
    (),
    Option<LandingTarget>,
) -> PropagatingProcess<Maneuver, (), LandingTarget>;

fn target_state(
    id: usize,
    version: usize,
    target: LandingTarget,
    causal_fn: TargetFn,
    description: &'static str,
) -> (
    CausalState<Situation, Maneuver, LandingTarget>,
    CausalAction,
) {
    (
        CausalState::new(
            id,
            version,
            PropagatingEffect::pure(Situation::default()),
            Causaloid::new_with_context(id as u64, causal_fn, target, description),
            None,
        ),
        CausalAction::new(hand_over, description, 1),
    )
}

/// Active when the ladder says land now and no plan is in force.
fn choose_a_patch(s: Situation) -> PropagatingEffect<Maneuver> {
    let active = s.failsafe == Command::LandNow && s.plan == PlanStatus::None;
    PropagatingEffect::pure(if active {
        Maneuver::ChooseTarget
    } else {
        Maneuver::None
    })
}

/// Active when the ladder says hold: hover where the drone is.
fn hold_over_the_ground(s: Situation) -> PropagatingEffect<Maneuver> {
    PropagatingEffect::pure(if s.failsafe == Command::Hold {
        Maneuver::HoldOver {
            x: s.position.0,
            y: s.position.1,
        }
    } else {
        Maneuver::None
    })
}

/// Active when the ladder says return home.
fn fly_home(s: Situation) -> PropagatingEffect<Maneuver> {
    PropagatingEffect::pure(if s.failsafe == Command::ReturnHome {
        Maneuver::ReturnHome
    } else {
        Maneuver::None
    })
}

/// Active when the ladder says land now and the Effect Ethos approves the plan: fly to the patch
/// and land.
fn land_on_the_patch(
    effect: CausalEffect<Situation>,
    _: (),
    target: Option<LandingTarget>,
) -> PropagatingProcess<Maneuver, (), LandingTarget> {
    let maneuver = match (effect.into_value(), target) {
        (Some(s), Some(t))
            if t.patch.is_some()
                && s.failsafe == Command::LandNow
                && s.plan == PlanStatus::Approved =>
        {
            Maneuver::LandOn {
                x: t.centre.0,
                y: t.centre.1,
            }
        }
        _ => Maneuver::None,
    };
    PropagatingProcess::new(
        Ok(CausalEffect::value(maneuver)),
        (),
        target,
        EffectLog::new(),
    )
}

/// Active when the ladder says land now and the Effect Ethos forbids the plan only because a person
/// is not yet ruled out near it: fly over the patch at height and look.
fn fly_over_and_look(
    effect: CausalEffect<Situation>,
    _: (),
    target: Option<LandingTarget>,
) -> PropagatingProcess<Maneuver, (), LandingTarget> {
    let maneuver = match (effect.into_value(), target) {
        (Some(s), Some(t))
            if t.patch.is_some()
                && s.failsafe == Command::LandNow
                && s.plan == PlanStatus::Look =>
        {
            Maneuver::LookOver {
                x: t.centre.0,
                y: t.centre.1,
            }
        }
        _ => Maneuver::None,
    };
    PropagatingProcess::new(
        Ok(CausalEffect::value(maneuver)),
        (),
        target,
        EffectLog::new(),
    )
}

/// The action every state fires. The maneuver travels in the state's output to the drone; firing
/// records the hand-over in the machine's log.
fn hand_over() -> Result<(), ActionError> {
    Ok(())
}

/// The landing norm: every proposal is permitted unless another norm forbids it.
fn any_proposal(_: &GroundContext, _: &ProposedAction) -> bool {
    true
}

/// The sacrifice norm: active when the machine proposes to ditch the drone.
fn ditching_proposed(_: &GroundContext, action: &ProposedAction) -> bool {
    action.action_name() == "ditch"
}

/// The person norm. A norm that cannot read what it needs forbids the proposal.
fn person_within_clearance(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::person_within_clearance(context, patch).unwrap_or(true))
}

/// The not-ruled-out norm. A norm that cannot read what it needs forbids the proposal.
fn person_not_ruled_out(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::person_not_ruled_out(context, patch).unwrap_or(true))
}

/// The battery norm. A norm that cannot read what it needs forbids the proposal.
fn battery_falls_short(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::battery_falls_short(context, patch).unwrap_or(true))
}

/// The drone norm. A norm that cannot read what it needs forbids the proposal.
fn drone_would_be_lost(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::drone_would_be_lost(context, patch).unwrap_or(true))
}

/// The patch a proposal names.
fn proposed_patch(action: &ProposedAction) -> Option<(i64, i64)> {
    match (
        action.parameters().get("across"),
        action.parameters().get("along"),
    ) {
        (Some(ActionParameterValue::Integer(i)), Some(ActionParameterValue::Integer(j))) => {
            Some((*i, *j))
        }
        _ => None,
    }
}
