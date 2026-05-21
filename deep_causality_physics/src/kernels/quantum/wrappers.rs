/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::quantum::mechanics;
use crate::{Gate, Operator};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::HilbertState;
use deep_causality_num::{FromPrimitive, RealField};

use crate::Probability;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

/// Causal wrapper for [`mechanics::klein_gordon_kernel`].
pub fn klein_gordon<R>(
    psi_manifold: &SimplicialManifold<R, R>,
    mass: R,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + FromPrimitive + Default + PartialEq + Debug,
{
    match mechanics::klein_gordon_kernel(psi_manifold, mass) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::born_probability_kernel`].
pub fn born_probability<R>(
    state: &HilbertState<R>,
    basis: &HilbertState<R>,
) -> PropagatingEffect<Probability<R>>
where
    R: RealField + FromPrimitive + core::iter::Sum + Debug,
{
    match mechanics::born_probability_kernel(state, basis) {
        Ok(val) => match Probability::<R>::new(val) {
            Ok(p) => PropagatingEffect::pure(p),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::expectation_value_kernel`].
pub fn expectation_value<R>(state: &HilbertState<R>, operator: &Operator<R>) -> PropagatingEffect<R>
where
    R: RealField + core::iter::Sum + Default + Debug,
{
    match mechanics::expectation_value_kernel(state, operator) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::apply_gate_kernel`].
pub fn apply_gate<R>(state: &HilbertState<R>, gate: &Gate<R>) -> PropagatingEffect<HilbertState<R>>
where
    R: RealField + Debug,
{
    match mechanics::apply_gate_kernel(state, gate) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::commutator_kernel`].
pub fn commutator<R>(a: &Operator<R>, b: &Operator<R>) -> PropagatingEffect<HilbertState<R>>
where
    R: RealField + Debug,
{
    match mechanics::commutator_kernel(a, b) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_s_gate_kernel`].
pub fn haruna_s_gate<R>(field: &CausalMultiVector<R>) -> PropagatingEffect<Operator<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::haruna_s_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_z_gate_kernel`].
pub fn haruna_z_gate<R>(field: &CausalMultiVector<R>) -> PropagatingEffect<Operator<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::haruna_z_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_x_gate_kernel`].
pub fn haruna_x_gate<R>(field: &CausalMultiVector<R>) -> PropagatingEffect<Operator<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::haruna_x_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_hadamard_gate_kernel`].
pub fn haruna_hadamard_gate<R>(
    field_a: &CausalMultiVector<R>,
    field_b: &CausalMultiVector<R>,
) -> PropagatingEffect<Operator<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::haruna_hadamard_gate_kernel(field_a, field_b) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_cz_gate_kernel`].
pub fn haruna_cz_gate<R>(
    field_a1: &CausalMultiVector<R>,
    field_a2: &CausalMultiVector<R>,
) -> PropagatingEffect<Operator<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::haruna_cz_gate_kernel(field_a1, field_a2) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_t_gate_kernel`].
pub fn haruna_t_gate<R>(field: &CausalMultiVector<R>) -> PropagatingEffect<Operator<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match mechanics::haruna_t_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::fidelity_kernel`].
pub fn fidelity<R>(
    ideal: &HilbertState<R>,
    actual: &HilbertState<R>,
) -> PropagatingEffect<Probability<R>>
where
    R: RealField + FromPrimitive + core::iter::Sum + Debug,
{
    match mechanics::fidelity_kernel(ideal, actual) {
        Ok(val) => match Probability::<R>::new(val) {
            Ok(p) => PropagatingEffect::pure(p),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
