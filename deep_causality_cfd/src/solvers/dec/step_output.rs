/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Per-step and per-run results of the DEC Navier–Stokes march.

use deep_causality_num::RealField;

use deep_causality_physics::SolenoidalField;

/// The result of one projected march step: the new divergence-free state
/// together with the diagnostics the step already computed — callers do
/// not recompute them.
#[derive(Debug, Clone, PartialEq)]
pub struct StepOutput<R: RealField> {
    state: SolenoidalField<R>,
    max_speed: R,
    divergence_residual: R,
}

impl<R: RealField> StepOutput<R> {
    pub(crate) fn new(state: SolenoidalField<R>, max_speed: R, divergence_residual: R) -> Self {
        Self {
            state,
            max_speed,
            divergence_residual,
        }
    }

    /// The projected (divergence-free) state after the step.
    pub fn state(&self) -> &SolenoidalField<R> {
        &self.state
    }

    /// Maximum pointwise speed of the projected state — the `sharp`-based
    /// value the CFL guard evaluated.
    pub fn max_speed(&self) -> R {
        self.max_speed
    }

    /// Post-projection divergence residual `‖δu♭‖_∞`.
    pub fn divergence_residual(&self) -> R {
        self.divergence_residual
    }

    /// Consumes the output, yielding the state.
    pub fn into_state(self) -> SolenoidalField<R> {
        self.state
    }
}

/// The result of a multi-step run: the final state, how many steps ran,
/// and whether the stop predicate was satisfied (always `true` for a
/// completed fixed-horizon run).
#[derive(Debug, Clone, PartialEq)]
pub struct RunOutput<R: RealField> {
    state: SolenoidalField<R>,
    steps: usize,
    satisfied: bool,
}

impl<R: RealField> RunOutput<R> {
    pub(crate) fn new(state: SolenoidalField<R>, steps: usize, satisfied: bool) -> Self {
        Self {
            state,
            steps,
            satisfied,
        }
    }

    /// The final projected state.
    pub fn state(&self) -> &SolenoidalField<R> {
        &self.state
    }

    /// Number of steps that ran.
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// Whether the stop condition was met (fixed-horizon runs: `true`;
    /// predicate runs: `false` when the step bound was exhausted first).
    pub fn satisfied(&self) -> bool {
        self.satisfied
    }

    /// Consumes the output, yielding the final state.
    pub fn into_state(self) -> SolenoidalField<R> {
        self.state
    }
}
