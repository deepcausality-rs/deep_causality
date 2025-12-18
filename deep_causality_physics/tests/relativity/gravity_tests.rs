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
    assert!((g.data()[0] - 0.0).abs() < 1e-10);
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
    // Rank 4 Riemann, Rank 1 velocity, Rank 1 separation
    // 2x2x2x2 = 16 elements
    let riemann = CausalTensor::new(vec![0.0; 16], vec![2, 2, 2, 2]).unwrap();
    let velocity = CausalTensor::new(vec![1.0, 0.0], vec![2]).unwrap();
    let separation = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();

    let result = geodesic_deviation_kernel(&riemann, &velocity, &separation);
    assert!(result.is_ok());

    let a = result.unwrap();
    // Zero Riemann => zero deviation
    for val in a.data() {
        assert!(*val == 0.0);
    }
}

#[test]
fn test_geodesic_deviation_kernel_dimension_error() {
    // Wrong ranks should error
    let riemann = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap(); // Rank 2, not 4
    let velocity = CausalTensor::new(vec![1.0, 0.0], vec![2]).unwrap();
    let separation = CausalTensor::new(vec![0.0, 1.0], vec![2]).unwrap();

    let result = geodesic_deviation_kernel(&riemann, &velocity, &separation);
    assert!(result.is_err(), "Should error on wrong Riemann rank");
}
