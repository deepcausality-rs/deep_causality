/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fault detectors, one causaloid per fault; the ground context the controller starts with;
//! and the fail-safe machine, one causal state per maneuver.

use super::constants::*;
use super::model_context::{
    FailsafeMachine, Ground, GroundContext, LandingTarget, Maneuver, Situation,
};
use deep_causality::{
    ActionError, CSM, CausalAction, CausalEffect, CausalState, Causaloid, EffectLog,
    PropagatingEffect, PropagatingProcess,
};
use deep_causality_context::{
    ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, DiscreteTime, TimeScale,
};
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

/// An empty ground context with one node: the clock, set to midnight. The controller sets it each
/// second and adds the patches as the camera and LiDAR see them.
pub fn ground_context() -> Result<GroundContext, ContextIndexError> {
    let mut context =
        GroundContext::with_capacity(GROUND_CONTEXT_ID, "ground below the drone", 4096);
    context.add_node(Contextoid::new(
        CLOCK_ID,
        ContextoidType::Tempoid(DiscreteTime::new(CLOCK_ID, TimeScale::Second, 0)),
    ))?;
    Ok(context)
}

/// The fail-safe machine. Each state's causaloid reads the situation and returns a maneuver; the
/// machine fires the state's action when the maneuver is not `None`. The landing state starts at
/// version 1 with no patch; the controller replaces it with a new version each time it chooses a
/// patch.
pub fn failsafe_machine() -> FailsafeMachine {
    let choose = machine_state(CHOOSE_STATE, choose_a_patch, "choose a landing patch");
    let hold = machine_state(HOLD_STATE, hold_over_the_ground, "hold over the ground");
    let home = machine_state(HOME_STATE, fly_home, "fly home");
    let land = landing_state(1, LandingTarget::default());
    CSM::new(&[
        (&choose.0, &choose.1),
        (&hold.0, &hold.1),
        (&home.0, &home.1),
        (&land.0, &land.1),
    ])
}

/// The landing state at a version, flying to `target`. Its causaloid holds the target as its
/// context.
pub fn landing_state(
    version: usize,
    target: LandingTarget,
) -> (
    CausalState<Situation, Maneuver, LandingTarget>,
    CausalAction,
) {
    let causaloid = Causaloid::new_with_context(
        LAND_STATE as u64,
        land_on_the_patch,
        target,
        "land on the chosen patch",
    );
    (
        CausalState::new(
            LAND_STATE,
            version,
            PropagatingEffect::pure(Situation::default()),
            causaloid,
            None,
        ),
        CausalAction::new(hand_over, "land on the chosen patch", 1),
    )
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

/// Active when the ladder says land now and no landing patch is chosen, or the chosen one no
/// longer reads as safe.
fn choose_a_patch(s: Situation) -> PropagatingEffect<Maneuver> {
    let active = s.failsafe == Command::LandNow && s.target_ground != Some(Ground::Safe);
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

/// Active when the ladder says land now and the chosen patch reads as safe: fly to it and land.
fn land_on_the_patch(
    effect: CausalEffect<Situation>,
    _: (),
    target: Option<LandingTarget>,
) -> PropagatingProcess<Maneuver, (), LandingTarget> {
    let maneuver = match (effect.into_value(), target) {
        (Some(s), Some(t))
            if t.patch.is_some()
                && s.failsafe == Command::LandNow
                && s.target_ground == Some(Ground::Safe) =>
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

/// The action every state fires. The maneuver travels in the state's output to the drone; firing
/// records the hand-over in the machine's log.
fn hand_over() -> Result<(), ActionError> {
    Ok(())
}
