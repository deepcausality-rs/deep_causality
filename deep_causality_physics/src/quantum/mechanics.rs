/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumOps;
use crate::quantum::gates_haruna;
use crate::{PhysicsError, PhysicsErrorEnum};
use deep_causality_haft::Functor;
use deep_causality_multivector::MultiVector;
use deep_causality_multivector::{CausalMultiVector, CausalMultiVectorWitness, HilbertState};
use deep_causality_num::Complex;
use deep_causality_num::DivisionAlgebra;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

pub type Operator = HilbertState;
pub type Gate = HilbertState;

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
    // 2. Term B: m^2 psi
    let m2 = mass * mass;
    let psi_data = psi_manifold.data();

    // The laplacian is on 0-forms (vertices), so we need the 0-form part of psi.
    // We slice psi_data to match the size of the laplacian tensor.
    // Use len() instead of size()
    let vertex_count = laplacian.len();
    if psi_data.len() < vertex_count {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            "psi_data is smaller than laplacian data".to_string(),
        )));
    }
    let psi_vertex_data = &psi_data.as_slice()[..vertex_count];
    let psi_vertex_tensor =
        CausalTensor::new(psi_vertex_data.to_vec(), laplacian.shape().to_vec())?;
    let m2_psi = psi_vertex_tensor * m2;

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
    // Ensure the states live in the same Hilbert space (metric/dimension)
    if state.mv().metric() != basis.mv().metric() {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch in Born Probability: {:?} vs {:?}",
                state.mv().metric(),
                basis.mv().metric()
            ),
        )));
    }

    // Probability P = |<basis|state>|^2
    let amplitude = state.mv().bracket(basis.mv());
    let p = amplitude.norm_sqr();

    // Guard against non-finite values
    if !p.is_finite() {
        return Err(PhysicsError::new(PhysicsErrorEnum::NumericalInstability(
            format!("Born probability is not finite: {}", p),
        )));
    }

    // Validate
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
    // Check metric compatibility
    if state.mv().metric() != operator.mv().metric() {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch in Expectation Value: {:?} vs {:?}",
                state.mv().metric(),
                operator.mv().metric()
            ),
        )));
    }

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
    if state.mv().metric() != gate.mv().metric() {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch in Apply Gate: {:?} vs {:?}",
                state.mv().metric(),
                gate.mv().metric()
            ),
        )));
    }

    let new_inner = gate.mv().geometric_product(state.mv());

    // Validate numerical stability of resulting state
    if new_inner
        .data()
        .iter()
        .any(|c| !c.re.is_finite() || !c.im.is_finite())
    {
        return Err(PhysicsError::new(PhysicsErrorEnum::NumericalInstability(
            "Non-finite component in state after gate application".into(),
        )));
    }

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
    // [A, B] = AB - BA
    // Operators are HilbertStates wrapping CausalMultiVector<Complex<f64>>
    let a_mv = a.mv();
    let b_mv = b.mv();

    if a_mv.metric() != b_mv.metric() {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch in Commutator: {:?} vs {:?}",
                a_mv.metric(),
                b_mv.metric()
            ),
        )));
    }

    let ab = a_mv.geometric_product(b_mv);
    let ba = b_mv.geometric_product(a_mv);

    let commutator = ab - ba;

    Ok(HilbertState::from_multivector(commutator))
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

/// Implements Haruna's Logical S-Gate.
///
/// Uses the Gauge Field Formalism to compute $S(\gamma) = \exp(i \frac{\pi}{2} a(\gamma)^2)$.
pub fn haruna_s_gate_kernel(field: &CausalMultiVector<f64>) -> Result<Operator, PhysicsError> {
    // Convert real field to complex for quantum gate calculation
    let field_complex = CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, 0.0));

    // Compute Logical S Gate
    let result = gates_haruna::logical_s(&field_complex);

    Ok(HilbertState::from_multivector(result))
}

/// Implements Haruna's Logical Z-Gate.
///
/// Uses the Gauge Field Formalism to compute $Z(\gamma) = \exp(i \pi a(\gamma))$.
pub fn haruna_z_gate_kernel(field: &CausalMultiVector<f64>) -> Result<Operator, PhysicsError> {
    let field_complex = CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, 0.0));
    let result = gates_haruna::logical_z(&field_complex);

    Ok(HilbertState::from_multivector(result))
}

/// Implements Haruna's Logical X-Gate.
///
/// Uses the Gauge Field Formalism to compute $X(\tilde{\gamma}) = \exp(i \pi b(\tilde{\gamma}))$.
pub fn haruna_x_gate_kernel(field: &CausalMultiVector<f64>) -> Result<Operator, PhysicsError> {
    let field_complex = CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, 0.0));
    let result = gates_haruna::logical_x(&field_complex);

    Ok(HilbertState::from_multivector(result))
}

/// Implements Haruna's Logical Hadamard Gate.
///
/// $H(\gamma) = e^{-i \frac{\pi}{4}} S(\gamma) \exp(i \frac{\pi}{2} b^2) S(\gamma)$.
pub fn haruna_hadamard_gate_kernel(
    field_a: &CausalMultiVector<f64>,
    field_b: &CausalMultiVector<f64>,
) -> Result<Operator, PhysicsError> {
    let a_complex = CausalMultiVectorWitness::fmap(field_a.clone(), |x| Complex::new(x, 0.0));
    let b_complex = CausalMultiVectorWitness::fmap(field_b.clone(), |x| Complex::new(x, 0.0));

    if a_complex.metric() != b_complex.metric() {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch in Haruna Hadamard: {:?} vs {:?}",
                a_complex.metric(),
                b_complex.metric()
            ),
        )));
    }

    let result = gates_haruna::logical_hadamard(&a_complex, &b_complex);

    Ok(HilbertState::from_multivector(result))
}

/// Implements Haruna's Logical CZ Gate.
///
/// $CZ(\gamma_1, \gamma_2) = \exp(i \pi a(\gamma_1) a(\gamma_2))$.
pub fn haruna_cz_gate_kernel(
    field_a1: &CausalMultiVector<f64>,
    field_a2: &CausalMultiVector<f64>,
) -> Result<Operator, PhysicsError> {
    let a1_complex = CausalMultiVectorWitness::fmap(field_a1.clone(), |x| Complex::new(x, 0.0));
    let a2_complex = CausalMultiVectorWitness::fmap(field_a2.clone(), |x| Complex::new(x, 0.0));

    if a1_complex.metric() != a2_complex.metric() {
        return Err(PhysicsError::new(PhysicsErrorEnum::DimensionMismatch(
            format!(
                "Metric mismatch in Haruna CZ: {:?} vs {:?}",
                a1_complex.metric(),
                a2_complex.metric()
            ),
        )));
    }

    let result = gates_haruna::logical_cz(&a1_complex, &a2_complex);

    Ok(HilbertState::from_multivector(result))
}

/// Implements Haruna's Logical T-Gate.
///
/// $T(\gamma) = \exp(i \pi (\frac{1}{2} a^3 - \frac{3}{4} a^2 + \frac{1}{2} a))$.
pub fn haruna_t_gate_kernel(field: &CausalMultiVector<f64>) -> Result<Operator, PhysicsError> {
    let field_complex = CausalMultiVectorWitness::fmap(field.clone(), |x| Complex::new(x, 0.0));
    let result = gates_haruna::logical_t(&field_complex);

    Ok(HilbertState::from_multivector(result))
}
