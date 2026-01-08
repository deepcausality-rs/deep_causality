/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_new_tensor_success() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let shape = vec![2, 3];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();

    assert_eq!(tensor.as_slice(), &data);
    assert_eq!(tensor.shape(), &shape);
    // Indirectly test strides via get method
    assert_eq!(tensor.get(&[0, 0]), Some(&1));
    assert_eq!(tensor.get(&[1, 2]), Some(&6));
}

#[test]
fn test_new_tensor_shape_mismatch() {
    let data = vec![1, 2, 3, 4, 5]; // 5 elements
    let shape = vec![2, 3]; // requires 6 elements
    let result = CausalTensor::new(data, shape);
    assert_eq!(result, Err(CausalTensorError::ShapeMismatch));
}

#[test]
fn test_new_scalar() {
    let data = vec![42];
    let shape = vec![];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();
    assert_eq!(tensor.as_slice(), &data);
    assert_eq!(tensor.shape(), &shape);
    assert_eq!(tensor.get(&[]), Some(&42));
}

#[test]
fn test_new_vector() {
    let data = vec![1, 2, 3];
    let shape = vec![3];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();
    assert_eq!(tensor.as_slice(), &data);
    assert_eq!(tensor.shape(), &shape);
    assert_eq!(tensor.get(&[0]), Some(&1));
    assert_eq!(tensor.get(&[1]), Some(&2));
    assert_eq!(tensor.get(&[2]), Some(&3));
}

#[test]
fn test_new_empty_tensor_with_zero_dim() {
    let data: Vec<i32> = vec![];
    let shape = vec![0];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();
    assert!(tensor.is_empty());
    assert_eq!(tensor.shape(), &shape);
}

#[test]
fn test_new_empty_tensor_with_non_zero_dim() {
    let data: Vec<i32> = vec![];
    let shape = vec![5, 0, 2];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();
    assert!(tensor.is_empty());
    assert_eq!(tensor.shape(), &shape);
}

#[test]
fn test_new_3d_tensor() {
    let data = (0..24).collect::<Vec<_>>();
    let shape = vec![2, 3, 4];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();
    assert_eq!(tensor.as_slice(), &data);
    assert_eq!(tensor.shape(), &shape);
    // Test strides via get
    assert_eq!(tensor.get(&[0, 0, 0]), Some(&0));
    assert_eq!(tensor.get(&[1, 1, 1]), Some(&17));
    assert_eq!(tensor.get(&[1, 2, 3]), Some(&23));
}
