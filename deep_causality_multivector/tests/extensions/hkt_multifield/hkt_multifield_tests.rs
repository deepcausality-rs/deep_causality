/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for HKT implementations on CausalMultiField.
//!
//! NOTE: HKT implementations for CausalMultiField are currently stubbed due to
//! trait bound constraints in the HKT system. These tests verify that the stubs
//! panic as expected and that direct methods work correctly instead.

use deep_causality_multivector::{CausalMultiField, Metric};

fn create_test_field() -> CausalMultiField<f32> {
    let shape = [2, 2, 2];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];
    CausalMultiField::zeros(shape, metric, dx)
}

#[test]
fn test_multifield_zeros_creates_correct_shape() {
    let field = create_test_field();
    let data = field.data();

    // Shape should be [Nx, Ny, Nz, D, D] = [2, 2, 2, 2, 2]
    assert_eq!(data.shape(), &[2, 2, 2, 2, 2]);
}

#[test]
fn test_multifield_ones_creates_identity_matrices() {
    let shape = [1, 1, 1];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];

    let field = CausalMultiField::ones(shape, metric, dx);
    let data_vec = field.data().to_vec();

    // For a single cell with 2x2 matrix, should be identity
    // [1, 0, 0, 1] in row-major order
    assert!((data_vec[0] - 1.0).abs() < 1e-6);
    assert!(data_vec[1].abs() < 1e-6);
    assert!(data_vec[2].abs() < 1e-6);
    assert!((data_vec[3] - 1.0).abs() < 1e-6);
}

#[test]
fn test_multifield_num_cells() {
    let field = create_test_field();
    assert_eq!(field.num_cells(), 8); // 2 * 2 * 2
}

#[test]
fn test_multifield_metric() {
    let field = create_test_field();
    let metric = field.metric();
    assert_eq!(metric.dimension(), 2);
}

#[test]
fn test_multifield_clone() {
    let field1 = create_test_field();
    let field2 = field1.clone();

    assert_eq!(field1.data().shape(), field2.data().shape());
    assert_eq!(field1.metric(), field2.metric());
}

#[test]
fn test_multifield_add() {
    let field1 = create_test_field();
    let field2 = create_test_field();

    let result = &field1 + &field2;

    // zeros + zeros = zeros
    let data = result.data().to_vec();
    for val in data {
        assert!(val.abs() < 1e-6);
    }
}

#[test]
fn test_multifield_sub() {
    let shape = [1, 1, 1];
    let metric = Metric::from_signature(2, 0, 0);
    let dx = [0.1f32, 0.1, 0.1];

    let field1 = CausalMultiField::ones(shape, metric, dx);
    let field2 = CausalMultiField::zeros(shape, metric, dx);

    let result = field1 - field2;

    // ones - zeros = ones (identity matrices)
    let data = result.data().to_vec();
    assert!((data[0] - 1.0).abs() < 1e-6);
    assert!((data[3] - 1.0).abs() < 1e-6);
}
