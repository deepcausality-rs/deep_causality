/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, HilbertState, Metric};
use deep_causality_num::Complex;
use deep_causality_physics::{
    apply_gate, born_probability, commutator, expectation_value, fidelity, haruna_cz_gate,
    haruna_hadamard_gate, haruna_s_gate, haruna_t_gate, haruna_x_gate, haruna_z_gate,
};

fn create_test_state() -> HilbertState {
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
    HilbertState::from_multivector(mv)
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
fn test_expectation_value_wrapper_success() {
    let state = create_test_state();
    let operator = create_test_state();

    let effect = expectation_value(&state, &operator);
    assert!(effect.is_ok());
}

#[test]
fn test_apply_gate_wrapper_success() {
    let state = create_test_state();
    let gate = create_test_state();

    let effect = apply_gate(&state, &gate);
    assert!(effect.is_ok());
}

#[test]
fn test_commutator_wrapper_success() {
    let a = create_test_state();
    let b = create_test_state();

    let effect = commutator(&a, &b);
    assert!(effect.is_ok());
}

#[test]
fn test_fidelity_wrapper_success() {
    let ideal = create_test_state();
    let actual = create_test_state();

    let effect = fidelity(&ideal, &actual);
    assert!(effect.is_ok());
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
fn test_haruna_cz_gate_wrapper_success() {
    let field_a1 = create_real_field();
    let field_a2 = create_real_field();
    let effect = haruna_cz_gate(&field_a1, &field_a2);
    assert!(effect.is_ok());
}

#[test]
fn test_haruna_t_gate_wrapper_success() {
    let field = create_real_field();
    let effect = haruna_t_gate(&field);
    assert!(effect.is_ok());
}
