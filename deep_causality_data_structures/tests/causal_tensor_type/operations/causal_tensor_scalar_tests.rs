/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::CausalTensor;

#[test]
fn test_tensor_add_scalar() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result = &tensor + 10; // Test &Tensor + Scalar
    assert_eq!(result.as_slice(), &[11, 12, 13]);

    let result2 = tensor + 10; // Test Tensor + Scalar
    assert_eq!(result2.as_slice(), &[11, 12, 13]);
}

#[test]
fn test_scalar_add_tensor() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result: CausalTensor<i32> = 10 + &tensor; // Test Scalar + &Tensor
    assert_eq!(result.as_slice(), &[11, 12, 13]);

    let result2 = 10 + tensor; // Test Scalar + Tensor
    assert_eq!(result2.as_slice(), &[11, 12, 13]);
}

#[test]
fn test_tensor_add_assign_scalar() {
    let mut tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    tensor += 10;
    assert_eq!(tensor.as_slice(), &[11, 12, 13]);
}

#[test]
fn test_tensor_sub_scalar() {
    let tensor = CausalTensor::new(vec![11, 12, 13], vec![3]).unwrap();
    let result = &tensor - 10;
    assert_eq!(result.as_slice(), &[1, 2, 3]);

    let result2 = tensor - 10;
    assert_eq!(result2.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_scalar_sub_tensor() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result: CausalTensor<i32> = 10 - &tensor;
    assert_eq!(result.as_slice(), &[9, 8, 7]);

    let result2 = 10 - tensor;
    assert_eq!(result2.as_slice(), &[9, 8, 7]);
}

#[test]
fn test_tensor_sub_assign_scalar() {
    let mut tensor = CausalTensor::new(vec![11, 12, 13], vec![3]).unwrap();
    tensor -= 10;
    assert_eq!(tensor.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_tensor_mul_scalar() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result: CausalTensor<i32> = &tensor * 10;
    assert_eq!(result.as_slice(), &[10, 20, 30]);

    let result2 = tensor * 10;
    assert_eq!(result2.as_slice(), &[10, 20, 30]);
}

#[test]
fn test_scalar_mul_tensor() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let result: CausalTensor<i32> = 10 * &tensor;
    assert_eq!(result.as_slice(), &[10, 20, 30]);

    let result2 = 10 * tensor;
    assert_eq!(result2.as_slice(), &[10, 20, 30]);
}

#[test]
fn test_tensor_mul_assign_scalar() {
    let mut tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    tensor *= 10;
    assert_eq!(tensor.as_slice(), &[10, 20, 30]);
}

#[test]
fn test_tensor_div_scalar() {
    let tensor = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    let result = &tensor / 10;
    assert_eq!(result.as_slice(), &[1, 2, 3]);

    let result2 = tensor / 10;
    assert_eq!(result2.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_scalar_div_tensor() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![2, 5, 10], vec![3]).unwrap();
    let result: CausalTensor<i32> = 100 / &tensor;
    assert_eq!(result.as_slice(), &[50, 20, 10]);

    let result2 = 100 / tensor;
    assert_eq!(result2.as_slice(), &[50, 20, 10]);
}

#[test]
fn test_tensor_div_assign_scalar() {
    let mut tensor = CausalTensor::new(vec![10, 20, 30], vec![3]).unwrap();
    tensor /= 10;
    assert_eq!(tensor.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_ops_with_f64() {
    let mut tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3]).unwrap();

    let add_res = &tensor + 0.5;
    assert_eq!(add_res.as_slice(), &[1.5, 2.5, 3.5]);

    let sub_res = &tensor - 0.5;
    assert_eq!(sub_res.as_slice(), &[0.5, 1.5, 2.5]);

    let mul_res = &tensor * 2.0;
    assert_eq!(mul_res.as_slice(), &[2.0, 4.0, 6.0]);

    let div_res = &tensor / 2.0;
    assert_eq!(div_res.as_slice(), &[0.5, 1.0, 1.5]);

    tensor += 1.0;
    assert_eq!(tensor.as_slice(), &[2.0, 3.0, 4.0]);
    tensor -= 1.0;
    assert_eq!(tensor.as_slice(), &[1.0, 2.0, 3.0]);
    tensor *= 2.0;
    assert_eq!(tensor.as_slice(), &[2.0, 4.0, 6.0]);
    tensor /= 2.0;
    assert_eq!(tensor.as_slice(), &[1.0, 2.0, 3.0]);
}
