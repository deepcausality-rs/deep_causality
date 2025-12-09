/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::error::{PhysicsError, PhysicsErrorEnum};
use crate::quantum::gates::QuantumOps;
use deep_causality_multivector::CausalMultiVector;
use deep_causality_multivector::HilbertState; // Use upstream alias/struct
use deep_causality_multivector::MultiVector;
use deep_causality_num::DivisionAlgebra;

pub type Operator = HilbertState;
pub type Gate = HilbertState;

// ============================================================================
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

// Kernels

/// Calculates the Klein-Gordon operator action: $(\Delta + m^2)\psi$.
///
/// Computes the action of the Klein-Gordon operator on a scalar field $\psi$.
/// The result is the "source" or "force" required to maintain that field configuration,
/// or zero if the field satisfies the free equation.
///
/// # Sign Convention
/// This implementation uses the Euclidean/Riemannian convention where the Laplacian $\Delta$
/// is positive-definite (akin to $k^2$). The equation is $(\Delta + m^2)\psi = 0$.
///
/// If `Manifold` contains a Lorentzian metric, the `laplacian` method (defined via Hodge stars)
/// naturally becomes the d'Alembertian $\square$ (Laplace-Beltrami). In that case, the equation
/// form $(\square + m^2)\psi = 0$ is preserved if the signature (+---) is handled by the metric.
///
/// # Arguments
/// * `psi_manifold` - Manifold containing the scalar field $\psi$ (0-form).
/// * `mass` - Mass $m$.
///
/// # Returns
/// * `Result<CausalTensor<f64>, PhysicsError>` - The result of applying the operator $O(\psi)$.
pub fn klein_gordon_kernel(
    psi_manifold: &Manifold<f64>,
    mass: f64,
) -> Result<CausalTensor<f64>, PhysicsError> {
    // 1. Term A: Laplacian(psi) = Delta psi
    // In deep_causality_topology, laplacian = d delta + delta d.
    // For 0-forms, Delta = delta d.
    let laplacian = psi_manifold.laplacian(0);

    // 2. Term B: m^2 psi
    let m2 = mass * mass;
    let psi_data = psi_manifold.data();
    let m2_psi = psi_data.clone() * m2;

    // 3. Result: Delta psi + m^2 psi
    // We sum them. Ideally, for a free field, this sum is zero.
    let result = laplacian + m2_psi;

    Ok(result)
}

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
    let amplitude = state.mv().bracket(basis.mv());
    let p = amplitude.norm_sqr();

    // Validate
    // Relaxed epsilon for floating point arithmetic in complex scenarios
    if !(-1e-9..=1.000001).contains(&p) {
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
    let new_inner = gate.mv().geometric_product(state.mv());
    // Wrap back in HilbertState
    Ok(HilbertState::from_multivector(new_inner))
}


/// Calculates commutator $[A, B] = AB - BA$.
///
/// # Arguments
/// * `a` - Operator $A$ (HilbertState/QuantumOperator).
/// * `b` - Operator $B$.
///
/// # Returns
/// * `Result<HilbertState, PhysicsError>` - Commutator result.
pub fn commutator_kernel(a: &Operator, b: &Operator) -> Result<HilbertState, PhysicsError> {
    use deep_causality_multivector::MultiVector;

    // [A, B] = AB - BA
    // Operators are HilbertStates wrapping CausalMultiVector<Complex<f64>>
    let a_mv = a.mv();
    let b_mv = b.mv();

    let ab = a_mv.geometric_product(b_mv);
    let ba = b_mv.geometric_product(a_mv);

    let commutator = ab - ba;

    Ok(HilbertState::from_multivector(commutator))
}

/// Stub: Haruna's Gate (Theoretical Non-Linear Operator).
///
/// This is a placeholder for theoretical unified field operators.
pub fn haruna_s_gate_kernel(_field: &CausalMultiVector<f64>) -> Result<Operator, PhysicsError> {
    Err(PhysicsError::new(PhysicsErrorEnum::Singularity(
        "Haruna's Gate not yet implemented (requires Non-Linear Quantum extension)".into(),
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
