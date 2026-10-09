/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The types the gradiometer-crosstalk example is built from.

use crate::FloatType;
use deep_causality_context_store::ContextSnapshot;
use deep_causality_quantum::{ObservedContext, QuantumError};

/// What correlates the two clouds' read-outs beyond the phase they share.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mechanism {
    /// Nothing: the shared phase alone, which differential extraction removes.
    Benign,
    /// Cloud A's fluorescence reaches cloud B's detection signal.
    LeakAToB,
    /// Cloud B's fluorescence reaches cloud A's detection signal.
    LeakBToA,
    /// Both leaks at once: not a candidate, only a world to draw observations from.
    LeakBothWays,
    /// The platform's rotation adds a centrifugal phase to the differential phase.
    Rotation,
}

impl Mechanism {
    /// Whether cloud A's fluorescence reaches cloud B's detection signal.
    pub fn leaks_a_to_b(self) -> bool {
        matches!(self, Mechanism::LeakAToB | Mechanism::LeakBothWays)
    }

    /// Whether cloud B's fluorescence reaches cloud A's detection signal.
    pub fn leaks_b_to_a(self) -> bool {
        matches!(self, Mechanism::LeakBToA | Mechanism::LeakBothWays)
    }

    /// Whether biasing the gradient is what the mechanism does.
    pub fn biases_gradient(self) -> bool {
        !matches!(self, Mechanism::Benign)
    }
}

/// The two clouds' fringes, as the ellipse model of Sorrentino et al. (eq. 1) fits them: each
/// cloud's excitation is its centre minus half the contrast times the cosine of its phase, and
/// cloud B's phase leads cloud A's by the differential phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fringe {
    /// Cloud A's centre.
    pub centre_a: FloatType,
    /// Cloud B's centre.
    pub centre_b: FloatType,
    /// The differential phase, in rad, without the rotation's centrifugal term.
    pub phase: FloatType,
}

/// A cause: a mechanism and the fringe it explains the passive read-out with.
#[derive(Debug, Clone, PartialEq)]
pub struct Cause {
    /// The name the verdict reports.
    pub name: &'static str,
    /// The mechanism.
    pub mechanism: Mechanism,
    /// The fringe.
    pub fringe: Fringe,
}

impl Cause {
    /// The same cause with its mechanism removed: the shared phase alone, over the same fringe.
    pub fn without_mechanism(&self) -> Cause {
        Cause {
            mechanism: Mechanism::Benign,
            ..self.clone()
        }
    }
}

/// The numbers the factors and the responses are built from.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Instrument {
    /// The fringe contrast.
    pub contrast: FloatType,
    /// The excited fraction a dark cloud's detection signal reads.
    pub dark_background: FloatType,
    /// The fraction of one cloud's fluorescence that reaches the other cloud's detection signal,
    /// in the worlds of the leak causes.
    pub leak: FloatType,
    /// The centrifugal phase at the operating rotation rate, in rad.
    pub operating_rotation_phase: FloatType,
    /// The centrifugal phase at the rotation step's rate, in rad.
    pub step_rotation_phase: FloatType,
    /// The differential phase one s⁻² of gravity gradient produces, `k_eff T² L`, in rad·s².
    pub phase_per_gradient: FloatType,
}

/// Which cloud an experiment brightens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cloud {
    /// Cloud A.
    A,
    /// Cloud B.
    B,
}

/// What an experiment sets and reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    /// No change; read both detection signals excited.
    Passive,
    /// Push every atom of one cloud into the bright state, leave the other cloud's atoms dark
    /// with no interferometer pulses, and read the dark cloud's detection signal excited.
    Brighten(Cloud),
    /// Rotate the sensor head at the step rate; read the two detection signals disagreeing.
    RotationStep,
}

/// A setting is not a context, so its observations record none.
impl ObservedContext for Setting {
    fn context_snapshot(&self) -> Result<Option<ContextSnapshot>, QuantumError> {
        Ok(None)
    }
}

/// How a run ended.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// One cause holds and separates from every rival.
    Survivor(String),
    /// No cause holds.
    OutsideTheModel,
    /// Causes remain that nothing offered separates.
    Unresolved,
}

/// One world's run: what the sequential campaign and the static plan concluded, and what each
/// spent.
#[derive(Debug, Clone, PartialEq)]
pub struct WorldRun {
    /// The world the observations were drawn from.
    pub world: &'static str,
    /// The candidate that is the world's own mechanism, if one is.
    pub expected: Option<&'static str>,
    /// The sequential campaign's verdict.
    pub campaign: Verdict,
    /// The experiments the campaign ran, in order.
    pub campaign_experiments: Vec<String>,
    /// What the campaign spent, in s.
    pub campaign_cost: FloatType,
    /// The static plan's verdict, every planned experiment run.
    pub plan: Verdict,
    /// What the static plan costs, in s.
    pub plan_cost: FloatType,
    /// Whether the plan separates every pair at the floor.
    pub plan_complete: bool,
    /// The planned experiments, as indices into the family.
    pub plan_experiments: Vec<usize>,
}

/// What a survivor implies: the differential phase the instrument reads in the survivor's world,
/// the phase it would read with the mechanism removed, and the corrective action.
#[derive(Debug, Clone, PartialEq)]
pub struct Consequence {
    /// The survivor.
    pub cause: &'static str,
    /// Its mechanism.
    pub mechanism: Mechanism,
    /// The differential phase the instrument reads, in rad.
    pub reading: FloatType,
    /// The differential phase it would read without the mechanism, in rad.
    pub unbiased: FloatType,
    /// What to do about it.
    pub action: &'static str,
}

impl Consequence {
    /// The consequence of `cause`, whose world reads `reading` and reads `unbiased` without the
    /// mechanism, with no action.
    pub fn of(cause: &Cause, reading: FloatType, unbiased: FloatType) -> Self {
        Consequence {
            cause: cause.name,
            mechanism: cause.mechanism,
            reading,
            unbiased,
            action: "none: differential extraction removes the shared phase",
        }
    }

    /// The same, with the action that removes the mechanism.
    pub fn corrected(self) -> Self {
        let action = match self.mechanism {
            Mechanism::LeakAToB | Mechanism::LeakBToA | Mechanism::LeakBothWays => {
                "realign the detection optics or add a baffle, then rerun the brightening experiment"
            }
            Mechanism::Rotation => "correct with the gyroscope's rate, or compensate the rotation",
            Mechanism::Benign => self.action,
        };
        Consequence { action, ..self }
    }

    /// The phase bias the mechanism adds to the reading, in rad.
    pub fn bias(&self) -> FloatType {
        self.reading - self.unbiased
    }
}
