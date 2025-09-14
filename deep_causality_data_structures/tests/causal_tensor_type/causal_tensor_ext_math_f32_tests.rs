/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_data_structures::{CausalTensor, CausalTensorError, CausalTensorMathExt};

const TOLERANCE: f32 = 1e-6;

#[test]
fn test_log_nat() {
    let data = vec![1.0f32, std::f32::consts::E, 10.0f32];
    let tensor = CausalTensor::new(data, vec![3]).unwrap();
    let log_tensor = tensor.log_nat().unwrap();

    let expected = [0.0f32, 1.0f32, std::f32::consts::LN_10];
    log_tensor
        .as_slice()
        .iter()
        .zip(expected.iter())
        .for_each(|(&a, &b)| assert!((a - b).abs() < TOLERANCE));
}

#[test]
fn test_log_nat_special_values() {
    let data = vec![0.0f32, -1.0f32];
    let tensor = CausalTensor::new(data, vec![2]).unwrap();
    let log_tensor = tensor.log_nat().unwrap();
    let slice: &[f32] = log_tensor.as_slice();

    assert!(slice[0].is_infinite() && slice[0].is_sign_negative()); // log(0) = -inf
    assert!(slice[1].is_nan()); // log(-1) = NaN
}

#[test]
fn test_log_nat_empty() {
    let tensor: CausalTensor<f32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = tensor.log_nat().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_log2() {
    let data = vec![1.0f32, 2.0f32, 8.0f32, 16.0f32];
    let tensor = CausalTensor::new(data, vec![4]).unwrap();
    let log_tensor = tensor.log2().unwrap();

    let expected = [0.0f32, 1.0f32, 3.0f32, 4.0f32];
    log_tensor
        .as_slice()
        .iter()
        .zip(expected.iter())
        .for_each(|(&a, &b)| assert!((a - b).abs() < TOLERANCE));
}

#[test]
fn test_log2_special_values() {
    let data = vec![0.0f32, -1.0f32];
    let tensor = CausalTensor::new(data, vec![2]).unwrap();
    let log_tensor = tensor.log2().unwrap();
    let slice: &[f32] = log_tensor.as_slice();

    assert!(slice[0].is_infinite() && slice[0].is_sign_negative()); // log2(0) = -inf
    assert!(slice[1].is_nan()); // log2(-1) = NaN
}

#[test]
fn test_log2_empty() {
    let tensor: CausalTensor<f32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = tensor.log2().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_log10() {
    let data = vec![1.0f32, 10.0f32, 100.0f32];
    let tensor = CausalTensor::new(data, vec![3]).unwrap();
    let log_tensor = tensor.log10().unwrap();

    let expected = [0.0f32, 1.0f32, 2.0f32];
    log_tensor
        .as_slice()
        .iter()
        .zip(expected.iter())
        .for_each(|(&a, &b)| assert!((a - b).abs() < TOLERANCE));
}

#[test]
fn test_log10_special_values() {
    let data = vec![0.0f32, -1.0f32];
    let tensor = CausalTensor::new(data, vec![2]).unwrap();
    let log_tensor = tensor.log10().unwrap();
    let slice: &[f32] = log_tensor.as_slice();

    assert!(slice[0].is_infinite() && slice[0].is_sign_negative()); // log10(0) = -inf
    assert!(slice[1].is_nan()); // log10(-1) = NaN
}

#[test]
fn test_log10_empty() {
    let tensor: CausalTensor<f32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = tensor.log10().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_surd_log2() {
    let data = vec![1.0f32, 2.0f32, 0.0f32, 16.0f32];
    let tensor = CausalTensor::new(data, vec![4]).unwrap();
    let log_tensor = tensor.surd_log2().unwrap();

    let expected = [0.0f32, 1.0f32, 0.0f32, 4.0f32];
    log_tensor
        .as_slice()
        .iter()
        .zip(expected.iter())
        .for_each(|(&a, &b)| assert!((a - b).abs() < TOLERANCE));
}

#[test]
fn test_surd_log2_special_values() {
    let data = vec![0.0f32, -1.0f32];
    let tensor = CausalTensor::new(data, vec![2]).unwrap();
    let log_tensor = tensor.surd_log2().unwrap();
    let slice: &[f32] = log_tensor.as_slice();

    assert_eq!(slice[0], 0.0f32); // surd_log2(0) = 0
    assert!(slice[1].is_nan()); // surd_log2(-1) = NaN
}

#[test]
fn test_surd_log2_empty() {
    let tensor: CausalTensor<f32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let result = tensor.surd_log2().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_safe_div_normal() {
    let a = CausalTensor::new(vec![4.0f32, 9.0f32, 0.0f32], vec![3]).unwrap();
    let b = CausalTensor::new(vec![2.0f32, 3.0f32, 0.0f32], vec![3]).unwrap();
    let result = a.safe_div(&b).unwrap();
    assert_eq!(result.as_slice(), &[2.0f32, 3.0f32, 0.0f32]);
}

#[test]
fn test_safe_div_broadcast() {
    let a = CausalTensor::new(vec![4.0f32, 8.0f32, 12.0f32], vec![3]).unwrap();
    let b_scalar = CausalTensor::new(vec![2.0f32], vec![1]).unwrap();
    let result = a.safe_div(&b_scalar).unwrap();
    assert_eq!(result.as_slice(), &[2.0f32, 4.0f32, 6.0f32]);
}

#[test]
fn test_safe_div_zero_by_zero() {
    let a = CausalTensor::new(vec![0.0f32], vec![1]).unwrap();
    let b = CausalTensor::new(vec![0.0f32], vec![1]).unwrap();
    let result = a.safe_div(&b).unwrap();
    assert_eq!(result.as_slice(), &[0.0f32]);
}

#[test]
fn test_safe_div_by_zero_error() {
    let a = CausalTensor::new(vec![1.0f32], vec![1]).unwrap();
    let b = CausalTensor::new(vec![0.0f32], vec![1]).unwrap();
    let result = a.safe_div(&b);
    assert!(matches!(result, Err(CausalTensorError::DivisionByZero)));
}
