/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::quantum::mechanics;
use crate::{Gate, Operator};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::HilbertState;

use crate::Probability;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

/// Causal wrapper for [`mechanics::klein_gordon_kernel`].
pub fn klein_gordon(
    psi_manifold: &Manifold<f64>,
    mass: f64,
) -> PropagatingEffect<CausalTensor<f64>> {
    match mechanics::klein_gordon_kernel(psi_manifold, mass) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::born_probability_kernel`].
pub fn born_probability(
    state: &HilbertState,
    basis: &HilbertState,
) -> PropagatingEffect<Probability> {
    match mechanics::born_probability_kernel(state, basis) {
        Ok(val) => match Probability::new(val) {
            Ok(p) => PropagatingEffect::pure(p),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::expectation_value_kernel`].
pub fn expectation_value(state: &HilbertState, operator: &Operator) -> PropagatingEffect<f64> {
    match mechanics::expectation_value_kernel(state, operator) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::apply_gate_kernel`].
pub fn apply_gate(state: &HilbertState, gate: &Gate) -> PropagatingEffect<HilbertState> {
    match mechanics::apply_gate_kernel(state, gate) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::commutator_kernel`].
pub fn commutator(a: &Operator, b: &Operator) -> PropagatingEffect<HilbertState> {
    match mechanics::commutator_kernel(a, b) {
        Ok(res) => PropagatingEffect::pure(res),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_s_gate_kernel`].
pub fn haruna_s_gate(field: &CausalMultiVector<f64>) -> PropagatingEffect<Operator> {
    match mechanics::haruna_s_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_z_gate_kernel`].
pub fn haruna_z_gate(field: &CausalMultiVector<f64>) -> PropagatingEffect<Operator> {
    match mechanics::haruna_z_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_x_gate_kernel`].
pub fn haruna_x_gate(field: &CausalMultiVector<f64>) -> PropagatingEffect<Operator> {
    match mechanics::haruna_x_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_hadamard_gate_kernel`].
pub fn haruna_hadamard_gate(
    field_a: &CausalMultiVector<f64>,
    field_b: &CausalMultiVector<f64>,
) -> PropagatingEffect<Operator> {
    match mechanics::haruna_hadamard_gate_kernel(field_a, field_b) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_cz_gate_kernel`].
pub fn haruna_cz_gate(
    field_a1: &CausalMultiVector<f64>,
    field_a2: &CausalMultiVector<f64>,
) -> PropagatingEffect<Operator> {
    match mechanics::haruna_cz_gate_kernel(field_a1, field_a2) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::haruna_t_gate_kernel`].
pub fn haruna_t_gate(field: &CausalMultiVector<f64>) -> PropagatingEffect<Operator> {
    match mechanics::haruna_t_gate_kernel(field) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`mechanics::fidelity_kernel`].
pub fn fidelity(ideal: &HilbertState, actual: &HilbertState) -> PropagatingEffect<Probability> {
    match mechanics::fidelity_kernel(ideal, actual) {
        Ok(val) => match Probability::new(val) {
            Ok(p) => PropagatingEffect::pure(p),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
