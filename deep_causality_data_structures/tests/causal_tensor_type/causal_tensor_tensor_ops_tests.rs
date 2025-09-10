/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::{CausalTensor, causal_tensor_type::error::CausalTensorError};

#[test]
fn test_add_tensors_same_shape() {
    let a = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let b = CausalTensor::new(vec![4, 5, 6], vec![3]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[5, 7, 9]);
    assert_eq!(result.shape(), &[3]);
}

#[test]
fn test_add_tensors_broadcast_row() {
    // Broadcast row vector
    let a = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let b = CausalTensor::new(vec![10, 20, 30], vec![1, 3]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[11, 22, 33, 14, 25, 36]);
    assert_eq!(result.shape(), &[2, 3]);
}

#[test]
fn test_add_tensors_broadcast_col() {
    // Broadcast column vector
    let a = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let b = CausalTensor::new(vec![10, 20], vec![2, 1]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[11, 12, 13, 24, 25, 26]);
    assert_eq!(result.shape(), &[2, 3]);
}

#[test]
fn test_add_tensors_broadcast_scalar() {
    // Broadcast scalar tensor
    let a = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let b = CausalTensor::new(vec![10], vec![]).unwrap();
    let result = (&a + &b).unwrap();
    assert_eq!(result.as_slice(), &[11, 12, 13]);
}

#[test]
fn test_add_tensors_shape_mismatch() {
    let a = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result = &a + &b;
    assert_eq!(result, Err(CausalTensorError::ShapeMismatch));
}

#[test]
fn test_sub_tensors() {
    let a = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let b = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result = (&a - &b).unwrap();
    assert_eq!(result.as_slice(), &[9, 18, 27]);
}

#[test]
fn test_mul_tensors() {
    let a = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let b = CausalTensor::new(vec![4, 5, 6], vec![3]).unwrap();
    let result = (&a * &b).unwrap();
    assert_eq!(result.as_slice(), &[4, 10, 18]);
}

#[test]
fn test_div_tensors() {
    let a = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let b = CausalTensor::new(vec![2, 5, 10], vec![3]).unwrap();
    let result = (&a / &b).unwrap();
    assert_eq!(result.as_slice(), &[5, 4, 3]);
}

#[test]
fn test_owned_and_borrowed_variants() {
    let a = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3, 4], vec![2]).unwrap();

    // &a + &b
    let res1 = (&a + &b).unwrap();
    assert_eq!(res1.as_slice(), &[4, 6]);

    // a + &b
    let res2 = (a.clone() + &b).unwrap();
    assert_eq!(res2.as_slice(), &[4, 6]);

    // &a + b
    let res3 = (&a + b.clone()).unwrap();
    assert_eq!(res3.as_slice(), &[4, 6]);

    // a + b
    let res4 = (a + b).unwrap();
    assert_eq!(res4.as_slice(), &[4, 6]);
}
