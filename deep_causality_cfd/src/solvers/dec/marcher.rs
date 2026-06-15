/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! [`Marcher`] realization for the DEC solver: one projected RK4 step under the
//! per-step [`Ambient`].

use crate::solvers::dec::DecNsSolver;
use crate::solvers::dec::step_output::StepOutput;
use crate::traits::Marcher;
use crate::types::{Ambient, CfdScalar};
use deep_causality_physics::{PhysicsError, SolenoidalField};

impl<'m, const D: usize, R: CfdScalar> Marcher<R> for DecNsSolver<'m, D, R> {
    type State = SolenoidalField<R>;
    type Ambient = Ambient<R>;
    type Output = StepOutput<R>;

    /// Advance one projected step. The ambient `ν` is applied to the rate before the
    /// step (between-step write; the four RK4 stages then read it). With a constant
    /// ambient this reproduces the construction-fixed march bit-for-bit.
    fn advance(
        &self,
        state: &SolenoidalField<R>,
        ambient: &Ambient<R>,
    ) -> Result<StepOutput<R>, PhysicsError> {
        self.rate().set_nu(*ambient.nu());
        self.step(state)
    }
}
