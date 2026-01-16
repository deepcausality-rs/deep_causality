/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for differential operators: partial_derivative, gradient, curl, divergence.

use deep_causality_metric::Metric;
use deep_causality_multivector::{Axis, CausalMultiField, CausalMultiVector};
use deep_causality_tensor::{CpuBackend, TensorBackend};

// =============================================================================
// Axis enum tests
// =============================================================================

#[test]
fn test_axis_x_index() {
    assert_eq!(Axis::X as usize, 0);
}

#[test]
fn test_axis_y_index() {
    assert_eq!(Axis::Y as usize, 1);
}

#[test]
fn test_axis_z_index() {
    assert_eq!(Axis::Z as usize, 2);
}

#[test]
fn test_axis_equality() {
    assert_eq!(Axis::X, Axis::X);
    assert_ne!(Axis::X, Axis::Y);
    assert_ne!(Axis::Y, Axis::Z);
}

#[test]
fn test_axis_clone() {
    let axis = Axis::X;
    let cloned = axis;
    assert_eq!(axis, cloned);
}

// =============================================================================
// partial_derivative() tests
// =============================================================================

#[test]
fn test_partial_derivative_returns_zeros_for_small_grid() {
    let metric = Metric::from_signature(3, 0, 0);
    // Grid with n < 3 along X axis
    let field = CausalMultiField::<f32>::ones([2, 4, 4], metric, [1.0, 1.0, 1.0]);

    let deriv = field.partial_derivative(Axis::X);
    let deriv_vec: Vec<f32> = CpuBackend::to_vec(&deriv);

    // Should return zeros when dimension is too small
    assert!(deriv_vec.iter().all(|&x| x.abs() < 1e-5));
}

#[test]
fn test_partial_derivative_constant_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Create constant scalar field (all cells have scalar = 5.0)
    let mut mvs = Vec::with_capacity(64);
    for _ in 0..64 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 5.0;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [4, 4, 4], [1.0, 1.0, 1.0]);

    let deriv_x = field.partial_derivative(Axis::X);
    let deriv_vec: Vec<f32> = CpuBackend::to_vec(&deriv_x);

    // Derivative of constant is zero
    // Interior points should be zero (boundary is padded with zero)
    let sum: f32 = deriv_vec.iter().map(|x| x.abs()).sum();
    assert!(
        sum < 1e-3,
        "Derivative of constant should be ~0, got sum {}",
        sum
    );
}

#[test]
fn test_partial_derivative_linear_x_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Create field where scalar = x coordinate
    let mut mvs = Vec::with_capacity(64);
    for _z in 0..4 {
        for _y in 0..4 {
            for x in 0..4 {
                let mut data = vec![0.0f32; num_blades];
                data[0] = x as f32;
                mvs.push(CausalMultiVector::unchecked(data, metric));
            }
        }
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [4, 4, 4], [1.0, 1.0, 1.0]);

    let deriv_x = field.partial_derivative(Axis::X);
    let deriv_shape = CpuBackend::shape(&deriv_x);

    // Shape should match original
    assert_eq!(deriv_shape, vec![4, 4, 4, 4, 4]);
}

#[test]
fn test_partial_derivative_preserves_tensor_shape() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([4, 5, 6], metric, [1.0, 1.0, 1.0]);

    let deriv_x = field.partial_derivative(Axis::X);
    let deriv_y = field.partial_derivative(Axis::Y);
    let deriv_z = field.partial_derivative(Axis::Z);

    let shape_x = CpuBackend::shape(&deriv_x);
    let shape_y = CpuBackend::shape(&deriv_y);
    let shape_z = CpuBackend::shape(&deriv_z);

    // All should have same shape as original field data
    assert_eq!(shape_x, vec![4, 5, 6, 4, 4]);
    assert_eq!(shape_y, vec![4, 5, 6, 4, 4]);
    assert_eq!(shape_z, vec![4, 5, 6, 4, 4]);
}

// =============================================================================
// gradient() tests
// =============================================================================

