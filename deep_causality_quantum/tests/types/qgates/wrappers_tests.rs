/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num_complex::Complex;
use deep_causality_quantum::{
    apply_gate, born_probability, commutator, expectation_value, fidelity, haruna_cz_gate,
    haruna_hadamard_gate, haruna_s_gate, haruna_t_gate, haruna_x_gate, haruna_z_gate,
};

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

fn create_real_field() -> CausalMultiVector<f64> {
    CausalMultiVector::new(
        vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap()
}

#[test]
fn test_born_probability_wrapper_success() {
    let state = create_test_state();
    let basis = create_test_state();

    let effect = born_probability(&state, &basis);
    assert!(effect.is_ok());
}

#[test]
fn test_born_probability_wrapper_error() {
    // An unnormalized state with large magnitude drives |<psi|psi>|^2 far
    // above 1, tripping the kernel's NormalizationError guard.
    let data = vec![Complex::new(100.0, 0.0); 8];
    let state = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap(),
    );
    let basis = state.clone();

    let effect = born_probability(&state, &basis);
    assert!(effect.is_err());
}

#[test]
fn test_expectation_value_wrapper_success() {
    let state = create_test_state();
    let operator = create_test_state();

    let effect = expectation_value(&state, &operator);
    assert!(effect.is_ok());
}

#[test]
fn test_expectation_value_wrapper_error() {
    let state = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let operator = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap(),
    );

    // Metric mismatch should trigger error
    let effect = expectation_value(&state, &operator);
    assert!(effect.is_err());
}

#[test]
fn test_apply_gate_wrapper_success() {
    let state = create_test_state();
    let gate = create_test_state();

    let effect = apply_gate(&state, &gate);
    assert!(effect.is_ok());
}

#[test]
fn test_apply_gate_wrapper_error() {
    let state = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let gate = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap(),
    );

    // Metric mismatch
    let effect = apply_gate(&state, &gate);
    assert!(effect.is_err());
}

#[test]
fn test_commutator_wrapper_success() {
    let a = create_test_state();
    let b = create_test_state();

    let effect = commutator(&a, &b);
    assert!(effect.is_ok());
}

#[test]
fn test_commutator_wrapper_error() {
    let a = create_test_state();
    let data = vec![Complex::new(0.0, 0.0); 4]; // 2D metric
    let b = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(2)).unwrap(),
    );

    // Metric mismatch
    let effect = commutator(&a, &b);
    assert!(effect.is_err());
}

#[test]
fn test_fidelity_wrapper_success() {
    let ideal = create_test_state();
    let actual = create_test_state();

    let effect = fidelity(&ideal, &actual);
    assert!(effect.is_ok());
}

#[test]
fn test_fidelity_wrapper_error() {
    // Fidelity uses born_probability internally.
    // Trigger normalization error with large magnitude state.
    let data = vec![Complex::new(100.0, 0.0); 8];
    let ideal = HilbertState::<f64>::from_multivector(
        CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap(),
    );
    let actual = ideal.clone();

    let effect = fidelity(&ideal, &actual);
    assert!(effect.is_err());
}

#[test]
fn test_haruna_s_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_s_gate(&field);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_z_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_z_gate(&field);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_x_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_x_gate(&field);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_hadamard_gate_wrapper_success() {
    let field_a = create_real_field();
    let field_b = create_real_field();
    let effect = haruna_hadamard_gate(&field_a, &field_b);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_hadamard_gate_wrapper_error() {
    let field_a = create_real_field();
    let field_b = CausalMultiVector::new(vec![1.0; 4], Metric::Euclidean(2)).unwrap(); // Mismatch

    let effect = haruna_hadamard_gate(&field_a, &field_b);
    assert!(effect.is_err());
}

#[test]
fn test_haruna_cz_gate_wrapper_success() {
    let field_a1 = create_real_field();
    let field_a2 = create_real_field();
    let effect = haruna_cz_gate(&field_a1, &field_a2);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_cz_gate_wrapper_error() {
    let field_a1 = create_real_field();
    let field_a2 = CausalMultiVector::new(vec![1.0; 4], Metric::Euclidean(2)).unwrap(); // Mismatch

    let effect = haruna_cz_gate(&field_a1, &field_a2);
    assert!(effect.is_err());
}

#[test]
fn test_haruna_t_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_t_gate(&field);
    assert!(effect.is_ok());
}

/// A real field whose logical-gate exponent overflows the 1e6 norm bound, so the
/// wrapped kernel returns an error — exercising the wrapper error arms that used
/// to be unreachable when the logical gates masked overflow as the identity.
fn overflowing_field() -> CausalMultiVector<f64> {
    let mut data = vec![0.0; 8];
    data[1] = 1e8;
    CausalMultiVector::new(data, Metric::Euclidean(3)).unwrap()
}

#[test]
fn test_haruna_s_gate_wrapper_error() {
    assert!(haruna_s_gate(&overflowing_field()).is_err());
}

#[test]
fn test_haruna_z_gate_wrapper_error() {
    assert!(haruna_z_gate(&overflowing_field()).is_err());
}

#[test]
fn test_haruna_x_gate_wrapper_error() {
    assert!(haruna_x_gate(&overflowing_field()).is_err());
}

#[test]
fn test_haruna_t_gate_wrapper_error() {
    assert!(haruna_t_gate(&overflowing_field()).is_err());
}
