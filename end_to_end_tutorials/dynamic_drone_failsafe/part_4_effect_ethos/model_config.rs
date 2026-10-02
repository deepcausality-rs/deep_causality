/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fault detectors, one causaloid per fault; the ground context the controller starts with;
//! the fail-safe machine, one causal state per maneuver; and the Effect Ethos, one norm per rule of
//! the safety protocol.

use super::constants::*;
use super::model;
use super::model_context::{
    FailsafeMachine, GroundContext, GroundEthos, LandingTarget, Maneuver, PlanStatus, Situation,
    Urgency,
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
use dynamic_drone_failsafe::Telemetry;

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

/// A ground context with the clock, set to midnight, and the seven nodes of the drone's own state,
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
        DESCENT_RATE_ID,
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
/// start at version 1 with no patch; the controller replaces both with a new version each time it
/// puts a plan in force.
pub fn failsafe_machine() -> FailsafeMachine {
    let choose = machine_state(CHOOSE_STATE, choose_a_patch, "choose a landing patch");
    let hold = machine_state(HOLD_STATE, hold_over_the_ground, "hold over the ground");
    let land = landing_state(1, LandingTarget::default());
    let look = look_state(1, LandingTarget::default());
    CSM::new(&[
        (&choose.0, &choose.1),
        (&hold.0, &hold.1),
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

/// The Effect Ethos: the safety protocol as norms. Every norm reads the proposal and the ground
/// context.
///
/// | Norm | Tag | Forbids a proposal, or prices it, when | Priority |
/// |------|-----|----------------------------------------|----------|
/// | person | landing | a person is more likely present than not within the clearance | 4 |
/// | not ruled out | landing | a person is not ruled out, at 99 %, everywhere within the clearance | 4 |
/// | path | landing | the flight there at the drone's height would cross the canopy or unseen ground | 3 |
/// | battery | landing | the battery cannot get the drone there and down with reserve | 3 |
/// | drone | landing | the drone would not survive the touchdown | 2 |
/// | edge | landing | a gust could put the drone on seen steep ground beside the patch | 2 |
/// | landing | landing | never; it permits every proposal, at no cost | 1 |
/// | sacrifice | landing | never; it prices a ditching at the drone's loss | 4 |
/// | emergency | emergency | never; once a cell has failed it defeats "not ruled out" | 5 |
/// | last resort | last resort | never; it defeats both person norms | 6 |
/// | near the unknown | emergency, last resort | a person is not ruled out within the clearance: 10⁴ | 1 |
/// | within 20, 10, 5 m | last resort | a person more likely than not lies that close: 10⁵, 10⁶, 10⁷ | 1 |
///
/// The sacrifice norm defeats the drone, edge and battery norms: a drone that will be lost may fly away
/// from people even if its battery dies first. The emergency norm lets a landing stand without a
/// full look when there is no time for one, but a person not ruled out near it still costs ten times
/// the drone, so ditching where people are ruled out beats landing beside the unknown. Only the last-resort norm defeats the person norms, and
/// it puts a price on every person near the patch instead, so the patch farthest from people
/// costs least and a tree always beats a person.
pub fn effect_ethos() -> Result<GroundEthos, DeonticError> {
    let impermissible = TeloidModal::Impermissible;
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
            impermissible,
            1,
            1,
            4,
        )?
        .add_deterministic_norm(
            NORM_NOT_RULED_OUT,
            "land",
            &[LANDING_TAG],
            person_not_ruled_out,
            impermissible,
            1,
            1,
            4,
        )?
        .add_deterministic_norm(
            NORM_PATH,
            "land",
            &[LANDING_TAG],
            path_crosses_trees,
            impermissible,
            1,
            1,
            3,
        )?
        .add_deterministic_norm(
            NORM_BATTERY,
            "land",
            &[LANDING_TAG],
            battery_falls_short,
            impermissible,
            1,
            1,
            3,
        )?
        .add_deterministic_norm(
            NORM_DRONE,
            "land",
            &[LANDING_TAG],
            drone_would_be_lost,
            impermissible,
            1,
            1,
            2,
        )?
        .add_deterministic_norm(
            NORM_EDGE,
            "land",
            &[LANDING_TAG],
            beside_steep_ground,
            impermissible,
            1,
            1,
            2,
        )?
        .add_deterministic_norm(
            NORM_SACRIFICE,
            "ditch",
            &[LANDING_TAG],
            ditching_proposed,
            TeloidModal::Optional(COST_DRONE_LOST),
            1,
            1,
            4,
        )?
        .add_deterministic_norm(
            NORM_EMERGENCY,
            "land",
            &[EMERGENCY_TAG],
            cell_failed,
            TeloidModal::Optional(0),
            1,
            1,
            5,
        )?
        .add_deterministic_norm(
            NORM_LAST_RESORT,
            "land",
            &[LAST_RESORT_TAG],
            any_proposal,
            TeloidModal::Optional(0),
            1,
            1,
            6,
        )?
        .add_deterministic_norm(
            NORM_WITHIN_20_M,
            "land",
            &[LAST_RESORT_TAG],
            person_within_20_m,
            TeloidModal::Optional(COST_WITHIN_20_M),
            1,
            1,
            1,
        )?
        .add_deterministic_norm(
            NORM_WITHIN_10_M,
            "land",
            &[LAST_RESORT_TAG],
            person_within_10_m,
            TeloidModal::Optional(COST_WITHIN_10_M),
            1,
            1,
            1,
        )?
        .add_deterministic_norm(
            NORM_WITHIN_5_M,
            "land",
            &[LAST_RESORT_TAG],
            person_within_5_m,
            TeloidModal::Optional(COST_WITHIN_5_M),
            1,
            1,
            1,
        )?
        .add_deterministic_norm(
            NORM_NEAR_UNKNOWN,
            "land",
            &[EMERGENCY_TAG, LAST_RESORT_TAG],
            person_not_ruled_out,
            TeloidModal::Optional(COST_NOT_RULED_OUT),
            1,
            1,
            1,
        )?
        .link_defeasance(NORM_SACRIFICE, NORM_DRONE)?
        .link_defeasance(NORM_SACRIFICE, NORM_EDGE)?
        .link_defeasance(NORM_SACRIFICE, NORM_BATTERY)?
        .link_defeasance(NORM_EMERGENCY, NORM_NOT_RULED_OUT)?
        .link_defeasance(NORM_LAST_RESORT, NORM_NOT_RULED_OUT)?
        .link_defeasance(NORM_LAST_RESORT, NORM_PERSON)?;
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

/// Active during a fault with no plan in force, the recovery window included: the drone chooses a
/// site while it waits.
fn choose_a_patch(s: Situation) -> PropagatingEffect<Maneuver> {
    let fault = s.urgency != Urgency::Routine;
    PropagatingEffect::pure(if fault && s.plan == PlanStatus::None {
        Maneuver::ChooseTarget
    } else {
        Maneuver::None
    })
}

/// Active during a fault with no plan in force: hover where the drone is.
fn hold_over_the_ground(s: Situation) -> PropagatingEffect<Maneuver> {
    let fault = s.urgency != Urgency::Routine;
    PropagatingEffect::pure(if fault && s.plan == PlanStatus::None {
        Maneuver::HoldOver {
            x: s.position.0,
            y: s.position.1,
        }
    } else {
        Maneuver::None
    })
}

/// Active when the Effect Ethos permits the plan in force and the recovery window has passed: fly to
/// the patch and land.
fn land_on_the_patch(
    effect: CausalEffect<Situation>,
    _: (),
    target: Option<LandingTarget>,
) -> PropagatingProcess<Maneuver, (), LandingTarget> {
    let maneuver = match (effect.into_value(), target) {
        (Some(s), Some(t)) if t.patch.is_some() && s.plan == PlanStatus::Approved && !s.waiting => {
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

/// Active in a contingency while the drone explores unseen ground, or when the plan waits on a look
/// or on the end of the recovery window: fly over the patch and look from the look height.
fn fly_over_and_look(
    effect: CausalEffect<Situation>,
    _: (),
    target: Option<LandingTarget>,
) -> PropagatingProcess<Maneuver, (), LandingTarget> {
    let maneuver = match (effect.into_value(), target) {
        (Some(s), Some(t))
            if t.patch.is_some()
                && s.urgency == Urgency::Practicable
                && s.plan == PlanStatus::Explore =>
        {
            Maneuver::Explore {
                x: t.centre.0,
                y: t.centre.1,
            }
        }
        (Some(s), Some(t))
            if t.patch.is_some()
                && (s.plan == PlanStatus::Look
                    || (s.plan == PlanStatus::Approved && s.waiting)) =>
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

/// The landing and last-resort norms: every proposal is permitted unless another norm forbids it.
fn any_proposal(_: &GroundContext, _: &ProposedAction) -> bool {
    true
}

/// The sacrifice norm: active when the machine proposes to ditch the drone.
fn ditching_proposed(_: &GroundContext, action: &ProposedAction) -> bool {
    action.action_name() == "ditch"
}

/// The emergency norm: active once a cell has failed. A norm that cannot read the battery does not
/// relax anything.
fn cell_failed(context: &GroundContext, _: &ProposedAction) -> bool {
    model::battery_critical(context).unwrap_or(false)
}

/// The person norm. A norm that cannot read what it needs forbids the proposal.
fn person_within_clearance(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action).is_none_or(|patch| {
        model::clearance_m(context)
            .and_then(|clearance| model::person_within(context, patch, clearance))
            .unwrap_or(true)
    })
}

/// The not-ruled-out norm, and the near-the-unknown cost. A norm that cannot read what it needs
/// applies.
fn person_not_ruled_out(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::person_not_ruled_out(context, patch).unwrap_or(true))
}

/// The path norm. A norm that cannot read what it needs forbids the proposal.
fn path_crosses_trees(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::path_crosses_trees(context, patch).unwrap_or(true))
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

/// The edge norm. A norm that cannot read what it needs forbids the proposal.
fn beside_steep_ground(context: &GroundContext, action: &ProposedAction) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::beside_steep_ground(context, patch).unwrap_or(true))
}

/// The band costs: a person more likely than not within 20, 10 or 5 m of the patch. A norm that cannot
/// read what it needs applies its cost.
fn person_within_20_m(context: &GroundContext, action: &ProposedAction) -> bool {
    person_within_band(context, action, BAND_20_M)
}

fn person_within_10_m(context: &GroundContext, action: &ProposedAction) -> bool {
    person_within_band(context, action, BAND_10_M)
}

fn person_within_5_m(context: &GroundContext, action: &ProposedAction) -> bool {
    person_within_band(context, action, BAND_5_M)
}

fn person_within_band(
    context: &GroundContext,
    action: &ProposedAction,
    band_m: dynamic_drone_failsafe::FloatType,
) -> bool {
    proposed_patch(action)
        .is_none_or(|patch| model::person_within(context, patch, band_m).unwrap_or(true))
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
