/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{HilbertState, Metric, QuantumGates, QuantumOps};
use deep_causality_num::{Complex64, ComplexNumber, One, Zero};

const DIM: usize = 10; // For Cl(0,10)
const SIZE: usize = 1 << DIM; // 1024
const EPSILON: f64 = 1e-9;

// Helper to create a HilbertState for Cl(0,10)
fn create_cl0_10_hilbert_state(data: Vec<Complex64>) -> HilbertState {
    HilbertState::new_spin10(data).unwrap()
}

// Helper to check if two complex numbers are approximately equal
fn assert_complex_approx_eq(c1: Complex64, c2: Complex64, epsilon: f64) {
    assert!(
        (c1.re - c2.re).abs() < epsilon,
        "Real parts differ: {} vs {}",
        c1.re,
        c2.re
    );
    assert!(
        (c1.im - c2.im).abs() < epsilon,
        "Imaginary parts differ: {} vs {}",
        c1.im,
        c2.im
    );
}

// Helper to check if two complex vectors are approximately equal
fn assert_complex_vec_approx_eq(v1: &[Complex64], v2: &[Complex64], epsilon: f64) {
    assert_eq!(v1.len(), v2.len());
    for (c1, c2) in v1.iter().zip(v2.iter()) {
        assert_complex_approx_eq(*c1, *c2, epsilon);
    }
}

// --- dag() tests ---

#[test]
fn test_dag_scalar_state() {
    let scalar_val = Complex64::new(3.0, 4.0);
    let mut data = vec![Complex64::zero(); SIZE];
    data[0] = scalar_val;
    let state = create_cl0_10_hilbert_state(data);

    let dag_state = state.dag();
    let mut expected_data = vec![Complex64::zero(); SIZE];
    expected_data[0] = scalar_val.conj(); // Scalar part is just complex conjugated

    assert_complex_vec_approx_eq(dag_state.mv().data(), &expected_data, EPSILON);
    assert_eq!(dag_state.mv().metric(), Metric::NonEuclidean(DIM));
}

#[test]
fn test_dag_vector_state() {
    // State: i*e1 (e1 is at index 1)
    let mut data = vec![Complex64::zero(); SIZE];
    data[1] = Complex64::new(0.0, 1.0);
    let state = create_cl0_10_hilbert_state(data);

    let dag_state = state.dag();
    let mut expected_data = vec![Complex64::zero(); SIZE];
    expected_data[1] = Complex64::new(0.0, -1.0); // (i*e1).dag() = -i*e1 (reversion sign for grade 1 is +1, complex conjugate is -i)

    assert_complex_vec_approx_eq(dag_state.mv().data(), &expected_data, EPSILON);
}

#[test]
fn test_dag_bivector_state() {
    // State: e12 (e12 is at index 3)
    let mut data = vec![Complex64::zero(); SIZE];
    data[3] = Complex64::new(1.0, 0.0);
    let state = create_cl0_10_hilbert_state(data);

    let dag_state = state.dag();
    let mut expected_data = vec![Complex64::zero(); SIZE];
    expected_data[3] = Complex64::new(-1.0, 0.0); // (e12).dag() = -e12 (reversion sign for grade 2 is -1)

    assert_complex_vec_approx_eq(dag_state.mv().data(), &expected_data, EPSILON);
}

#[test]
fn test_dag_purely_imaginary_bivector_state() {
    // State: i*e12
    let mut data = vec![Complex64::zero(); SIZE];
    data[3] = Complex64::new(0.0, 1.0);
    let state = create_cl0_10_hilbert_state(data);

    let dag_state = state.dag();
    let mut expected_data = vec![Complex64::zero(); SIZE];
    expected_data[3] = Complex64::new(0.0, 1.0); // (i*e12).dag() = (-i)*(-e12) = i*e12

    assert_complex_vec_approx_eq(dag_state.mv().data(), &expected_data, EPSILON);
}

#[test]
fn test_dag_dag_equals_original() {
    let mut data = vec![Complex64::zero(); SIZE];
    data[0] = Complex64::new(1.0, 2.0);
    data[1] = Complex64::new(3.0, 4.0);
    data[3] = Complex64::new(5.0, 6.0);
    let state = create_cl0_10_hilbert_state(data.clone());

    let dag_dag_state = state.dag().dag();
    assert_complex_vec_approx_eq(dag_dag_state.mv().data(), &data, EPSILON);
}

// --- bracket() tests ---

#[test]
fn test_bracket_orthogonal_states() {
    // For Cl(0,10), identity (scalar 1) and e1 are orthogonal.
    let identity_state = HilbertState::gate_identity();
    let mut e1_data = vec![Complex64::zero(); SIZE];
    e1_data[1] = Complex64::one(); // e1 basis vector
    let e1_state = create_cl0_10_hilbert_state(e1_data);

    let result = identity_state.bracket(&e1_state);
    assert_complex_approx_eq(result, Complex64::zero(), EPSILON);
}

