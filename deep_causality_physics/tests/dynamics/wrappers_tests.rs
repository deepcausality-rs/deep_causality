/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Frequency, Mass, MomentOfInertia, angular_momentum, kalman_filter_linear, kinetic_energy,
    rotational_kinetic_energy, torque,
};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// kinetic_energy Wrapper Tests
// =============================================================================

#[test]
fn test_kinetic_energy_wrapper_success() {
    let mass = Mass::new(2.0).unwrap();
    let velocity = CausalMultiVector::new(
        vec![0.0, 3.0, 4.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let effect = kinetic_energy(&mass, &velocity);
    assert!(effect.is_ok());

    let energy = effect.value().clone().into_value().unwrap();
    assert!(energy.value() > 0.0);
}

// =============================================================================
// rotational_kinetic_energy Wrapper Tests
// =============================================================================

#[test]
fn test_rotational_kinetic_energy_wrapper_success() {
    let inertia = MomentOfInertia::new(4.0).unwrap();
    let omega = Frequency::new(3.0).unwrap();

    let effect = rotational_kinetic_energy(&inertia, &omega);
    assert!(effect.is_ok());

    let energy = effect.value().clone().into_value().unwrap();
    assert!(energy.value() > 0.0);
}

// =============================================================================
// torque Wrapper Tests
// =============================================================================

#[test]
fn test_torque_wrapper_success() {
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let force = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let effect = torque(&radius, &force);
    assert!(effect.is_ok());
}

// =============================================================================
// angular_momentum Wrapper Tests
// =============================================================================

#[test]
fn test_angular_momentum_wrapper_success() {
    let radius = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let momentum = CausalMultiVector::new(
        vec![0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let effect = angular_momentum(&radius, &momentum);
    assert!(effect.is_ok());
    let effect = angular_momentum(&radius, &momentum);
    assert!(effect.is_ok());
}

// =============================================================================
// kalman_filter_linear Wrapper Tests
// =============================================================================

#[test]
fn test_kalman_filter_linear_wrapper_success() {
    // 1D Kalman Filter test
    // State: [position]
    let x_pred = CausalTensor::new(vec![10.0], vec![1, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![5.0], vec![1, 1]).unwrap();
    let measurement = CausalTensor::new(vec![12.0], vec![1, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let r = CausalTensor::new(vec![2.0], vec![1, 1]).unwrap();
    let q = CausalTensor::new(vec![0.1], vec![1, 1]).unwrap();

    let effect = kalman_filter_linear(&x_pred, &p_pred, &measurement, &h, &r, &q);
    assert!(effect.is_ok());

    let (x_new, p_new) = effect.value().clone().into_value().unwrap();
    // Verify state was updated towards measurement (12.0)
    assert!(x_new.data()[0] > 10.0);
    // Verify covariance decreased
    assert!(p_new.data()[0] < 5.0);
}

#[test]
fn test_kalman_filter_linear_wrapper_error() {
    // Dimension mismatch error
    let x_pred = CausalTensor::new(vec![10.0], vec![1, 1]).unwrap();
    let p_pred = CausalTensor::new(vec![5.0], vec![1, 1]).unwrap();
    // Measurement has wrong dimension [2,1] vs state [1,1]
    let measurement = CausalTensor::new(vec![12.0, 13.0], vec![2, 1]).unwrap();
    let h = CausalTensor::new(vec![1.0], vec![1, 1]).unwrap();
    let r = CausalTensor::new(vec![2.0], vec![1, 1]).unwrap();
    let q = CausalTensor::new(vec![0.1], vec![1, 1]).unwrap();

    let effect = kalman_filter_linear(&x_pred, &p_pred, &measurement, &h, &r, &q);
    assert!(effect.is_err());
}

// =============================================================================
// Error Propagation Tests for wrappers
// =============================================================================

#[test]
fn test_torque_wrapper_error_propagation() {
    // Create mismatching metrics to force an error in outer product
    let radius = CausalMultiVector::new(vec![1.0, 0.0], Metric::Euclidean(1)).unwrap();
    let force = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0],
        Metric::Euclidean(2), // Different metric/dimension
    )
    .unwrap();

    let effect = torque(&radius, &force);
    assert!(effect.is_err());
}

#[test]
fn test_angular_momentum_wrapper_error_propagation() {
    let radius = CausalMultiVector::new(vec![1.0, 0.0], Metric::Euclidean(1)).unwrap();
    let momentum = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], Metric::Euclidean(2)).unwrap();

    let effect = angular_momentum(&radius, &momentum);
    assert!(effect.is_err());
}

// =============================================================================
// generalized_master_equation Wrapper Tests
// =============================================================================

#[test]
fn test_generalized_master_equation_wrapper_success() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation;

    let state = vec![Probability::new(0.5).unwrap()];
    let history: Vec<Vec<Probability>> = vec![];
    let mk: Vec<CausalTensor<f64>> = vec![];

    // Test simple identity/zero op case
    let effect = generalized_master_equation(&state, &history, None, &mk);

    assert!(effect.is_ok());
    let res = effect.value().clone().into_value().unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].value(), 0.0);
}

#[test]
fn test_generalized_master_equation_wrapper_error() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation;

    let state = vec![Probability::new(0.5).unwrap()];
    // Error condition: History length does not match Memory Kernel length (0 != 1)
    let history: Vec<Vec<Probability>> = vec![];
    let k = CausalTensor::new(vec![0.1], vec![1, 1]).unwrap();
    let mk = vec![k];

    let effect = generalized_master_equation(&state, &history, None, &mk);
    assert!(effect.is_err());
}
