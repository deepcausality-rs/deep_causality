/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{CausalTensor, CausalTensorError};

#[test]
fn test_reshape_success() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let reshaped = tensor.reshape(&[3, 2]).unwrap();

    assert_eq!(reshaped.shape(), &[3, 2]);
    assert_eq!(reshaped.as_slice(), &[1, 2, 3, 4, 5, 6]);
    // Indirectly test strides by checking element access
    assert_eq!(reshaped.get(&[0, 0]), Some(&1));
    assert_eq!(reshaped.get(&[0, 1]), Some(&2));
    assert_eq!(reshaped.get(&[1, 0]), Some(&3));
    assert_eq!(reshaped.get(&[2, 1]), Some(&6));
}

#[test]
fn test_reshape_to_vector() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let reshaped = tensor.reshape(&[6]).unwrap();
    assert_eq!(reshaped.shape(), &[6]);
    assert_eq!(reshaped.as_slice(), &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_reshape_shape_mismatch() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let result = tensor.reshape(&[2, 2]);
    assert_eq!(result, Err(CausalTensorError::ShapeMismatch));
}

#[test]
fn test_ravel() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let original_data = tensor.as_slice().to_vec();
    let raveled = tensor.ravel();

    assert_eq!(raveled.shape(), &[6]);
    assert_eq!(raveled.as_slice(), original_data.as_slice());
}

#[test]
fn test_ravel_on_vector() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let raveled = tensor.ravel();
    assert_eq!(raveled.shape(), &[3]);
}

#[test]
fn test_ravel_on_scalar() {
    let tensor = CausalTensor::new(vec![42], vec![]).unwrap();
    let raveled = tensor.ravel();
    // A scalar has len 1, so ravel should produce a vector of len 1.
    assert_eq!(raveled.shape(), &[1]);
    assert_eq!(raveled.as_slice(), &[42]);
}
