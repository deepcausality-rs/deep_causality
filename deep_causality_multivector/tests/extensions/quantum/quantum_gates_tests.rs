/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{HilbertState, Metric, QuantumGates};
use deep_causality_num::{Complex64, One, Zero};

const DIM: usize = 10; // For Cl(0,10)
const SIZE: usize = 1 << DIM; // 1024

// Helper to create an expected data vector for a specific gate
fn create_expected_data_vector(index: usize, value: Complex64, size: usize) -> Vec<Complex64> {
    let mut data = vec![Complex64::zero(); size];
    data[index] = value;
    data
}

// Helper to check if two complex vectors are approximately equal
fn assert_complex_vec_approx_eq(v1: &[Complex64], v2: &[Complex64], epsilon: f64) {
    assert_eq!(v1.len(), v2.len());
    for (c1, c2) in v1.iter().zip(v2.iter()) {
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
}

#[test]
fn test_gate_identity() {
    let gate = HilbertState::gate_identity();
    let mut expected_data = vec![Complex64::zero(); SIZE];
    expected_data[0] = Complex64::one(); // Scalar 1

    assert_complex_vec_approx_eq(gate.mv().data(), &expected_data, 1e-9);
    assert_eq!(gate.mv().metric(), Metric::NonEuclidean(DIM));

    // Physical property: I^2 = I
    let gate_squared = gate.clone().into_inner() * gate.clone().into_inner();
    assert_complex_vec_approx_eq(gate_squared.data(), &expected_data, 1e-9);
}

#[test]
fn test_gate_x() {
    let gate = HilbertState::gate_x();
    let i = Complex64::new(0.0, 1.0);
    let expected_data = create_expected_data_vector(1, i, SIZE); // X = i*e1 (e1 is at index 1)

    assert_complex_vec_approx_eq(gate.mv().data(), &expected_data, 1e-9);
    assert_eq!(gate.mv().metric(), Metric::NonEuclidean(DIM));

    // Physical property: X^2 = I
    let gate_squared = gate.clone().into_inner() * gate.clone().into_inner();
    let mut expected_identity_data = vec![Complex64::zero(); SIZE];
    expected_identity_data[0] = Complex64::one();
    assert_complex_vec_approx_eq(gate_squared.data(), &expected_identity_data, 1e-9);
}

#[test]
fn test_gate_y() {
    let gate = HilbertState::gate_y();
    let i = Complex64::new(0.0, 1.0);
    let expected_data = create_expected_data_vector(2, i, SIZE); // Y = i*e2 (e2 is at index 2)

    assert_complex_vec_approx_eq(gate.mv().data(), &expected_data, 1e-9);
    assert_eq!(gate.mv().metric(), Metric::NonEuclidean(DIM));

    // Physical property: Y^2 = I
    let gate_squared = gate.clone().into_inner() * gate.clone().into_inner();
    let mut expected_identity_data = vec![Complex64::zero(); SIZE];
    expected_identity_data[0] = Complex64::one();
    assert_complex_vec_approx_eq(gate_squared.data(), &expected_identity_data, 1e-9);
}

#[test]
fn test_gate_z() {
    let gate = HilbertState::gate_z();
    let i = Complex64::new(0.0, 1.0);
    // Z = i*e12 (e12 is at index 3, which is 0b11)
    let expected_data = create_expected_data_vector(3, i, SIZE);

    assert_complex_vec_approx_eq(gate.mv().data(), &expected_data, 1e-9);
    assert_eq!(gate.mv().metric(), Metric::NonEuclidean(DIM));

    // Physical property: Z^2 = I
    let gate_squared = gate.clone().into_inner() * gate.clone().into_inner();
    let mut expected_identity_data = vec![Complex64::zero(); SIZE];
    expected_identity_data[0] = Complex64::one();
    assert_complex_vec_approx_eq(gate_squared.data(), &expected_identity_data, 1e-9);
}

#[test]
fn test_gate_hadamard() {
    let gate = HilbertState::gate_hadamard();
    let x_gate = HilbertState::gate_x();
    let z_gate = HilbertState::gate_z();

    // H = (X + Z) / sqrt(2)
    let sum_mv = x_gate.into_inner() + z_gate.into_inner();
    let scale = Complex64::new(1.0 / 2.0f64.sqrt(), 0.0);
    let expected_h_mv = sum_mv * scale;

    assert_complex_vec_approx_eq(gate.mv().data(), expected_h_mv.data(), 1e-9);
    assert_eq!(gate.mv().metric(), Metric::NonEuclidean(DIM));

    // Physical property: H^2 = I
    let gate_squared = gate.clone().into_inner() * gate.clone().into_inner();
    let mut expected_identity_data = vec![Complex64::zero(); SIZE];
    expected_identity_data[0] = Complex64::one();
    assert_complex_vec_approx_eq(gate_squared.data(), &expected_identity_data, 1e-9);
}

#[test]
fn test_gate_s() {
    let gate = HilbertState::gate_s();
    let i_gate = HilbertState::gate_identity();
    let z_gate = HilbertState::gate_z();

    let theta = std::f64::consts::FRAC_PI_4; // pi/4
    let cos_theta = Complex64::new(theta.cos(), 0.0);
    let sin_theta = Complex64::new(theta.sin(), 0.0);
    let minus_i = Complex64::new(0.0, -1.0);

    // S = cos(theta)I - i*sin(theta)Z
    let term1 = i_gate.into_inner() * cos_theta;
    let term2 = z_gate.into_inner() * (minus_i * sin_theta);
    let expected_s_mv = term1 + term2;

    assert_complex_vec_approx_eq(gate.mv().data(), expected_s_mv.data(), 1e-9);
    assert_eq!(gate.mv().metric(), Metric::NonEuclidean(DIM));
}

#[test]
fn test_gate_t() {
    let gate = HilbertState::gate_t();
    let i_gate = HilbertState::gate_identity();
    let z_gate = HilbertState::gate_z();

    let theta = std::f64::consts::FRAC_PI_8; // pi/8
    let cos_theta = Complex64::new(theta.cos(), 0.0);
    let sin_theta = Complex64::new(theta.sin(), 0.0);
    let minus_i = Complex64::new(0.0, -1.0);

    // T = cos(theta)I - i*sin(theta)Z
    let term1 = i_gate.into_inner() * cos_theta;
    let term2 = z_gate.into_inner() * (minus_i * sin_theta);
    let expected_t_mv = term1 + term2;

    assert_complex_vec_approx_eq(gate.mv().data(), expected_t_mv.data(), 1e-9);
    assert_eq!(gate.mv().metric(), Metric::NonEuclidean(DIM));
}