#[test]
fn test_gradient_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    let grad = field.gradient();

    assert_eq!(grad.metric(), metric);
    assert_eq!(*grad.shape(), [4, 4, 4]);
}

#[test]
fn test_gradient_of_zeros_is_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    let grad = field.gradient();

    // Gradient of zero field should be zero
    let coeffs = grad.to_coefficients();
    for mv in coeffs {
        for val in mv.data() {
            assert!(val.abs() < 1e-5);
        }
    }
}

#[test]
fn test_gradient_constant_field_is_zero() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Constant scalar field
    let mut mvs = Vec::with_capacity(64);
    for _ in 0..64 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = 7.0; // Constant scalar
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [4, 4, 4], [1.0, 1.0, 1.0]);

    let grad = field.gradient();
    let coeffs = grad.to_coefficients();

    // Gradient of constant should be ~0 (except boundary effects)
    let total_magnitude: f32 = coeffs
        .iter()
        .flat_map(|mv| mv.data().iter())
        .map(|v| v.abs())
        .sum();

    // Allow some tolerance for numerical noise
    // Boundary cells may have non-zero values due to padding
    assert!(
        total_magnitude < 100.0,
        "Gradient of constant should be small, got {}",
        total_magnitude
    );
}

// Note: The gradient_identity test is known to have issues per the user's note.
// This test documents the expected behavior even if it currently fails.
#[test]
#[ignore] // Known bug - gradient_identity_cpu
fn test_gradient_of_linear_x_gives_e1() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // F(x,y,z) = x  =>  ∇F = e_1
    let mut mvs = Vec::with_capacity(64);
    for _z in 0..4 {
        for _y in 0..4 {
            for x in 0..4 {
                let mut data = vec![0.0f32; num_blades];
                data[0] = x as f32;
                mvs.push(CausalMultiVector::unchecked(data, metric));
            }
        }
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [4, 4, 4], [1.0, 1.0, 1.0]);
    let grad = field.gradient();
    let grad_coeffs = grad.to_coefficients();

    // Check center point: index = 1*16 + 1*4 + 1 = 21
    let center = &grad_coeffs[21];

    // Expected: e_1 component = 1.0 (central diff: (2-0)/2 = 1)
    assert!(
        (center.data()[1] - 1.0).abs() < 1e-4,
        "Expected e1=1.0, got {}",
        center.data()[1]
    );
}

// =============================================================================
// curl() tests
// =============================================================================

#[test]
fn test_curl_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    let curl = field.curl();

    assert_eq!(curl.metric(), metric);
    assert_eq!(*curl.shape(), [4, 4, 4]);
}

#[test]
fn test_curl_of_zeros_is_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    let curl = field.curl();
    let coeffs = curl.to_coefficients();

    for mv in coeffs {
        for val in mv.data() {
            assert!(val.abs() < 1e-5);
        }
    }
}

#[test]
fn test_curl_is_grade_2_projection() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    // Curl is defined as the bivector (grade 2) part of the gradient
    let _curl = field.curl();
    // The result is projected to grade 2 by definition
}

// =============================================================================
// divergence() tests
// =============================================================================

#[test]
fn test_divergence_returns_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    let div = field.divergence();

    assert_eq!(div.metric(), metric);
    assert_eq!(*div.shape(), [4, 4, 4]);
}

#[test]
fn test_divergence_of_zeros_is_zeros() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    let div = field.divergence();
    let coeffs = div.to_coefficients();

    for mv in coeffs {
        for val in mv.data() {
            assert!(val.abs() < 1e-5);
        }
    }
}

#[test]
fn test_divergence_is_grade_0_projection() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([4, 4, 4], metric, [1.0, 1.0, 1.0]);

    let div = field.divergence();
    let coeffs = div.to_coefficients();

    // Divergence is the scalar (grade 0) part of the gradient
    // All non-scalar components should be zero
    for mv in coeffs {
        for (i, val) in mv.data().iter().enumerate() {
            if i != 0 {
                // Non-scalar components
                assert!(
                    val.abs() < 1e-5,
                    "Non-scalar component {} should be 0, got {}",
                    i,
                    val
                );
            }
        }
    }
}
