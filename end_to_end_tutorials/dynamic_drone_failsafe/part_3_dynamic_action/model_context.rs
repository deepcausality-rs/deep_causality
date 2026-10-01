/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the fail-safe controller's causal process carries from one second to the next, and what
//! its fail-safe machine reads and returns.

use deep_causality::{
    ActionParameterValue, CSM, CausalityError, CsmEvaluable, PropagatingProcess, UncertainParameter,
};
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

/// The fail-safe machine: a causal state machine that turns the ladder's command into a maneuver
/// over the ground.
pub type FailsafeMachine = CSM<Situation, Maneuver, LandingTarget>;

/// One second's input: the drone's telemetry, one frame of the thermal camera and LiDAR, and the
/// drone's ground position as the frames place it.
#[derive(Debug, Clone)]
pub struct Frame {
    pub telemetry: Telemetry,
    pub readings: Vec<PatchReading>,
    pub position: (FloatType, FloatType),
}

/// What the fail-safe machine reads each second: the ladder's command, where the drone is, and
/// what the context says about the landing patch, if one is chosen.
#[derive(Debug, Clone, Copy, Default)]
pub struct Situation {
    pub failsafe: Command,
    pub position: (FloatType, FloatType),
    pub target_ground: Option<Ground>,
}

/// The patch the landing state flies to. The landing state's causaloid holds it as its context.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LandingTarget {
    pub patch: Option<(i64, i64)>,
    pub centre: (FloatType, FloatType),
}

/// What the fail-safe machine tells the drone to do.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Maneuver {
    /// No state is active: the drone follows the ladder's command.
    #[default]
    None,
    /// Hover over a ground point.
    HoldOver { x: FloatType, y: FloatType },
    /// Fly back to the launch point.
    ReturnHome,
    /// Fly to the landing patch and descend onto it.
    LandOn { x: FloatType, y: FloatType },
    /// Choose a landing patch.
    ChooseTarget,
}

impl CsmEvaluable for Maneuver {
    fn is_active(&self, _: Option<&UncertainParameter>) -> Result<bool, CausalityError> {
        Ok(*self != Maneuver::None)
    }

    fn to_action_param(&self) -> ActionParameterValue {
        ActionParameterValue::String(format!("{self:?}"))
    }
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

/// The running sums that fuse every frame of one patch: its centre and, for each quantity, the sum of the
/// readings' inverse variances and the sum of the readings weighted by them.
#[derive(Debug, Clone, Copy, Default)]
pub struct Fusion {
    pub centre: (FloatType, FloatType),
    pub precision: [FloatType; 5],
    pub weighted: [FloatType; 5],
}

/// What the controller remembers: how long the fix and the link have been gone, the faults it has
/// confirmed and the fail-safe it has latched; the fusion sums of every patch seen, its judgement
/// of each, and the temperature of the land around it; where the drone is, the landing patch, the
/// landing state's version, and the machine state running.
#[derive(Debug, Clone, Default)]
pub struct FailsafeState {
    pub no_fix_for_s: usize,
    pub link_down_for_s: usize,
    pub faults: Faults,
    pub failsafe: Command,
    pub fusion: BTreeMap<(i64, i64), Fusion>,
    pub ground: BTreeMap<(i64, i64), Ground>,
    pub land_temperature_c: Option<FloatType>,
    pub position: (FloatType, FloatType),
    pub target: LandingTarget,
    pub landing_version: usize,
    pub running: Option<(usize, usize)>,
}
