/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::RealField;
use deep_causality_tensor::{CausalTensor, CausalTensorError, Tensor};
use std::fmt::Display;

// Helper for comparing floats with a tolerance
fn assert_approx_eq<T>(a: T, b: T, epsilon: T)
where
    T: Display + RealField,
{
    let diff = if a > b { a - b } else { b - a };
    assert!(diff < epsilon, "{} is not approximately equal to {}", a, b);
}

#[test]
fn test_inverse_1x1_matrix() {
    let tensor = CausalTensor::new(vec![2.0f64], vec![1, 1]).unwrap();
    let inverse = tensor.inverse().unwrap();
    assert_eq!(inverse.shape(), &[1, 1]);
    assert_approx_eq(inverse.as_slice()[0], 0.5f64, 1e-9);

    let singular = CausalTensor::new(vec![0.0f64], vec![1, 1]).unwrap();
    let err = singular.inverse().unwrap_err();
    assert_eq!(err, CausalTensorError::SingularMatrix);
}

#[test]
fn test_inverse_2x2_matrix() {
    let tensor = CausalTensor::new(vec![4.0, 7.0, 2.0, 6.0], vec![2, 2]).unwrap();
    // Inverse of [[4, 7], [2, 6]] is (1/10) * [[6, -7], [-2, 4]] = [[0.6, -0.7], [-0.2, 0.4]]
    let expected_inverse = [0.6, -0.7, -0.2, 0.4];

    let inverse = tensor.inverse().unwrap();
    assert_eq!(inverse.shape(), &[2, 2]);
    for (i, expected_val) in expected_inverse.iter().enumerate() {
        assert_approx_eq(inverse.as_slice()[i], *expected_val, 1e-9);
    }

    // Singular 2x2 matrix
    let singular = CausalTensor::new(vec![1.0, 2.0, 2.0, 4.0], vec![2, 2]).unwrap(); // Det = 1*4 - 2*2 = 0
    let err = singular.inverse().unwrap_err();
    assert_eq!(err, CausalTensorError::SingularMatrix);
}

#[test]
fn test_inverse_3x3_matrix() {
    let tensor = CausalTensor::new(
        vec![
            1.0, 2.0, 3.0, //
            0.0, 1.0, 4.0, //
            5.0, 6.0, 0.0, //
        ],
        vec![3, 3],
    )
    .unwrap();
    // Known inverse from wolframalpha or other tools for this matrix
    // [ -24,  18,   5]
    // [  20, -15,  -4]
    // [  -5,   4,   1]
    let expected_inverse = vec![
        -24.0, 18.0, 5.0, //
        20.0, -15.0, -4.0, //
        -5.0, 4.0, 1.0, //
    ];

    let inverse = tensor.inverse().unwrap();
    assert_eq!(inverse.shape(), &[3, 3]);
    for (i, expected_val) in expected_inverse.iter().enumerate() {
        assert_approx_eq(inverse.as_slice()[i], *expected_val, 1e-9);
    }

    // Singular 3x3 matrix (row 2 is sum of row 0 and row 1)
    let singular = CausalTensor::new(
        vec![
            1.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, //
            1.0, 1.0, 0.0, //
        ],
        vec![3, 3],
    )
    .unwrap();
    let err = singular.inverse().unwrap_err();
    assert_eq!(err, CausalTensorError::SingularMatrix);
}

#[test]
fn test_inverse_non_square_matrix_error() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let err = tensor.inverse().unwrap_err();
    assert_eq!(err, CausalTensorError::ShapeMismatch);
}

#[test]
fn test_inverse_non_2d_tensor_error() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap(); // 1D tensor
    let err = tensor.inverse().unwrap_err();
    assert_eq!(err, CausalTensorError::DimensionMismatch);

    let tensor_3d = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap(); // 3D tensor
    let err_3d = tensor_3d.inverse().unwrap_err();
    assert_eq!(err_3d, CausalTensorError::DimensionMismatch);
}
