/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The run loops over the fallible step. The `EndoArrow` combinators are
//! infallible by signature, so the march carries its `Result` chain in a
//! plain loop and reports the failing step index.

use alloc::format;

use crate::error::physics_error::PhysicsError;
use crate::quantities::fluid_dynamics::solenoidal_field::SolenoidalField;
use crate::theories::fluid_dynamics::dec::step_output::{RunOutput, StepOutput};

use super::DecNsSolver;
use crate::theories::fluid_dynamics::dec::DecNsScalar;

impl<const D: usize, R: DecNsScalar> DecNsSolver<'_, D, R> {
    /// Marches exactly `n` steps from `initial`.
    ///
    /// # Errors
    /// The first step error, annotated with the 1-based step index at
    /// which it occurred.
    pub fn run_n(
        &self,
        initial: SolenoidalField<R>,
        n: usize,
    ) -> Result<RunOutput<R>, PhysicsError> {
        let mut state = initial;
        for i in 1..=n {
            state = self
                .step(&state)
                .map_err(|e| Self::at_step(i, e))?
                .into_state();
        }
        Ok(RunOutput::new(state, n, true))
    }

    /// Marches until `predicate` holds on a produced step output or
    /// `max_steps` is reached. The predicate receives the 1-based step
    /// index and the step output (state plus the per-step diagnostics).
    ///
    /// Returns `satisfied = false` when the bound was exhausted first.
    ///
    /// # Errors
    /// The first step error, annotated with the 1-based step index at
    /// which it occurred.
    pub fn run_until<P>(
        &self,
        initial: SolenoidalField<R>,
        mut predicate: P,
        max_steps: usize,
    ) -> Result<RunOutput<R>, PhysicsError>
    where
        P: FnMut(usize, &StepOutput<R>) -> bool,
    {
        let mut state = initial;
        for i in 1..=max_steps {
            let output = self.step(&state).map_err(|e| Self::at_step(i, e))?;
            if predicate(i, &output) {
                return Ok(RunOutput::new(output.into_state(), i, true));
            }
            state = output.into_state();
        }
        Ok(RunOutput::new(state, max_steps, false))
    }

    /// Annotates a step error with the index at which it occurred.
    fn at_step(index: usize, e: PhysicsError) -> PhysicsError {
        PhysicsError::CalculationError(format!("march failed at step {index}: {e}"))
    }
}
