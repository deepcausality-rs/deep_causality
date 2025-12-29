/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;

#[test]
fn test_add_scalar() {
    // Test &Tensor + scalar
    let tensor1 = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result1: CausalTensor<i32> = &tensor1 + 5;
    assert_eq!(result1.as_slice(), &[6, 7, 8]);
    // Ensure original tensor is unchanged
    assert_eq!(tensor1.as_slice(), &[1, 2, 3]);

    // Test Tensor + scalar (consuming)
    let tensor2 = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result2: CausalTensor<i32> = tensor2 + 5;
    assert_eq!(result2.as_slice(), &[6, 7, 8]);

    // Test Tensor += scalar (in-place)
    let mut tensor3 = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    tensor3 += 5;
    assert_eq!(tensor3.as_slice(), &[6, 7, 8]);
}

#[test]
fn test_sub_scalar() {
    // Test &Tensor - scalar
    let tensor1 = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let result1: CausalTensor<i32> = &tensor1 - 5;
    assert_eq!(result1.as_slice(), &[5, 15, 25]);
    // Ensure original tensor is unchanged
    assert_eq!(tensor1.as_slice(), &[10, 20, 30]);

    // Test Tensor - scalar (consuming)
    let tensor2 = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let result2: CausalTensor<i32> = tensor2 - 5;
    assert_eq!(result2.as_slice(), &[5, 15, 25]);

    // Test Tensor -= scalar (in-place)
    let mut tensor3 = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    tensor3 -= 5;
    assert_eq!(tensor3.as_slice(), &[5, 15, 25]);
}

#[test]
fn test_mul_scalar() {
    // Test &Tensor * scalar
    let tensor1 = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result1: CausalTensor<i32> = &tensor1 * 3;
    assert_eq!(result1.as_slice(), &[3, 6, 9]);
    // Ensure original tensor is unchanged
    assert_eq!(tensor1.as_slice(), &[1, 2, 3]);

    // Test Tensor * scalar (consuming)
    let tensor2 = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result2: CausalTensor<i32> = tensor2 * 3;
    assert_eq!(result2.as_slice(), &[3, 6, 9]);

    // Test Tensor *= scalar (in-place)
    let mut tensor3 = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    tensor3 *= 3;
    assert_eq!(tensor3.as_slice(), &[3, 6, 9]);
}

#[test]
fn test_div_scalar() {
    // Test &Tensor / scalar
    let tensor1 = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let result1: CausalTensor<i32> = &tensor1 / 2;
    assert_eq!(result1.as_slice(), &[5, 10, 15]);
    // Ensure original tensor is unchanged
    assert_eq!(tensor1.as_slice(), &[10, 20, 30]);

    // Test Tensor / scalar (consuming)
    let tensor2 = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let result2: CausalTensor<i32> = tensor2 / 2;
    assert_eq!(result2.as_slice(), &[5, 10, 15]);

    // Test Tensor /= scalar (in-place)
    let mut tensor3 = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    tensor3 /= 2;
    assert_eq!(tensor3.as_slice(), &[5, 10, 15]);
}
