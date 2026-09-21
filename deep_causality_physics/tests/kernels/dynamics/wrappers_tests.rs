/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::{
    Frequency, Mass, MomentOfInertia, angular_momentum, angular_momentum_kernel,
    generalized_master_equation_kernel, kalman_filter_linear, kalman_filter_linear_kernel,
    kinetic_energy, kinetic_energy_kernel, rotational_kinetic_energy,
    rotational_kinetic_energy_kernel, torque, torque_kernel,
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

    // Delegation, not merely success. `torque` and `angular_momentum` in this file already had
    // this assertion; the two energy wrappers did not.
    let effect = kinetic_energy(&mass, &velocity);
    assert_eq!(
        effect.value_cloned().unwrap().value(),
        kinetic_energy_kernel(mass, &velocity).unwrap(),
        "kinetic_energy must carry the value its kernel produced"
    );

    // m = 2, v = (3, 4): |v|^2 = 25 and KE = 0.5 * 2 * 25 = 25.
    let energy = effect.value_cloned().unwrap();
    let ke: f64 = energy.value();
    assert!((ke - 25.0).abs() < 1e-12, "KE = {ke}");
}

// =============================================================================
// rotational_kinetic_energy Wrapper Tests
// =============================================================================

#[test]
fn test_rotational_kinetic_energy_wrapper_success() {
    let inertia = MomentOfInertia::new(4.0).unwrap();
    let omega = Frequency::new(3.0).unwrap();

    // Delegation, not merely success.
    let effect = rotational_kinetic_energy(&inertia, &omega);
    assert_eq!(
        effect.value_cloned().unwrap().value(),
        rotational_kinetic_energy_kernel(inertia, omega).unwrap(),
        "rotational_kinetic_energy must carry the value its kernel produced"
    );

    // I = 4, omega = 3: E = 0.5 * 4 * 9 = 18.
    let energy = effect.value_cloned().unwrap();
    let e: f64 = energy.value();
    assert!((e - 18.0).abs() < 1e-12, "E = {e}");
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
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        torque_kernel(&radius, &force).unwrap(),
        "torque must carry the value its kernel produced"
    );
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
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        angular_momentum_kernel(&radius, &momentum).unwrap(),
        "angular_momentum must carry the value its kernel produced"
    );
    let effect = angular_momentum(&radius, &momentum);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        angular_momentum_kernel(&radius, &momentum).unwrap(),
        "angular_momentum must carry the value its kernel produced"
    );
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
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        kalman_filter_linear_kernel(&x_pred, &p_pred, &measurement, &h, &r, &q).unwrap(),
        "kalman_filter_linear must carry the value its kernel produced"
    );

    let (x_new, p_new) = effect.value_cloned().unwrap();
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

    let state = vec![Probability::<f64>::new(0.5).unwrap()];
    let history: Vec<Vec<Probability<f64>>> = vec![];
    let mk: Vec<CausalTensor<f64>> = vec![];

    // Test simple identity/zero op case
    let effect = generalized_master_equation(&state, &history, None, &mk);
    // Delegation, not merely success: `assert!(effect.is_ok())` alone passed even when
    // a wrapper discarded its kernel's answer and returned a constant.
    assert_eq!(
        effect.value_cloned().unwrap(),
        generalized_master_equation_kernel(&state, &history, None, &mk).unwrap(),
        "generalized_master_equation must carry the value its kernel produced"
    );
    let res = effect.value_cloned().unwrap();
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].value(), 0.0);
}

#[test]
fn test_generalized_master_equation_wrapper_error() {
    use deep_causality_physics::Probability;
    use deep_causality_physics::generalized_master_equation;

    let state = vec![Probability::<f64>::new(0.5).unwrap()];
    // Error condition: History length does not match Memory Kernel length (0 != 1)
    let history: Vec<Vec<Probability<f64>>> = vec![];
    let k = CausalTensor::new(vec![0.1], vec![1, 1]).unwrap();
    let mk = vec![k];

    let effect = generalized_master_equation(&state, &history, None, &mk);
    assert!(effect.is_err());
}
