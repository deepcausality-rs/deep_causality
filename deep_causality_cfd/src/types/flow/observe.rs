/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// The set of diagnostics a march collects into its `Report`. Built fluently;
/// extended incrementally (centerline, Strouhal, drag, divergence) as the Flow
/// observe layer grows.
#[derive(Debug, Clone, Copy, Default)]
pub struct Observe {
    pub(crate) kinetic_energy: bool,
    pub(crate) divergence: bool,
    pub(crate) max_speed: bool,
}

impl Observe {
    /// Collect the kinetic-energy series (one sample per step, plus the seed).
    pub fn kinetic_energy(mut self) -> Self {
        self.kinetic_energy = true;
        self
    }

    /// Collect the divergence-residual series (the solver's incompressibility error).
    pub fn divergence(mut self) -> Self {
        self.divergence = true;
        self
    }

    /// Collect the maximum-speed series.
    pub fn max_speed(mut self) -> Self {
        self.max_speed = true;
        self
    }
}
