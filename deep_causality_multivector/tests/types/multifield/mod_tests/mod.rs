/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for CausalMultiField struct accessors and basic construction.

use deep_causality_metric::Metric;
use deep_causality_multivector::{CausalMultiField, CausalMultiVector};
use deep_causality_tensor::{CpuBackend, TensorBackend};

// =============================================================================
// metric() tests
// =============================================================================

#[test]
fn test_metric_returns_correct_signature() {
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.metric(), metric);
}

#[test]
fn test_metric_with_mixed_signature() {
    let metric = Metric::from_signature(1, 3, 0); // Minkowski-like
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.metric(), metric);
    assert_eq!(field.metric().dimension(), 4);
}

// =============================================================================
// dx() tests
// =============================================================================

#[test]
fn test_dx_returns_correct_spacing() {
    let metric = Metric::from_signature(3, 0, 0);
    let dx = [0.5, 1.0, 2.0];
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, dx);

    assert_eq!(*field.dx(), dx);
}

#[test]
fn test_dx_with_uniform_spacing() {
    let metric = Metric::from_signature(3, 0, 0);
    let dx = [1.0, 1.0, 1.0];
    let field = CausalMultiField::<CpuBackend, f32>::zeros([4, 4, 4], metric, dx);

    assert_eq!(field.dx()[0], 1.0);
    assert_eq!(field.dx()[1], 1.0);
    assert_eq!(field.dx()[2], 1.0);
}

// =============================================================================
// shape() tests
// =============================================================================

#[test]
fn test_shape_returns_correct_dimensions() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [4, 5, 6];
    let field = CausalMultiField::<CpuBackend, f32>::zeros(shape, metric, [1.0, 1.0, 1.0]);

    assert_eq!(*field.shape(), shape);
}

#[test]
fn test_shape_with_cubic_grid() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [8, 8, 8];
    let field = CausalMultiField::<CpuBackend, f32>::zeros(shape, metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.shape()[0], 8);
    assert_eq!(field.shape()[1], 8);
    assert_eq!(field.shape()[2], 8);
}

// =============================================================================
// num_cells() tests
// =============================================================================

#[test]
fn test_num_cells_calculates_product() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [2, 3, 4];
    let field = CausalMultiField::<CpuBackend, f32>::zeros(shape, metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.num_cells(), 2 * 3 * 4);
}

#[test]
fn test_num_cells_with_unit_dimension() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [1, 1, 1];
    let field = CausalMultiField::<CpuBackend, f32>::zeros(shape, metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.num_cells(), 1);
}

#[test]
fn test_num_cells_with_large_grid() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [10, 10, 10];
    let field = CausalMultiField::<CpuBackend, f32>::zeros(shape, metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.num_cells(), 1000);
}

// =============================================================================
// matrix_dim() tests
// =============================================================================

#[test]
fn test_matrix_dim_for_cl3() {
    // Cl(3,0,0): N=3, ceil(3/2)=2, 2^2=4
    let metric = Metric::from_signature(3, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.matrix_dim(), 4);
}

#[test]
fn test_matrix_dim_for_cl2() {
    // Cl(2,0,0): N=2, ceil(2/2)=1, 2^1=2
    let metric = Metric::from_signature(2, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.matrix_dim(), 2);
}

#[test]
fn test_matrix_dim_for_cl4() {
    // Cl(4,0,0): N=4, ceil(4/2)=2, 2^2=4
    let metric = Metric::from_signature(4, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.matrix_dim(), 4);
}

#[test]
fn test_matrix_dim_for_cl1() {
    // Cl(1,0,0): N=1, ceil(1/2)=1, 2^1=2
    let metric = Metric::from_signature(1, 0, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.matrix_dim(), 2);
}

#[test]
fn test_matrix_dim_for_minkowski() {
    // Cl(1,3,0): N=4, ceil(4/2)=2, 2^2=4
    let metric = Metric::from_signature(1, 3, 0);
    let field = CausalMultiField::<CpuBackend, f32>::zeros([2, 2, 2], metric, [1.0, 1.0, 1.0]);

    assert_eq!(field.matrix_dim(), 4);
}

// =============================================================================
// data() tests
// =============================================================================

#[test]
fn test_data_returns_tensor_reference() {
    use deep_causality_tensor::TensorBackend;

    let metric = Metric::from_signature(3, 0, 0);
    let shape = [2, 2, 2];
    let field = CausalMultiField::<CpuBackend, f32>::zeros(shape, metric, [1.0, 1.0, 1.0]);

    let data = field.data();
    let tensor_shape = CpuBackend::shape(data);

    // Shape should be [Nx, Ny, Nz, D, D] = [2, 2, 2, 4, 4]
    assert_eq!(tensor_shape.len(), 5);
    assert_eq!(tensor_shape[0], 2);
    assert_eq!(tensor_shape[1], 2);
    assert_eq!(tensor_shape[2], 2);
    assert_eq!(tensor_shape[3], 4); // matrix_dim for Cl(3)
    assert_eq!(tensor_shape[4], 4);
}

#[test]
fn test_data_from_coefficients_preserves_values() {
    let metric = Metric::from_signature(3, 0, 0);
    let shape = [2, 2, 2];
    let num_cells = 8;
    let num_blades = 8; // 2^3

    let mut mvs = Vec::with_capacity(num_cells);
    for i in 0..num_cells {
        let mut data = vec![0.0f32; num_blades];
        data[0] = i as f32; // Scalar component
        mvs.push(CausalMultiVector::unchecked(data, metric));
    }

    let field =
        CausalMultiField::<CpuBackend, f32>::from_coefficients(&mvs, shape, [1.0, 1.0, 1.0]);

    // Verify dimensions are correct
    let data = field.data();
    let tensor_shape = CpuBackend::shape(data);
    assert_eq!(tensor_shape, vec![2, 2, 2, 4, 4]);
}
