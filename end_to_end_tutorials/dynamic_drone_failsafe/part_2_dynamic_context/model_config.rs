/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fault detectors, one causaloid per fault, and the ground context the controller starts
//! with.

use super::constants::*;
use super::model_context::GroundContext;
use deep_causality::{Causaloid, PropagatingEffect};
use deep_causality_context::{
    ContextIndexError, Contextoid, ContextoidType, ContextuableGraph, DiscreteTime, TimeScale,
};
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
