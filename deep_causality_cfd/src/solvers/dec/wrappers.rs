/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The causal-monad surface of the solver, in the crate's existing
//! kernel-wrapper tradition (`Ok → pure`, `Err → from_error`).
//!
//! The monad's `pure` requires `Value: Default`, and the
//! [`SolenoidalField`](deep_causality_physics::SolenoidalField) type-state deliberately has
//! no `Default` (a default-constructed "projected" field would be a third
//! construction path around the projection). The wrapper therefore carries
//! the projected **cochain tensor** as the monad payload; re-entry into
//! the type-state happens through the solver step, not through the
//! payload.

use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_tensor::CausalTensor;

use crate::solvers::dec::DecNsScalar;
use crate::solvers::dec::dec_ns_solver::DecNsSolver;
use deep_causality_physics::SolenoidalField;

/// Causal wrapper for [`DecNsSolver::step`]: one projected march step,
/// carrying the divergence-free edge cochain of the new state.
pub fn dec_ns_step<const D: usize, R>(
    solver: &DecNsSolver<'_, D, R>,
    state: &SolenoidalField<R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: DecNsScalar,
{
    match solver.step(state) {
        Ok(output) => PropagatingEffect::pure(output.state().as_one_form().clone()),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
