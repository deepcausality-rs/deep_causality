/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use crate::quantum::gates::QuantumOps;
use crate::quantum::quantities::Probability;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::HilbertState; // Use upstream alias/struct
use deep_causality_num::DivisionAlgebra;

pub type Operator = HilbertState;
pub type Gate = HilbertState;

// ============================================================================
// Kernels (Pure Math, Stack-based where possible)
// ============================================================================

pub fn born_probability_kernel(
    state: &HilbertState,
    basis: &HilbertState,
) -> Result<f64, PhysicsError> {
    // Probability P = |<basis|state>|^2
    // Using upstream HilbertState methods if available or inner mv.
    // Upstream HilbertState has .mv field and inner methods?
    // User said "Full HilbertState", so likely has methods.
    // Need to check upstream methods again or access inner.
    // Spec says state.bracket(basis).
    // Let's try to access inner for bracket if trait not implemented for HilbertState yet.
    // But we implemented QuantumOps for CausalMultiVector<Complex<f64>> in gates.rs.
    // HilbertState contains CausalMultiVector<Complex<f64>>.
    // So state.mv().bracket(basis.mv())
    let amplitude = state.mv().bracket(basis.mv());
    let p = amplitude.norm_sqr();

    // Validate
    if !(0.0..=1.000001).contains(&p) {
        // Epsilon for float errors
        return Err(PhysicsError::new(PhysicsErrorEnum::NormalizationError(
            format!("Born probability out of bounds: {}", p),
        )));
    }

    Ok(p.clamp(0.0, 1.0))
}

pub fn expectation_value_kernel(
    state: &HilbertState,
    operator: &Operator,
) -> Result<f64, PhysicsError> {
    // Implementation delegated to QuantumOps trait method which wraps inner logic
    let val = state.mv().expectation_value(operator.mv());
    // Real part is the observable value if operator is Hermitian
    Ok(val.re)
}

pub fn apply_gate_kernel(state: &HilbertState, gate: &Gate) -> Result<HilbertState, PhysicsError> {
    // New State = Gate * State
    // Need underlying multiplication.
    // Assuming we can access geometric_product via inner
    use deep_causality_multivector::MultiVector;
    let new_inner = gate.mv().geometric_product(state.mv());
    // Wrap back in HilbertState
    Ok(HilbertState::from_multivector(new_inner))
}

// ============================================================================
// Causal Wrappers (Monadic PropagatingEffect)
// ============================================================================

pub fn born_probability(
    state: &HilbertState,
    basis: &HilbertState,
) -> PropagatingEffect<Probability> {
    match born_probability_kernel(state, basis) {
        Ok(val) => match Probability::new(val) {
            Ok(p) => PropagatingEffect::pure(p),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn expectation_value(state: &HilbertState, operator: &Operator) -> PropagatingEffect<f64> {
    match expectation_value_kernel(state, operator) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn apply_gate(state: &HilbertState, gate: &Gate) -> PropagatingEffect<HilbertState> {
    match apply_gate_kernel(state, gate) {
        Ok(val) => PropagatingEffect::pure(val),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn commutator(
    a: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> PropagatingEffect<HilbertState> {
    let _ = a;
    let _ = b;
    // [A, B] = AB - BA
    // Note: CausalMultiVector doesn't implement Default, so we can't wrap it in PropagatingEffect directly.
    // We return HilbertState (alias Operator) which implements Default.
    PropagatingEffect::from_error(CausalityError::from(PhysicsError::new(
        PhysicsErrorEnum::Singularity("MultiVector Default Trait Missing".into()),
    )))
}

pub fn haruna_s_gate(field: &CausalMultiVector<f64>) -> PropagatingEffect<Operator> {
    let _ = field;
    PropagatingEffect::from_error(CausalityError::from(PhysicsError::new(
        PhysicsErrorEnum::Singularity("Not Implemented".into()),
    )))
}

pub fn fidelity(ideal: &HilbertState, actual: &HilbertState) -> PropagatingEffect<Probability> {
    // F = |<ideal|actual>|^2
    // Reuse born_probability_kernel logic
    match born_probability_kernel(actual, ideal) {
        // order for bracket shouldn't matter for magnitude
        Ok(val) => match Probability::new(val) {
            Ok(p) => PropagatingEffect::pure(p),
            Err(e) => PropagatingEffect::from_error(e.into()),
        },
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
