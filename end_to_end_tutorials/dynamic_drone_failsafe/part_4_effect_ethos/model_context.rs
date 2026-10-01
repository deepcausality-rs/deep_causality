/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the fail-safe controller's causal process carries from one second to the next, what its
//! fail-safe machine reads and returns, and what the Effect Ethos judges.

use deep_causality::{
    ActionParameterValue, CSM, CausalityError, CsmEvaluable, PropagatingProcess, UncertainParameter,
};
use deep_causality_context::{
    Context, Contextoid, DiscreteTime, EuclideanSpace, NoSpaceTime, UncertainData,
};
use deep_causality_ethos::{EffectEthos, TeloidID};
use dynamic_drone_failsafe::{FloatType, PatchReading, Telemetry};
use std::collections::{BTreeMap, BTreeSet};

/// The ground below the drone as the controller believes it to be, and the drone's own state. Each
/// patch the camera and LiDAR have seen is a space node linked to five uncertain data nodes, one
/// per measured quantity. A clock node holds the local time of day; six data nodes hold where the
/// drone is, its height, its position error, its battery endurance and the land temperature.
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

/// The Effect Ethos: the norms every landing the fail-safe machine proposes must pass. Its norms
/// read the same context the controller fills.
pub type GroundEthos = EffectEthos<
    UncertainData<FloatType>,
    EuclideanSpace<FloatType>,
    DiscreteTime,
    NoSpaceTime<FloatType>,
>;

/// The fail-safe machine: a causal state machine that turns the urgency of the situation into a
/// maneuver over the ground.
pub type FailsafeMachine = CSM<Situation, Maneuver, LandingTarget>;

/// One second's input: the drone's telemetry, one frame of the thermal camera and LiDAR, and the
/// drone's ground position as the frames place it.
#[derive(Debug, Clone)]
pub struct Frame {
    pub telemetry: Telemetry,
    pub readings: Vec<PatchReading>,
    pub position: (FloatType, FloatType),
}

/// How urgent the situation is, after the contingency and emergency procedures pilots follow.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Urgency {
    /// No fault: fly the mission.
    #[default]
    Routine,
    /// The fix or the link is lost and the battery is healthy: a contingency. Hold and wait for
    /// recovery, then land as soon as practicable.
    Practicable,
    /// A cell has failed or the battery has run down: an emergency. Land as soon as possible; the
    /// drone may be ditched.
    Possible,
    /// No option under the emergency norms is permitted: come down where the fewest are
    /// endangered, farthest from people.
    LastResort,
}

/// What the fail-safe machine reads each second: the urgency, whether the drone is still waiting
/// out the recovery window, where the drone is, and where the plan in force stands with the Effect
/// Ethos.
#[derive(Debug, Clone, Copy, Default)]
pub struct Situation {
    pub urgency: Urgency,
    pub waiting: bool,
    pub position: (FloatType, FloatType),
    pub plan: PlanStatus,
}

/// Where a plan stands with the Effect Ethos.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PlanStatus {
    /// No plan is in force.
    #[default]
    None,
    /// The Ethos forbids the landing only because a person is not yet ruled out near the patch:
    /// fly over the patch and look.
    Look,
    /// The Ethos approves the landing.
    Approved,
    /// Nothing is permitted or worth a look: fly at height to unseen ground and look for a landing
    /// site.
    Explore,
}

/// What the machine proposes: to land on a patch, or to ditch the drone on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Proposal {
    Land,
    Ditch,
}

/// A plan the Effect Ethos has seen: the patch, what the machine proposed for it, where it stands,
/// the urgency whose norms judged it, when it was put in force, and until when a look at it may
/// run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    pub target: LandingTarget,
    pub proposal: Proposal,
    pub status: PlanStatus,
    pub urgency: Urgency,
    pub since_s: usize,
    pub look_until_s: usize,
}

/// One proposal the machine can put to the Effect Ethos: what, on which patch, and its centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    pub proposal: Proposal,
    pub patch: (i64, i64),
    pub centre: (FloatType, FloatType),
}

/// How the Effect Ethos ruled on one proposal: the patch's judgement, the proposal, and the
/// verdict with why it forbids the proposal if it does.
#[derive(Debug, Clone, PartialEq)]
pub struct Ruling {
    pub kind: Ground,
    pub candidate: Candidate,
    pub review: Review,
    pub why: String,
}

/// The Effect Ethos's verdict on one proposal.
#[derive(Debug, Clone, PartialEq)]
pub enum Review {
    /// Permitted, at this harm cost: the sum of the costs of the norms that stand.
    Approved(i64),
    /// Forbidden only by norms a look can settle: a person not yet ruled out near the patch, and
    /// perhaps steep ground beside it. Carries those norms.
    Look(Vec<TeloidID>),
    /// Forbidden by these norms.
    Rejected(Vec<TeloidID>),
}

/// The patch the landing and look states fly to. Their causaloids hold it as their context.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LandingTarget {
    pub patch: Option<(i64, i64)>,
    pub centre: (FloatType, FloatType),
}

/// What the fail-safe machine tells the drone to do.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Maneuver {
    /// No state is active: the drone flies its mission.
    #[default]
    None,
    /// Hover over a ground point.
    HoldOver { x: FloatType, y: FloatType },
    /// Fly to the landing patch and descend onto it.
    LandOn { x: FloatType, y: FloatType },
    /// Fly over a patch, descend to the look height and hover there, so the camera and LiDAR see
    /// the ground around it sharply.
    LookOver { x: FloatType, y: FloatType },
    /// Fly at height to unseen ground and look for a landing site.
    Explore { x: FloatType, y: FloatType },
    /// Choose a landing patch and put it to the Effect Ethos. With nothing approved, the drone
    /// holds over the ground.
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
    Trees,
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
/// confirmed, the urgency and since when the drone has waited for recovery; the fusion sums of
/// every patch seen, its judgement of each, and the temperature of the land around it; where the
/// drone is, when a cell failed and the lowest cell's last voltage, the plan in force, the patches
/// a look could not clear, the version of the states that fly the plan, the machine state running,
/// when the machine last proposed in vain, and how many proposals each norm rejected.
#[derive(Debug, Clone, Default)]
pub struct FailsafeState {
    pub no_fix_for_s: usize,
    pub link_down_for_s: usize,
    pub faults: Faults,
    pub urgency: Urgency,
    pub waiting_since_s: Option<usize>,
    pub fusion: BTreeMap<(i64, i64), Fusion>,
    pub ground: BTreeMap<(i64, i64), Ground>,
    pub land_temperature_c: Option<FloatType>,
    pub position: (FloatType, FloatType),
    pub cell_failed_at_s: Option<usize>,
    pub last_cell_v: Option<FloatType>,
    pub set_aside: BTreeSet<(i64, i64)>,
    pub plan: Option<Plan>,
    pub plan_version: usize,
    pub running: Option<(usize, usize)>,
    pub failed_round_s: Option<usize>,
    pub rejections: BTreeMap<TeloidID, usize>,
}
