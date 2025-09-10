/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{CausalTensor, causal_tensor_type::error::CausalTensorError};

// --- sum_axes tests ---

#[test]
fn test_sum_axes_2d() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();

    // Sum along axis 0 (columns)
    let sum0 = tensor.sum_axes(&[0]).unwrap();
    assert_eq!(sum0.shape(), &[3]);
    assert_eq!(sum0.as_slice(), &[5, 7, 9]);

    // Sum along axis 1 (rows)
    let sum1 = tensor.sum_axes(&[1]).unwrap();
    assert_eq!(sum1.shape(), &[2]);
    assert_eq!(sum1.as_slice(), &[6, 15]);
}

#[test]
fn test_sum_axes_full_reduction() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let sum_all = tensor.sum_axes(&[]).unwrap();
    assert_eq!(sum_all.shape(), &[]);
    assert_eq!(sum_all.as_slice(), &[21]);
}

#[test]
fn test_sum_axes_3d() {
    let data = (1..=8).collect();
    let tensor = CausalTensor::new(data, vec![2, 2, 2]).unwrap();
    // Tensor:
    // [[[1, 2], [3, 4]],
    //  [[5, 6], [7, 8]]]

    // Sum along axis 0
    let sum0 = tensor.sum_axes(&[0]).unwrap();
    assert_eq!(sum0.shape(), &[2, 2]);
    assert_eq!(sum0.as_slice(), &[6, 8, 10, 12]);

    // Sum along axes 0 and 2
    let sum02 = tensor.sum_axes(&[0, 2]).unwrap();
    assert_eq!(sum02.shape(), &[2]);
    assert_eq!(sum02.as_slice(), &[14, 22]);
}

#[test]
fn test_sum_axes_empty_tensor() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![2, 0]).unwrap();

    // Sum along an axis of a tensor with a zero-sized dimension
    let sum = tensor.sum_axes(&[0]).unwrap();
    assert_eq!(sum.shape(), &[0]);
    assert!(sum.is_empty());

    // Full reduction of an empty tensor should result in a scalar 0
    let sum_all = tensor.sum_axes(&[]).unwrap();
    assert_eq!(sum_all.shape(), &[]);
    assert_eq!(sum_all.as_slice(), &[0]);
}

#[test]
fn test_sum_axes_out_of_bounds() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let result = tensor.sum_axes(&[2]);
    assert_eq!(result, Err(CausalTensorError::AxisOutOfBounds));
}

// --- mean_axes tests ---

#[test]
fn test_mean_axes_2d() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();

    // Mean along axis 0
    let mean0 = tensor.mean_axes(&[0]).unwrap();
    assert_eq!(mean0.shape(), &[3]);
    assert_eq!(mean0.as_slice(), &[2.5, 3.5, 4.5]);

    // Mean along axis 1
    let mean1 = tensor.mean_axes(&[1]).unwrap();
    assert_eq!(mean1.shape(), &[2]);
    assert_eq!(mean1.as_slice(), &[2.0, 5.0]);
}

#[test]
fn test_mean_axes_full_reduction() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let mean_all = tensor.mean_axes(&[]).unwrap();
    assert_eq!(mean_all.shape(), &[]);
    assert_eq!(mean_all.as_slice(), &[3.5]);
}

#[test]
fn test_mean_axes_out_of_bounds() {
    let tensor = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let result = tensor.mean_axes(&[1]);
    assert_eq!(result, Err(CausalTensorError::AxisOutOfBounds));

    let result_sum = tensor.sum_axes(&[1]);
    assert_eq!(result_sum, Err(CausalTensorError::AxisOutOfBounds));
}

#[test]
fn test_mean_axes_div_by_zero() {
    let tensor: CausalTensor<f64> = CausalTensor::new(vec![], vec![2, 0]).unwrap();
    let result = tensor.mean_axes(&[1]); // reduce over the 0-sized dimension
    assert_eq!(result, Err(CausalTensorError::InvalidOperation));
}

// --- arg_sort tests ---

#[test]
fn test_arg_sort_success() {
    let tensor = CausalTensor::new(vec![3, 1, 4, 1, 5, 9, 2, 6], vec![8]).unwrap();
    let indices = tensor.arg_sort().unwrap();
    assert_eq!(indices, vec![1, 3, 6, 0, 2, 4, 7, 5]);
}

#[test]
fn test_arg_sort_empty() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let indices = tensor.arg_sort().unwrap();
    assert!(indices.is_empty());
}

#[test]
fn test_arg_sort_dimension_mismatch() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let result = tensor.arg_sort();
    assert_eq!(result, Err(CausalTensorError::DimensionMismatch));
}

#[test]
fn test_arg_sort_unorderable_values() {
    let data = vec![1.0, f64::NAN, 2.0];
    let tensor = CausalTensor::new(data, vec![3]).unwrap();
    let result = tensor.arg_sort();
    assert_eq!(result, Err(CausalTensorError::UnorderableValue));
}
