/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the fail-safe controller's causal process carries from one second to the next.

use deep_causality::PropagatingProcess;
use deep_causality_context::{
    Context, Contextoid, DiscreteTime, EuclideanSpace, NoSpaceTime, UncertainData,
};
use dynamic_drone_failsafe::{Command, FloatType, PatchReading, Telemetry};
use std::collections::BTreeMap;

/// The ground below the drone as the controller believes it to be. Each patch the camera and
/// LiDAR have seen is a space node linked to five uncertain data nodes, one per measured quantity.
/// A clock node holds the local time of day.
pub type GroundContext = Context<
    UncertainData<FloatType>,
    EuclideanSpace<FloatType>,
    DiscreteTime,
    NoSpaceTime<FloatType>,
>;

/// One node of the ground context.
pub type GroundNode = Contextoid<
    UncertainData<FloatType>,
    EuclideanSpace<FloatType>,
    DiscreteTime,
    NoSpaceTime<FloatType>,
>;

/// The controller's causal process, now with the ground as its context.
pub type FailsafeProcess<V> = PropagatingProcess<V, FailsafeState, GroundContext>;

/// One second's input: the drone's telemetry and one frame of the thermal camera and LiDAR.
#[derive(Debug, Clone)]
pub struct Frame {
    pub telemetry: Telemetry,
    pub readings: Vec<PatchReading>,
}

/// The faults the controller has confirmed this second.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Faults {
    pub gnss_degraded: bool,
    pub gnss_lost: bool,
    pub link_lost: bool,
    pub battery_critical: bool,
}

/// What the controller concludes about one ground patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ground {
    Safe,
    Steep,
    Water,
    Person,
    Unsure,
}

/// The moment the fail-safe chose to land now: the time, where the drone was, the patch below it
/// and the controller's judgement of that patch.
#[derive(Debug, Clone, Copy)]
pub struct LandNow {
    pub time_s: usize,
    pub position: (FloatType, FloatType),
    pub patch: (i64, i64),
    pub below: Option<Ground>,
}

/// The running sums that fuse every frame of one patch: for each quantity, the sum of the
/// readings' inverse variances and the sum of the readings weighted by them.
#[derive(Debug, Clone, Copy, Default)]
pub struct Fusion {
    pub precision: [FloatType; 5],
    pub weighted: [FloatType; 5],
}

/// What the controller remembers: how long the fix and the link have been gone, the faults it has
/// confirmed and the fail-safe it has latched; the fusion sums of every patch seen, its judgement
/// of each, and the temperature of the land around it.
#[derive(Debug, Clone, Default)]
pub struct FailsafeState {
    pub no_fix_for_s: usize,
    pub link_down_for_s: usize,
    pub faults: Faults,
    pub failsafe: Command,
    pub fusion: BTreeMap<(i64, i64), Fusion>,
    pub ground: BTreeMap<(i64, i64), Ground>,
    pub land_temperature_c: Option<FloatType>,
}
