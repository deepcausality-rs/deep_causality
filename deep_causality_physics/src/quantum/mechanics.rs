/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use crate::quantum::gates::QuantumOps;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::HilbertState; // Use upstream alias/struct
use deep_causality_num::DivisionAlgebra;

pub type Operator = HilbertState;
pub type Gate = HilbertState;

// ============================================================================
// Kernels (Pure Math, Stack-based where possible)
// ============================================================================

/// Calculates the Born probability: $P = |\langle \text{basis} | \text{state} \rangle|^2$.
///
/// # Arguments
/// * `state` - Quantum state vector $|\psi\rangle$.
/// * `basis` - Basis vector/state $|\phi\rangle$.
///
/// # Returns
/// * `Ok(f64)` - Probability $P$.
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

/// Calculates the expectation value: $\langle A \rangle = \langle \psi | A | \psi \rangle$.
///
/// # Arguments
/// * `state` - Quantum state $|\psi\rangle$.
/// * `operator` - Observable operator $A$.
///
/// # Returns
/// * `Ok(f64)` - Expectation value (Real part of complex result).
pub fn expectation_value_kernel(
    state: &HilbertState,
    operator: &Operator,
) -> Result<f64, PhysicsError> {
    // Implementation delegated to QuantumOps trait method which wraps inner logic
    let val = state.mv().expectation_value(operator.mv());
    // Real part is the observable value if operator is Hermitian
    Ok(val.re)
}

/// Applies a quantum gate to a state: $|\psi'\rangle = U |\psi\rangle$.
///
/// # Arguments
/// * `state` - Initial state $|\psi\rangle$.
/// * `gate` - Quantum gate/operator $U$.
///
/// # Returns
/// * `Ok(HilbertState)` - New state $|\psi'\rangle$.
pub fn apply_gate_kernel(state: &HilbertState, gate: &Gate) -> Result<HilbertState, PhysicsError> {
    // New State = Gate * State
    // Need underlying multiplication.
    // Assuming we can access geometric_product via inner
    use deep_causality_multivector::MultiVector;
    let new_inner = gate.mv().geometric_product(state.mv());
    // Wrap back in HilbertState
    Ok(HilbertState::from_multivector(new_inner))
}

// Additional Kernels

/// Stub: Calculates commutator $[A, B] = AB - BA$.
///
/// # Arguments
/// * `a` - Operator $A$.
/// * `b` - Operator $B$.
///
/// # Returns
/// * `Result<HilbertState, PhysicsError>` - Commutator result.
pub fn commutator_kernel(
    a: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
) -> Result<HilbertState, PhysicsError> {
    let _ = a;
    let _ = b;
    // [A, B] = AB - BA
    // Result wrapped in HilbertState
    Err(PhysicsError::new(PhysicsErrorEnum::Singularity(
        "MultiVector Default Trait Missing".into(),
    )))
}

/// Stub: Haruna's Gate (Theoretical).
pub fn haruna_s_gate_kernel(field: &CausalMultiVector<f64>) -> Result<Operator, PhysicsError> {
    let _ = field;
    Err(PhysicsError::new(PhysicsErrorEnum::Singularity(
        "Not Implemented".into(),
    )))
}

/// Calculates Quantum Fidelity: $F = |\langle \psi_{\text{ideal}} | \psi_{\text{actual}} \rangle|^2$.
///
/// # Arguments
/// * `ideal` - Ideal/Target state.
/// * `actual` - Actual/Noisy state.
///
/// # Returns
/// * `Result<f64, PhysicsError>` - Fidelity $F$.
pub fn fidelity_kernel(ideal: &HilbertState, actual: &HilbertState) -> Result<f64, PhysicsError> {
    // F = |<ideal|actual>|^2
    // Reuse born_probability_kernel logic
    born_probability_kernel(actual, ideal)
}
