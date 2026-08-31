/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    QuantumError, apply_gate_kernel, born_probability_kernel, commutator_kernel,
    expectation_value_kernel, fidelity_kernel,
};

// Helper to create a normalized quantum state
fn create_test_state() -> HilbertState<f64> {
    let data = vec![
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let mv = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    HilbertState::<f64>::from_multivector(mv)
}

// =============================================================================
// Born Probability Kernel Tests
// =============================================================================

#[test]
fn test_born_probability_kernel_normalized() {
    let state = create_test_state();
    let basis = create_test_state();

    let result = born_probability_kernel(&state, &basis);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!(
        (0.0..=1.0).contains(&p),
        "Probability must be in [0,1], got {}",
        p
    );
    // Identical states ⇒ P = |⟨ψ|ψ⟩|² = 1 (pins the canonical value, not just the range).
    assert!(
        (p - 1.0).abs() < 1e-9,
        "identical states ⇒ P = 1, got {}",
        p
    );
}

#[test]
fn test_born_probability_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let basis_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = born_probability_kernel(&state, &basis_wrong);
    assert!(result.is_err());
}

#[test]
fn test_born_probability_kernel_orthogonal() {
    let state1 = create_test_state(); // |0>

    // Create orthogonal state |1> (e1 component)
    let data2 = vec![
        Complex::new(0.0, 0.0),
        Complex::new(1.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
        Complex::new(0.0, 0.0),
    ];
    let mv2 = CausalMultiVector::new(data2, Metric::Euclidean(3)).unwrap();
    let state2 = HilbertState::<f64>::from_multivector(mv2);

    let result = born_probability_kernel(&state1, &state2);
    assert!(result.is_ok());

    let p = result.unwrap();
    assert!(
        p < 0.01,
        "Orthogonal states should have ~0 overlap, got {}",
        p
    );
}

#[test]
fn test_born_probability_kernel_nonfinite() {
    // A state with enormous amplitudes makes |<basis|state>|^2 overflow to
    // +inf, tripping the "Born probability is not finite" guard.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let basis = HilbertState::<f64>::from_multivector(mv);

    let result = born_probability_kernel(&state, &basis);
    assert!(result.is_err());
}

// =============================================================================
// Expectation Value Kernel Tests
// =============================================================================

#[test]
fn test_expectation_value_kernel_valid() {
    let state = create_test_state();
    let operator = create_test_state(); // Use state as simple operator

    let result = expectation_value_kernel(&state, &operator);
    assert!(result.is_ok());
}

#[test]
fn test_expectation_value_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let operator_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = expectation_value_kernel(&state, &operator_wrong);
    assert!(result.is_err());
}

// =============================================================================
// Apply Gate Kernel Tests
// =============================================================================

#[test]
fn test_apply_gate_kernel_identity() {
    let state = create_test_state();
    let gate = create_test_state(); // Identity-like operation

    let result = apply_gate_kernel(&state, &gate);
    assert!(result.is_ok());
}

#[test]
fn test_apply_gate_kernel_dimension_error() {
    let state = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let gate_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = apply_gate_kernel(&state, &gate_wrong);
    assert!(result.is_err());
}

#[test]
fn test_apply_gate_kernel_nonfinite() {
    // A gate and state with enormous amplitudes make the geometric product
    // overflow to non-finite components, tripping the "Non-finite component in
    // state after gate application" guard.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let gate = HilbertState::<f64>::from_multivector(mv);

    let result = apply_gate_kernel(&state, &gate);
    assert!(result.is_err());
}

// =============================================================================
// Commutator Kernel Tests
// =============================================================================

#[test]
fn test_commutator_kernel_valid() {
    let op_a = create_test_state();
    let op_b = create_test_state();

    let result = commutator_kernel(&op_a, &op_b);
    assert!(result.is_ok());
}

#[test]
fn test_commutator_kernel_dimension_error() {
    let op_a = create_test_state();
    let data_wrong = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)];
    let mv_wrong = CausalMultiVector::new(data_wrong, Metric::Euclidean(1)).unwrap();
    let op_wrong = HilbertState::<f64>::from_multivector(mv_wrong);

    let result = commutator_kernel(&op_a, &op_wrong);
    assert!(result.is_err());
}

#[test]
fn test_commutator_kernel_self_is_zero() {
    let op_a = create_test_state();

    // [A, A] = 0
    let result = commutator_kernel(&op_a, &op_a).unwrap();

    // [A, A] = 0 identically — check ALL components, not just the scalar part,
    // so a non-scalar leak cannot hide.
    assert!(
        result.mv().data().iter().all(|c| c.norm() < 1e-10),
        "Commutator [A,A] must vanish across all components"
    );
}

// =============================================================================
// Fidelity Kernel Tests
// =============================================================================

#[test]
fn test_fidelity_kernel_identical_states() {
    let ideal = create_test_state();
    let actual = create_test_state();

    let result = fidelity_kernel(&ideal, &actual);
    assert!(result.is_ok());

    let f = result.unwrap();
    assert!(
        (0.0..=1.0).contains(&f),
        "Fidelity must be in [0,1], got {}",
        f
    );
    // Identical states ⇒ F = 1.
    assert!(
        (f - 1.0).abs() < 1e-9,
        "identical states ⇒ F = 1, got {}",
        f
    );
}

// =============================================================================
// Haruna Gate Kernel Tests
// =============================================================================

#[test]
fn test_expectation_value_kernel_nonfinite() {
    // Amplitudes at f64::MAX overflow the geometric products, so the scalar
    // part of adj(psi)·A·psi is not finite. The kernel reports that instead of
    // handing back an infinity.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let state = HilbertState::<f64>::from_multivector(mv.clone());
    let operator = HilbertState::<f64>::from_multivector(mv);

    let err = expectation_value_kernel(&state, &operator).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonFiniteValue("expectation value is not finite".into())
    );
}

#[test]
fn test_expectation_value_kernel_rejects_non_hermitian_operator() {
    // A = i·1 is anti-Hermitian, and <psi|A|psi> = i for the unit scalar state.
    // The kernel refuses rather than returning the real projection 0, which
    // would be a different observable.
    let state = create_test_state();
    let mut op_data = vec![Complex::new(0.0, 0.0); 8];
    op_data[0] = Complex::new(0.0, 1.0);
    let operator = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(op_data, Metric::Euclidean(3)).unwrap(),
    );

    let err = expectation_value_kernel(&state, &operator).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonPositiveOperator(
            "expectation value has a non-negligible imaginary part; operator is not Hermitian"
                .into()
        )
    );
}

#[test]
fn test_commutator_kernel_nonfinite() {
    // Amplitudes at f64::MAX overflow both AB and BA, so AB - BA carries
    // non-finite components and the kernel reports that rather than returning
    // a state full of NaN.
    let huge = vec![Complex::new(f64::MAX, 0.0); 8];
    let mv = CausalMultiVector::new(huge, Metric::Euclidean(3)).unwrap();
    let a = HilbertState::<f64>::from_multivector(mv.clone());
    let b = HilbertState::<f64>::from_multivector(mv);

    let err = commutator_kernel(&a, &b).unwrap_err();
    assert_eq!(
        err,
        QuantumError::NonFiniteValue("Non-finite component in commutator result".into())
    );
}
