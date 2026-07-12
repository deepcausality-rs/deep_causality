/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    apply_gate_kernel, born_probability_kernel, commutator_kernel, expectation_value_kernel,
    fidelity_kernel, haruna_cz_gate_kernel, haruna_hadamard_gate_kernel, haruna_s_gate_kernel,
    haruna_t_gate_kernel, haruna_x_gate_kernel, haruna_z_gate_kernel,
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

// Helper to create a real-valued multivector field
fn create_real_field() -> CausalMultiVector<f64> {
    CausalMultiVector::new(
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap()
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
fn test_haruna_s_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_s_gate_kernel(&field);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_z_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_z_gate_kernel(&field);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_x_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_x_gate_kernel(&field);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_hadamard_gate_kernel_valid() {
    let field_a = create_real_field();
    let field_b = create_real_field();
    let result = haruna_hadamard_gate_kernel(&field_a, &field_b);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_hadamard_gate_kernel_dimension_error() {
    let field_a = create_real_field();
    let field_wrong = CausalMultiVector::new(vec![1.0, 0.0], Metric::Euclidean(1)).unwrap();
    let result = haruna_hadamard_gate_kernel(&field_a, &field_wrong);
    assert!(result.is_err());
}

#[test]
fn test_haruna_cz_gate_kernel_valid() {
    let field_a1 = create_real_field();
    let field_a2 = create_real_field();
    let result = haruna_cz_gate_kernel(&field_a1, &field_a2);
    assert!(result.is_ok());
}

#[test]
fn test_haruna_gate_kernels_error_on_overflowing_field() {
    // An overflowing field has no finite logical gate; the kernels now surface a
    // QuantumError instead of silently masking the failure as the identity gate.
    let mut data = vec![0.0; 8];
    data[1] = 1e8;
    let field = CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap();
    assert!(haruna_s_gate_kernel(&field).is_err());
    assert!(haruna_z_gate_kernel(&field).is_err());
    assert!(haruna_x_gate_kernel(&field).is_err());
    assert!(haruna_t_gate_kernel(&field).is_err());
}

#[test]
fn test_haruna_t_gate_kernel_valid() {
    let field = create_real_field();
    let result = haruna_t_gate_kernel(&field);
    assert!(result.is_ok());
}
