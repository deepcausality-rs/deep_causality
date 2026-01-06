/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{einstein_tensor_kernel, geodesic_deviation_kernel};
use deep_causality_tensor::CausalTensor;

// =============================================================================
// einstein_tensor_kernel Tests
// =============================================================================

#[test]
fn test_einstein_tensor_kernel_valid() {
    // G_uv = R_uv - 0.5 * R * g_uv
    // 2x2 tensors for simplicity
    let ricci = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let scalar_r = 2.0; // Trace of Ricci

    let result = einstein_tensor_kernel(&ricci, scalar_r, &metric);
    assert!(result.is_ok());

    let g = result.unwrap();
    // G = R - 0.5*2*I = I - I = 0
    assert!((g.data()[0] - 0.0f64).abs() < 1e-10);
}

#[test]
fn test_einstein_tensor_kernel_dimension_error() {
    let ricci = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 2], vec![2]).unwrap(); // Rank 1
    let result = einstein_tensor_kernel(&ricci, 2.0, &metric);
    assert!(result.is_err());
}

#[test]
fn test_einstein_tensor_kernel_shape_mismatch() {
    let ricci = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = einstein_tensor_kernel(&ricci, 2.0, &metric);
    assert!(result.is_err());
}

#[test]
fn test_einstein_tensor_kernel_non_square() {
    let ricci = CausalTensor::new(vec![1.0; 4], vec![1, 4]).unwrap();
    let metric = CausalTensor::new(vec![1.0; 4], vec![1, 4]).unwrap();
    let result = einstein_tensor_kernel(&ricci, 2.0, &metric);
    assert!(result.is_err());
}

#[test]
fn test_einstein_tensor_kernel_flat_space() {
    // Flat Minkowski: R_uv = 0, R = 0
    let ricci = CausalTensor::new(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2]).unwrap();
    let metric = CausalTensor::new(vec![1.0, 0.0, 0.0, -1.0], vec![2, 2]).unwrap();
    let scalar_r = 0.0;

    let result = einstein_tensor_kernel(&ricci, scalar_r, &metric);
    assert!(result.is_ok());

    let g = result.unwrap();
    // G = 0 - 0 = 0
    for val in g.data() {
        assert!(*val == 0.0);
    }
}

// =============================================================================
// geodesic_deviation_kernel Tests
// =============================================================================

#[test]
fn test_geodesic_deviation_kernel_valid() {
    // A^u = -R^u_vsp * V^v * n^s * V^p
    // Rank 4 Riemann [4,4,4,4], velocity and separation as slices
    let riemann = CausalTensor::new(vec![0.0f64; 256], vec![4, 4, 4, 4]).unwrap();
    let velocity: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let separation: [f64; 4] = [0.0, 1.0, 0.0, 0.0];

    let result = geodesic_deviation_kernel(&riemann, &velocity, &separation);
    assert!(result.is_ok());

    let a = result.unwrap();
    // Zero Riemann => zero deviation
    for val in a.iter() {
        assert!(*val == 0.0);
    }
}

#[test]
fn test_geodesic_deviation_kernel_dimension_error() {
    // Wrong ranks should error
    let riemann = CausalTensor::new(vec![1.0f64; 4], vec![2, 2]).unwrap(); // Rank 2, not 4
    let velocity: [f64; 4] = [1.0, 0.0, 0.0, 0.0];
    let separation: [f64; 4] = [0.0, 1.0, 0.0, 0.0];

    let result = geodesic_deviation_kernel(&riemann, &velocity, &separation);
    assert!(result.is_err(), "Should error on wrong Riemann rank");
}

// =============================================================================
// geodesic_integrator_kernel Tests
// =============================================================================

use deep_causality_physics::geodesic_integrator_kernel;

#[test]
fn test_geodesic_integrator_kernel_flat_space() {
    // In flat space (Γ = 0), geodesics are straight lines
    let initial_position: Vec<f64> = vec![0.0, 0.0, 0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.1, 0.0, 0.0]; // Moving mostly in t

    // Zero Christoffel symbols (flat space)
    let christoffel = CausalTensor::new(vec![0.0f64; 64], vec![4, 4, 4]).unwrap();

    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.1, 10);
    assert!(result.is_ok());

    let trajectory = result.unwrap();
    assert_eq!(trajectory.len(), 11); // Initial + 10 steps

    // In flat space, velocity should remain constant
    let (_, final_vel) = &trajectory[10];
    assert!((final_vel[0] - initial_velocity[0]).abs() < 1e-10);
    assert!((final_vel[1] - initial_velocity[1]).abs() < 1e-10);
}

#[test]
fn test_geodesic_integrator_kernel_straight_line() {
    // Verify position changes linearly in flat space
    let initial_position: Vec<f64> = vec![0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 2.0];

    let christoffel = CausalTensor::new(vec![0.0f64; 8], vec![2, 2, 2]).unwrap();
    let dt = 0.1;
    let num_steps = 5;

    let result = geodesic_integrator_kernel(
        &initial_position,
        &initial_velocity,
        &christoffel,
        dt,
        num_steps,
    );
    assert!(result.is_ok());

    let trajectory = result.unwrap();
    let (final_pos, _) = &trajectory[num_steps];

    // Expected: x = x0 + v * t = 0 + [1, 2] * 0.5 = [0.5, 1.0]
    let expected_x = initial_velocity[0] * dt * num_steps as f64;
    let expected_y = initial_velocity[1] * dt * num_steps as f64;

    assert!(
        (final_pos[0] - expected_x).abs() < 1e-9,
        "Expected x={}, got {}",
        expected_x,
        final_pos[0]
    );
    assert!(
        (final_pos[1] - expected_y).abs() < 1e-9,
        "Expected y={}, got {}",
        expected_y,
        final_pos[1]
    );
}

#[test]
fn test_geodesic_integrator_kernel_dimension_mismatch() {
    let initial_position: Vec<f64> = vec![0.0, 0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.0]; // Wrong dimension

    let christoffel = CausalTensor::new(vec![0.0f64; 27], vec![3, 3, 3]).unwrap();

    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.1, 10);
    assert!(result.is_err());
}

#[test]
fn test_geodesic_integrator_kernel_wrong_christoffel_rank() {
    let initial_position: Vec<f64> = vec![0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.0];

    let christoffel = CausalTensor::new(vec![0.0f64; 4], vec![2, 2]).unwrap(); // Rank 2, not 3

    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.1, 10);
    assert!(result.is_err());
}

#[test]
fn test_geodesic_integrator_kernel_invalid_step() {
    let initial_position: Vec<f64> = vec![0.0, 0.0];
    let initial_velocity: Vec<f64> = vec![1.0, 0.0];
    let christoffel = CausalTensor::new(vec![0.0f64; 8], vec![2, 2, 2]).unwrap();

    // Zero step size should error
    let result =
        geodesic_integrator_kernel(&initial_position, &initial_velocity, &christoffel, 0.0, 10);
    assert!(result.is_err());

    // NaN step size should error
    let result = geodesic_integrator_kernel(
        &initial_position,
        &initial_velocity,
        &christoffel,
        f64::NAN,
        10,
    );
    assert!(result.is_err());
}
