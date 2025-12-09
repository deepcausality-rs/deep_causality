/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::quantum::mechanics;
use crate::quantum::mechanics::{Gate, Operator};
use crate::quantum::quantities::Probability;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::HilbertState; // Use upstream alias/struct

// Wrappers

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
pub fn commutator(
    a: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> PropagatingEffect<HilbertState> {
    match mechanics::commutator_kernel(a, b) {
        Ok(val) => PropagatingEffect::pure(val),
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
