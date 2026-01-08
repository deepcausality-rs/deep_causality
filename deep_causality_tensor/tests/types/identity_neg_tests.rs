/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for identity traits (is_zero, is_one) and negation for InternalCpuTensor

use deep_causality_num::{One, Zero};
use deep_causality_tensor::InternalCpuTensor;

// Zero trait tests - is_zero
#[test]
fn test_tensor_is_zero_all_zeros() {
    let tensor = InternalCpuTensor::<f64>::new(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2]).unwrap();
    assert!(tensor.is_zero());
}

#[test]
fn test_tensor_is_zero_not_all_zeros() {
    let tensor = InternalCpuTensor::<f64>::new(vec![0.0, 1.0, 0.0, 0.0], vec![2, 2]).unwrap();
    assert!(!tensor.is_zero());
}

// One trait tests - is_one
#[test]
fn test_tensor_is_one_all_ones() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 1.0, 1.0, 1.0], vec![2, 2]).unwrap();
    assert!(tensor.is_one());
}

#[test]
fn test_tensor_is_one_not_all_ones() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 1.0, 2.0, 1.0], vec![2, 2]).unwrap();
    assert!(!tensor.is_one());
}

// Negation tests
#[test]
fn test_tensor_neg_owned() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, -2.0, 3.0, -4.0], vec![2, 2]).unwrap();
    let result = -tensor;
    assert_eq!(result.as_slice(), &[-1.0, 2.0, -3.0, 4.0]);
}

#[test]
fn test_tensor_neg_ref() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, -2.0, 3.0, -4.0], vec![2, 2]).unwrap();
    let result = -&tensor;
    assert_eq!(result.as_slice(), &[-1.0, 2.0, -3.0, 4.0]);
    // Original unchanged
    assert_eq!(tensor.as_slice(), &[1.0, -2.0, 3.0, -4.0]);
}

#[test]
fn test_tensor_neg_i32() {
    let tensor = InternalCpuTensor::<i32>::new(vec![1, -2, 3, -4], vec![2, 2]).unwrap();
    let result = -tensor;
    assert_eq!(result.as_slice(), &[-1, 2, -3, 4]);
}

// Integer type identity tests
#[test]
fn test_identity_i32() {
    let zeros = InternalCpuTensor::<i32>::new(vec![0, 0, 0, 0], vec![2, 2]).unwrap();
    let ones = InternalCpuTensor::<i32>::new(vec![1, 1, 1, 1], vec![2, 2]).unwrap();
    assert!(zeros.is_zero());
    assert!(ones.is_one());
}