#[test]
fn test_bracket_same_scalar_state() {
    // State |psi> = 2*I
    let mut data = vec![Complex64::zero(); SIZE];
    data[0] = Complex64::new(2.0, 0.0);
    let state = create_cl0_10_hilbert_state(data);

    let result = state.bracket(&state); // <psi|psi> = (2*I).dag() * (2*I) = 2*I * 2*I = 4*I^2 = 4
    assert_complex_approx_eq(result, Complex64::new(4.0, 0.0), EPSILON);
}

#[test]
fn test_bracket_conjugated_scalar_state() {
    // State |psi> = (1 + i)*I
    let mut data1 = vec![Complex64::zero(); SIZE];
    data1[0] = Complex64::new(1.0, 1.0);
    let state1 = create_cl0_10_hilbert_state(data1);

    // State |phi> = (1 - i)*I
    let mut data2 = vec![Complex64::zero(); SIZE];
    data2[0] = Complex64::new(1.0, -1.0);
    let state2 = create_cl0_10_hilbert_state(data2);

    // <psi|phi> = ((1+i)I).dag() * ((1-i)I) = (1-i)I * (1-i)I = (1-i)^2 * I = (1 - 2i + i^2)I = -2i*I
    // Scalar part is -2i
    let result = state1.bracket(&state2);
    assert_complex_approx_eq(result, Complex64::new(0.0, -2.0), EPSILON);
}

#[test]
fn test_bracket_normalized_identity_state() {
    // Identity gate is inherently normalized as a state if it represents |0> or vacuum
    let identity_state = HilbertState::gate_identity();
    let result = identity_state.bracket(&identity_state);
    assert_complex_approx_eq(result, Complex64::new(1.0, 0.0), EPSILON);
}

// --- expectation_value() tests ---

#[test]
fn test_expectation_value_identity_on_identity_state() {
    // <I|I|I> = ScalarPart(I.dag() * I * I) = ScalarPart(I) = 1
    let state = HilbertState::gate_identity();
    let operator = HilbertState::gate_identity();
    let result = state.expectation_value(&operator);
    assert_complex_approx_eq(result, Complex64::new(1.0, 0.0), EPSILON);
}

#[test]
fn test_expectation_value_z_on_identity_state() {
    // <I|Z|I> = ScalarPart(I.dag() * Z * I) = ScalarPart(Z) = 0 (since Z is pure bivector)
    let state = HilbertState::gate_identity();
    let operator = HilbertState::gate_z();
    let result = state.expectation_value(&operator);
    assert_complex_approx_eq(result, Complex64::zero(), EPSILON);
}

#[test]
fn test_expectation_value_x_on_x_state() {
    // For a state X, Expectation value of I is <X|I|X> = -1
    let state = HilbertState::gate_x(); // State is X (i*e1)
    let operator = HilbertState::gate_identity(); // Operator is I (scalar 1)
    let result = state.expectation_value(&operator);
    assert_complex_approx_eq(result, Complex64::new(-1.0, 0.0), EPSILON);
}

// --- normalize() tests ---

#[test]
fn test_normalize_unnormalized_scalar_state() {
    // State: 2*I
    let mut data = vec![Complex64::zero(); SIZE];
    data[0] = Complex64::new(2.0, 0.0);
    let state = create_cl0_10_hilbert_state(data);

    let normalized_state = state.normalize();

    let mut expected_data = vec![Complex64::zero(); SIZE];
    expected_data[0] = Complex64::one(); // Should normalize to I

    assert_complex_vec_approx_eq(normalized_state.mv().data(), &expected_data, EPSILON);
    assert_complex_approx_eq(
        normalized_state.bracket(&normalized_state),
        Complex64::new(1.0, 0.0),
        EPSILON,
    );
}

#[test]
fn test_normalize_already_normalized_state() {
    // State: I
    let state = HilbertState::gate_identity();
    let normalized_state = state.normalize();

    let mut expected_data = vec![Complex64::zero(); SIZE];
    expected_data[0] = Complex64::one(); // Should remain I

    assert_complex_vec_approx_eq(normalized_state.mv().data(), &expected_data, EPSILON);
    assert_complex_approx_eq(
        normalized_state.bracket(&normalized_state),
        Complex64::new(1.0, 0.0),
        EPSILON,
    );
}

#[test]
fn test_normalize_zero_vector() {
    // State: All zeros
    let data = vec![Complex64::zero(); SIZE];
    let state = create_cl0_10_hilbert_state(data.clone());
    let normalized_state = state.normalize();

    // Should return the original zero vector (unmodified)
    assert_complex_vec_approx_eq(normalized_state.mv().data(), &data, EPSILON);
    assert_complex_approx_eq(
        normalized_state.bracket(&normalized_state),
        Complex64::new(0.0, 0.0),
        EPSILON,
    );
}
