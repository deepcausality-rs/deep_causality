/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::CausalTensor;

#[test]
fn test_inspectors_on_standard_tensor() {
    let data = vec![1, 2, 3, 4, 5, 6];
    let shape = vec![2, 3];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();

    assert!(!tensor.is_empty());
    assert_eq!(tensor.shape(), shape.as_slice());
    assert_eq!(tensor.num_dim(), 2);
    assert_eq!(tensor.len(), 6);
    assert_eq!(tensor.as_slice(), data.as_slice());
}

#[test]
fn test_inspectors_on_scalar_tensor() {
    let data = vec![42];
    let shape = vec![];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();

    assert!(!tensor.is_empty());
    assert_eq!(tensor.shape(), shape.as_slice());
    assert_eq!(tensor.num_dim(), 0);
    assert_eq!(tensor.len(), 1);
    assert_eq!(tensor.as_slice(), data.as_slice());
}

#[test]
fn test_inspectors_on_empty_tensor() {
    let data: Vec<i32> = vec![];
    let shape = vec![0];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();

    assert!(tensor.is_empty());
    assert_eq!(tensor.shape(), shape.as_slice());
    assert_eq!(tensor.num_dim(), 1);
    assert_eq!(tensor.len(), 0);
    assert_eq!(tensor.as_slice(), data.as_slice());
}

#[test]
fn test_inspectors_on_empty_tensor_multi_dim() {
    let data: Vec<i32> = vec![];
    let shape = vec![2, 0, 3];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();

    assert!(tensor.is_empty());
    assert_eq!(tensor.shape(), shape.as_slice());
    assert_eq!(tensor.num_dim(), 3);
    assert_eq!(tensor.len(), 0);
    assert_eq!(tensor.as_slice(), data.as_slice());
}

#[test]
fn test_inspectors_on_vector() {
    let data = vec![10, 20, 30];
    let shape = vec![3];
    let tensor = CausalTensor::new(data.clone(), shape.clone()).unwrap();

    assert!(!tensor.is_empty());
    assert_eq!(tensor.shape(), shape.as_slice());
    assert_eq!(tensor.num_dim(), 1);
    assert_eq!(tensor.len(), 3);
    assert_eq!(tensor.as_slice(), data.as_slice());
}
