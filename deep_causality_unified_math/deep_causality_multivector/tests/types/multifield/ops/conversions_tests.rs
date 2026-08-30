/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for CausalMultiField conversion operations: zeros, ones, from_coefficients, to_coefficients.

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;
// =============================================================================
// zeros() tests
// =============================================================================

#[test]
fn test_zeros_creates_zero_field() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert!(field.is_zero());
}

#[test]
fn test_zeros_has_correct_shape() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [3, 4, 5];
    let field = CausalMultiField::<f32>::zeros(shape, metric, [1.0, 1.0, 1.0]);

    assert_eq!(*field.shape(), shape);
    assert_eq!(field.num_cells(), 60);
}

#[test]
fn test_zeros_tensor_shape() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([2, 3, 4], metric, [1.0, 1.0, 1.0]);

    let tensor_shape = CausalTensor::shape(field.data());
    // [Nx, Ny, Nz, D, D] = [2, 3, 4, 4, 4]
    assert_eq!(tensor_shape, vec![2, 3, 4, 4, 4]);
}

#[test]
fn test_zeros_preserves_metric() {
    let metric = Metric::from_signature(1, 3, 0);
    let field = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.metric(), metric);
}

#[test]
fn test_zeros_preserves_dx() {
    let metric = Metric::from_signature(3, 0, 0);
    let dx = [0.1, 0.2, 0.3];
    let field = CausalMultiField::<f32>::zeros([2, 2, 2], metric, dx);

    assert_eq!(*field.dx(), dx);
}

#[test]
fn test_ones_not_zero() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert!(!field.is_zero());
}

#[test]
fn test_ones_has_correct_shape() {
    let metric = Metric::from_signature(2, 0, 0);
    let shape = [3, 3, 3];
    let field = CausalMultiField::<f32>::ones(shape, metric, [1.0, 1.0, 1.0]);

    assert_eq!(*field.shape(), shape);
}

// =============================================================================
// from_coefficients() tests
// =============================================================================

#[test]
fn test_from_coefficients_correct_cell_count() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;
    let shape = [2, 2, 2];

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, shape, [1.0, 1.0, 1.0]);

    assert_eq!(field.num_cells(), 8);
}

#[test]
#[should_panic(expected = "Expected 8 multivectors")]
fn test_from_coefficients_wrong_count_panics() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    // Provide 4 multivectors but shape requires 8
    let mut mvs = Vec::with_capacity(4);
    for i in 0..4 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32;
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let _ = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
}

#[test]
#[should_panic(expected = "Cannot create field from empty")]
fn test_from_coefficients_empty_panics() {
    let mvs: Vec<CausalMultiVector<f32>> = Vec::new();

    let _ = CausalMultiField::<f32>::from_coefficients(&mvs, [0, 0, 0], [1.0, 1.0, 1.0]);
}

#[test]
fn test_from_coefficients_preserves_metric() {
    let metric = Metric::from_signature(2, 1, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for _ in 0..8 {
        mvs.push(CausalMultiVector::unchecked(
            vec![0.0f32; num_blades],
            metric,
        ));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);

    assert_eq!(field.metric(), metric);
}

// =============================================================================
// to_coefficients() tests
// =============================================================================

#[test]
fn test_to_coefficients_returns_correct_count() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([2, 3, 4], metric, [1.0, 1.0, 1.0]);

    let coeffs = field.to_coefficients();

    assert_eq!(coeffs.len(), 24); // 2 * 3 * 4
}

#[test]
fn test_to_coefficients_preserves_metric() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    let coeffs = field.to_coefficients();

    for mv in coeffs {
        assert_eq!(mv.metric(), metric);
    }
}

// =============================================================================
// Roundtrip tests (from_coefficients -> to_coefficients)
// =============================================================================

#[test]
fn test_roundtrip_preserves_scalar_values() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32; // Scalar part
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let recovered = field.to_coefficients();

    for (orig, rec) in mvs.iter().zip(recovered.iter()) {
        for (o, r) in orig.data().iter().zip(rec.data().iter()) {
            assert!((o - r).abs() < 1e-4, "Roundtrip mismatch: {} != {}", o, r);
        }
    }
}

#[test]
fn test_roundtrip_preserves_vector_values() {
    let metric = Metric::from_signature(3, 0, 0);
    let num_blades = 8;

    let mut mvs = Vec::with_capacity(8);
    for i in 0..8 {
        let mut data = vec![0.0f32; num_blades];
        data[1] = (i + 1) as f32; // e_1 component
        data[2] = (i + 2) as f32; // e_2 component
        data[4] = (i + 3) as f32; // e_3 component
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field = CausalMultiField::<f32>::from_coefficients(&mvs, [2, 2, 2], [1.0, 1.0, 1.0]);
    let recovered = field.to_coefficients();

    for (orig, rec) in mvs.iter().zip(recovered.iter()) {
        for (o, r) in orig.data().iter().zip(rec.data().iter()) {
            assert!((o - r).abs() < 1e-4, "Roundtrip mismatch: {} != {}", o, r);
        }
    }
}

#[test]
fn test_roundtrip_identity_field() {
    let metric = Metric::from_signature(3, 0, 0);

    // ones() creates identity matrices, which correspond to scalar=1
    let field = CausalMultiField::<f32>::ones([2, 2, 2], metric, [1.0, 1.0, 1.0]);
    let coeffs = field.to_coefficients();

    for mv in coeffs {
        // Scalar (identity) should be 1, others near 0
        assert!(
            (mv.data()[0] - 1.0).abs() < 1e-4,
            "Identity scalar should be 1, got {}",
            mv.data()[0]
        );
    }
}

// =============================================================================
// compute_matrix_dim() tests
// =============================================================================

#[test]
fn test_compute_matrix_dim_n1() {
    // N=1: ceil(1/2) = 1, 2^1 = 2
    assert_eq!(CausalMultiField::<f32>::compute_matrix_dim(1), 2);
}

#[test]
fn test_compute_matrix_dim_n2() {
    // N=2: ceil(2/2) = 1, 2^1 = 2
    assert_eq!(CausalMultiField::<f32>::compute_matrix_dim(2), 2);
}

#[test]
fn test_compute_matrix_dim_n3() {
    // N=3: ceil(3/2) = 2, 2^2 = 4
    assert_eq!(CausalMultiField::<f32>::compute_matrix_dim(3), 4);
}

#[test]
fn test_compute_matrix_dim_n4() {
    // N=4: ceil(4/2) = 2, 2^2 = 4
    assert_eq!(CausalMultiField::<f32>::compute_matrix_dim(4), 4);
}

#[test]
fn test_compute_matrix_dim_n5() {
    // N=5: ceil(5/2) = 3, 2^3 = 8
    assert_eq!(CausalMultiField::<f32>::compute_matrix_dim(5), 8);
}

#[test]
fn test_compute_matrix_dim_n6() {
    // N=6: ceil(6/2) = 3, 2^3 = 8
    assert_eq!(CausalMultiField::<f32>::compute_matrix_dim(6), 8);
}
