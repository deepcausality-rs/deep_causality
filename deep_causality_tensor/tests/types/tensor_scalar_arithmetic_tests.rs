/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for tensor + scalar arithmetic operations (tensor {+,-,*,/} scalar)
//! and compound assignment operators (+=, -=, *=, /=)

use deep_causality_tensor::InternalCpuTensor;

#[test]
fn test_tensor_add_scalar_ref() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let result = &tensor + 10.0;
    assert_eq!(result.as_slice(), &[11.0, 12.0, 13.0, 14.0]);
}

#[test]
fn test_tensor_add_scalar_owned() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let result = tensor + 10.0;
    assert_eq!(result.as_slice(), &[11.0, 12.0, 13.0, 14.0]);
}

#[test]
fn test_tensor_add_assign() {
    let mut tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    tensor += 10.0;
    assert_eq!(tensor.as_slice(), &[11.0, 12.0, 13.0, 14.0]);
}

#[test]
fn test_tensor_sub_scalar_ref() {
    let tensor = InternalCpuTensor::<f64>::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    let result = &tensor - 5.0;
    assert_eq!(result.as_slice(), &[5.0, 15.0, 25.0, 35.0]);
}

#[test]
fn test_tensor_sub_scalar_owned() {
    let tensor = InternalCpuTensor::<f64>::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    let result = tensor - 5.0;
    assert_eq!(result.as_slice(), &[5.0, 15.0, 25.0, 35.0]);
}

#[test]
fn test_tensor_sub_assign() {
    let mut tensor =
        InternalCpuTensor::<f64>::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    tensor -= 5.0;
    assert_eq!(tensor.as_slice(), &[5.0, 15.0, 25.0, 35.0]);
}

#[test]
fn test_tensor_mul_scalar_ref() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let result = &tensor * 2.0;
    assert_eq!(result.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
}

#[test]
fn test_tensor_mul_scalar_owned() {
    let tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let result = tensor * 2.0;
    assert_eq!(result.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
}

#[test]
fn test_tensor_mul_assign() {
    let mut tensor = InternalCpuTensor::<f64>::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    tensor *= 2.0;
    assert_eq!(tensor.as_slice(), &[2.0, 4.0, 6.0, 8.0]);
}

#[test]
fn test_tensor_div_scalar_ref() {
    let tensor = InternalCpuTensor::<f64>::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    let result = &tensor / 10.0;
    assert_eq!(result.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_tensor_div_scalar_owned() {
    let tensor = InternalCpuTensor::<f64>::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    let result = tensor / 10.0;
    assert_eq!(result.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_tensor_div_assign() {
    let mut tensor =
        InternalCpuTensor::<f64>::new(vec![10.0, 20.0, 30.0, 40.0], vec![2, 2]).unwrap();
    tensor /= 10.0;
    assert_eq!(tensor.as_slice(), &[1.0, 2.0, 3.0, 4.0]);
}

// Integer type tests
#[test]
fn test_tensor_scalar_i32() {
    let tensor = InternalCpuTensor::<i32>::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    assert_eq!((&tensor + 10).as_slice(), &[11, 12, 13, 14]);
    assert_eq!((&tensor - 1).as_slice(), &[0, 1, 2, 3]);
    assert_eq!((&tensor * 2).as_slice(), &[2, 4, 6, 8]);
    assert_eq!((&tensor / 1).as_slice(), &[1, 2, 3, 4]);
}

#[test]
fn test_tensor_scalar_assign_i32() {
    let mut tensor = InternalCpuTensor::<i32>::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    tensor += 10;
    assert_eq!(tensor.as_slice(), &[11, 12, 13, 14]);
    tensor -= 10;
    assert_eq!(tensor.as_slice(), &[1, 2, 3, 4]);
    tensor *= 2;
    assert_eq!(tensor.as_slice(), &[2, 4, 6, 8]);
    tensor /= 2;
    assert_eq!(tensor.as_slice(), &[1, 2, 3, 4]);
}
