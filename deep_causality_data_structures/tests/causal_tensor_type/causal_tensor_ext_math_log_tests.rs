/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_data_structures::{CausalTensor, CausalTensorLogMathExt};

// CausalTensorLogMathExt<f32> tests
#[test]
fn test_log_nat_f32_empty() {
    let tensor = CausalTensor::<f32>::new(vec![], vec![0]).unwrap();
    assert!(tensor.is_empty());

    let result = tensor.log_nat();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    assert!(log_tensor.is_empty());
    assert_eq!(log_tensor.shape(), &[0]);
}

#[test]
fn test_log_nat_f32_non_empty() {
    let data = vec![1.0, std::f32::consts::E, 0.0, -1.0];
    let shape = vec![2, 2];
    let tensor = CausalTensor::<f32>::new(data, shape).unwrap();

    let result = tensor.log_nat();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    let slice = log_tensor.as_slice();

    assert_eq!(slice[0], 0.0); // ln(1.0)
    assert!((slice[1] - 1.0).abs() < f32::EPSILON); // ln(e)
    assert!(slice[2].is_infinite() && slice[2].is_sign_negative()); // ln(0.0)
    assert!(slice[3].is_nan()); // ln(-1.0)

    assert_eq!(log_tensor.shape(), &[2, 2]);
}

#[test]
fn test_log2_f32_empty() {
    let tensor = CausalTensor::<f32>::new(vec![], vec![0]).unwrap();
    assert!(tensor.is_empty());

    let result = tensor.log2();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    assert!(log_tensor.is_empty());
    assert_eq!(log_tensor.shape(), &[0]);
}

#[test]
fn test_log2_f32_non_empty() {
    let data = vec![2.0, 8.0, 0.0, -1.0];
    let shape = vec![2, 2];
    let tensor = CausalTensor::<f32>::new(data, shape).unwrap();

    let result = tensor.log2();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    let slice = log_tensor.as_slice();

    assert_eq!(slice[0], 1.0); // log2(2.0)
    assert_eq!(slice[1], 3.0); // log2(8.0)
    assert!(slice[2].is_infinite() && slice[2].is_sign_negative()); // log2(0.0)
    assert!(slice[3].is_nan()); // log2(-1.0)

    assert_eq!(log_tensor.shape(), &[2, 2]);
}

#[test]
fn test_log10_f32_empty() {
    let tensor = CausalTensor::<f32>::new(vec![], vec![0]).unwrap();
    assert!(tensor.is_empty());

    let result = tensor.log10();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    assert!(log_tensor.is_empty());
    assert_eq!(log_tensor.shape(), &[0]);
}

#[test]
fn test_log10_f32_non_empty() {
    let data = vec![10.0, 100.0, 0.0, -1.0];
    let shape = vec![2, 2];
    let tensor = CausalTensor::<f32>::new(data, shape).unwrap();

    let result = tensor.log10();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    let slice = log_tensor.as_slice();

    assert_eq!(slice[0], 1.0); // log10(10.0)
    assert_eq!(slice[1], 2.0); // log10(100.0)
    assert!(slice[2].is_infinite() && slice[2].is_sign_negative()); // log10(0.0)
    assert!(slice[3].is_nan()); // log10(-1.0)

    assert_eq!(log_tensor.shape(), &[2, 2]);
}

// CausalTensorLogMathExt<f64> tests
#[test]
fn test_log_nat_f64_empty() {
    let tensor = CausalTensor::<f64>::new(vec![], vec![0]).unwrap();
    assert!(tensor.is_empty());

    let result = tensor.log_nat();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    assert!(log_tensor.is_empty());
    assert_eq!(log_tensor.shape(), &[0]);
}

#[test]
fn test_log_nat_f64_non_empty() {
    let data = vec![1.0, std::f64::consts::E, 0.0, -1.0];
    let shape = vec![2, 2];
    let tensor = CausalTensor::<f64>::new(data, shape).unwrap();

    let result = tensor.log_nat();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    let slice = log_tensor.as_slice();

    assert_eq!(slice[0], 0.0); // ln(1.0)
    assert!((slice[1] - 1.0).abs() < f64::EPSILON); // ln(e)
    assert!(slice[2].is_infinite() && slice[2].is_sign_negative()); // ln(0.0)
    assert!(slice[3].is_nan()); // ln(-1.0)

    assert_eq!(log_tensor.shape(), &[2, 2]);
}

#[test]
fn test_log2_f64_empty() {
    let tensor = CausalTensor::<f64>::new(vec![], vec![0]).unwrap();
    assert!(tensor.is_empty());

    let result = tensor.log2();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    assert!(log_tensor.is_empty());
    assert_eq!(log_tensor.shape(), &[0]);
}

#[test]
fn test_log2_f64_non_empty() {
    let data = vec![2.0, 8.0, 0.0, -1.0];
    let shape = vec![2, 2];
    let tensor = CausalTensor::<f64>::new(data, shape).unwrap();

    let result = tensor.log2();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    let slice = log_tensor.as_slice();

    assert_eq!(slice[0], 1.0); // log2(2.0)
    assert_eq!(slice[1], 3.0); // log2(8.0)
    assert!(slice[2].is_infinite() && slice[2].is_sign_negative()); // log2(0.0)
    assert!(slice[3].is_nan()); // log2(-1.0)

    assert_eq!(log_tensor.shape(), &[2, 2]);
}

#[test]
fn test_log10_f64_empty() {
    let tensor = CausalTensor::<f64>::new(vec![], vec![0]).unwrap();
    assert!(tensor.is_empty());

    let result = tensor.log10();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    assert!(log_tensor.is_empty());
    assert_eq!(log_tensor.shape(), &[0]);
}

#[test]
fn test_log10_f64_non_empty() {
    let data = vec![10.0, 100.0, 0.0, -1.0];
    let shape = vec![2, 2];
    let tensor = CausalTensor::<f64>::new(data, shape).unwrap();

    let result = tensor.log10();
    assert!(result.is_ok());

    let log_tensor = result.unwrap();
    let slice = log_tensor.as_slice();

    assert_eq!(slice[0], 1.0); // log10(10.0)
    assert_eq!(slice[1], 2.0); // log10(100.0)
    assert!(slice[2].is_infinite() && slice[2].is_sign_negative()); // log10(0.0)
    assert!(slice[3].is_nan()); // log10(-1.0)

    assert_eq!(log_tensor.shape(), &[2, 2]);
}
