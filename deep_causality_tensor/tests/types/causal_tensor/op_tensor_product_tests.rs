/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_tensor_product_1d_1d() {
    let a = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3, 4], vec![2]).unwrap();
    let product = a.tensor_product(&b).unwrap();

    assert_eq!(product.shape(), &[2, 2]);
    assert_eq!(product.as_slice(), &[3, 4, 6, 8]);
}

#[test]
fn test_tensor_product_1d_2d() {
    let a = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3, 4, 5, 6], vec![2, 2]).unwrap();
    // a: [1, 2]
    // b: [[3, 4],
    //     [5, 6]]
    // Result shape: [2, 2, 2]
    // Expected:
    // 1 * [[3,4],[5,6]] = [[3,4],[5,6]]
    // 2 * [[3,4],[5,6]] = [[6,8],[10,12]]
    // Flattened: [3, 4, 5, 6, 6, 8, 10, 12]
    let product = a.tensor_product(&b).unwrap();

    assert_eq!(product.shape(), &[2, 2, 2]);
    assert_eq!(product.as_slice(), &[3, 4, 5, 6, 6, 8, 10, 12]);
}

#[test]
fn test_tensor_product_2d_1d() {
    let a = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![5, 6], vec![2]).unwrap();
    // a: [[1, 2],
    //     [3, 4]]
    // b: [5, 6]
    // Result shape: [2, 2, 2]
    // Expected:
    // 1 * [5,6] = [5,6]
    // 2 * [5,6] = [10,12]
    // 3 * [5,6] = [15,18]
    // 4 * [5,6] = [20,24]
    // Flattened: [5, 6, 10, 12, 15, 18, 20, 24]
    let product = a.tensor_product(&b).unwrap();

    assert_eq!(product.shape(), &[2, 2, 2]);
    assert_eq!(product.as_slice(), &[5, 6, 10, 12, 15, 18, 20, 24]);
}

#[test]
fn test_tensor_product_2d_2d() {
    let a = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();
    // a: [[1, 2],
    //     [3, 4]]
    // b: [[5, 6],
    //     [7, 8]]
    // Result shape: [2, 2, 2, 2]
    // Expected:
    // 1 * [[5,6],[7,8]] = [[5,6],[7,8]]
    // 2 * [[5,6],[7,8]] = [[10,12],[14,16]]
    // 3 * [[5,6],[7,8]] = [[15,18],[21,24]]
    // 4 * [[5,6],[7,8]] = [[20,24],[28,32]]
    // Flattened: [5, 6, 7, 8, 10, 12, 14, 16, 15, 18, 21, 24, 20, 24, 28, 32]
    let product = a.tensor_product(&b).unwrap();

    assert_eq!(product.shape(), &[2, 2, 2, 2]);
    assert_eq!(
        product.as_slice(),
        &[5, 6, 7, 8, 10, 12, 14, 16, 15, 18, 21, 24, 20, 24, 28, 32]
    );
}

#[test]
fn test_tensor_product_empty_self() {
    let a: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let b = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let err = a.tensor_product(&b).unwrap_err();
    assert_eq!(err, CausalTensorError::EmptyTensor);
}

#[test]
fn test_tensor_product_empty_rhs() {
    let a = CausalTensor::new(vec![1, 2], vec![2]).unwrap();
    let b: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let err = a.tensor_product(&b).unwrap_err();
    assert_eq!(err, CausalTensorError::EmptyTensor);
}

#[test]
fn test_tensor_product_empty_both() {
    let a: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let b: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let err = a.tensor_product(&b).unwrap_err();
    assert_eq!(err, CausalTensorError::EmptyTensor);
}

#[test]
fn test_tensor_product_scalar_scalar() {
    let a = CausalTensor::new(vec![7], vec![]).unwrap();
    let b = CausalTensor::new(vec![3], vec![]).unwrap();
    let product = a.tensor_product(&b).unwrap();
    assert_eq!(product.shape(), &[] as &[usize]);
    assert_eq!(product.as_slice(), &[21]);
}

#[test]
fn test_tensor_product_scalar_1d() {
    let a = CausalTensor::new(vec![7], vec![]).unwrap();
    let b = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let product = a.tensor_product(&b).unwrap();
    assert_eq!(product.shape(), &[3]);
    assert_eq!(product.as_slice(), &[7, 14, 21]);
}

#[test]
fn test_tensor_product_1d_scalar() {
    let a = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let b = CausalTensor::new(vec![7], vec![]).unwrap();
    let product = a.tensor_product(&b).unwrap();
    assert_eq!(product.shape(), &[3]);
    assert_eq!(product.as_slice(), &[7, 14, 21]);
}

#[test]
fn test_tensor_product_f32() {
    let a = CausalTensor::new(vec![1.0f32, 2.0f32], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3.0f32, 4.0f32], vec![2]).unwrap();
    let product = a.tensor_product(&b).unwrap();
    assert_eq!(product.shape(), &[2, 2]);
    assert_eq!(product.as_slice(), &[3.0, 4.0, 6.0, 8.0]);
}

#[test]
fn test_tensor_product_f64() {
    let a = CausalTensor::new(vec![1.0f64, 2.0f64], vec![2]).unwrap();
    let b = CausalTensor::new(vec![3.0f64, 4.0f64], vec![2]).unwrap();
    let product = a.tensor_product(&b).unwrap();
    assert_eq!(product.shape(), &[2, 2]);
    assert_eq!(product.as_slice(), &[3.0, 4.0, 6.0, 8.0]);
}
