/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

// Helper for comparing tensor floats with a tolerance
fn assert_tensor_approx_eq(a: &CausalTensor<f64>, b: &CausalTensor<f64>, epsilon: f64) {
    assert_eq!(a.shape(), b.shape(), "Tensor shapes do not match");
    for (val_a, val_b) in a.as_slice().iter().zip(b.as_slice().iter()) {
        assert_approx_eq(*val_a, *val_b, epsilon);
    }
}

#[test]
fn test_cholesky_decomposition_success() {
    let data = vec![25.0, 15.0, -5.0, 15.0, 18.0, 0.0, -5.0, 0.0, 11.0];
    let a = CausalTensor::new(data, vec![3, 3]).unwrap();

    let expected_l_data = vec![5.0, 0.0, 0.0, 3.0, 3.0, 0.0, -1.0, 1.0, 3.0];
    let expected_l = CausalTensor::new(expected_l_data, vec![3, 3]).unwrap();

    let l = a.cholesky_decomposition().unwrap();
    assert_tensor_approx_eq(&l, &expected_l, 1e-9);
}

#[test]
fn test_cholesky_decomposition_not_positive_definite() {
    // This matrix has eigenvalues 3 and -1, so it's not positive-definite
    let data = vec![1.0, 2.0, 2.0, 1.0];
    let a = CausalTensor::new(data, vec![2, 2]).unwrap();

    let result = a.cholesky_decomposition();
    assert!(matches!(result, Err(CausalTensorError::SingularMatrix)));
}

#[test]
fn test_cholesky_decomposition_non_square() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let a = CausalTensor::new(data, vec![2, 3]).unwrap();

    let result = a.cholesky_decomposition();
    assert!(matches!(result, Err(CausalTensorError::ShapeMismatch)));
}

#[test]
fn test_cholesky_decomposition_non_2d() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let a = CausalTensor::new(data, vec![4]).unwrap(); // 1D tensor

    let result = a.cholesky_decomposition();
    assert!(matches!(result, Err(CausalTensorError::DimensionMismatch)));
}

#[test]
fn test_solve_least_squares_cholsky_success() {
    // Example from linear regression: y = 1.3*x + 0.8
    // for points (0,1), (1,2), (2,3), (3,5)
    let a_data = vec![0.0, 1.0, 1.0, 1.0, 2.0, 1.0, 3.0, 1.0];
    let a = CausalTensor::new(a_data, vec![4, 2]).unwrap();

    let b_data = vec![1.0, 2.0, 3.0, 5.0];
    let b = CausalTensor::new(b_data, vec![4, 1]).unwrap();

    // The known solution
    let expected_x_data = vec![1.3, 0.8];
    let expected_x = CausalTensor::new(expected_x_data, vec![2, 1]).unwrap();

    let x = CausalTensor::solve_least_squares_cholsky(&a, &b).unwrap();

    assert_tensor_approx_eq(&x, &expected_x, 1e-9);
}

#[test]
fn test_solve_least_squares_cholsky_non_2d_a() {
    let a_data = vec![1.0, 2.0, 3.0, 4.0];
    let a = CausalTensor::new(a_data, vec![4]).unwrap(); // 1D

    let b_data = vec![1.0, 2.0, 3.0, 4.0];
    let b = CausalTensor::new(b_data, vec![4, 1]).unwrap();

    let result = CausalTensor::solve_least_squares_cholsky(&a, &b);
    assert!(matches!(result, Err(CausalTensorError::DimensionMismatch)));
}

#[test]
fn test_solve_least_squares_cholsky_non_2d_b() {
    let a_data = vec![1.0, 2.0, 3.0, 4.0];
    let a = CausalTensor::new(a_data, vec![2, 2]).unwrap();

    let b_data = vec![1.0, 2.0];
    let b = CausalTensor::new(b_data, vec![2]).unwrap(); // 1D

    let result = CausalTensor::solve_least_squares_cholsky(&a, &b);
    assert!(matches!(result, Err(CausalTensorError::DimensionMismatch)));
}

#[test]
fn test_solve_least_squares_cholsky_shape_mismatch_rows() {
    // a has 4 rows, b has 3 rows
    let a_data = vec![0.0, 1.0, 1.0, 1.0, 2.0, 1.0, 3.0, 1.0];
    let a = CausalTensor::new(a_data, vec![4, 2]).unwrap();

    let b_data = vec![1.0, 2.0, 3.0];
    let b = CausalTensor::new(b_data, vec![3, 1]).unwrap();

    let result = CausalTensor::solve_least_squares_cholsky(&a, &b);
    assert!(matches!(result, Err(CausalTensorError::ShapeMismatch)));
}

#[test]
fn test_solve_least_squares_cholsky_b_not_column_vector() {
    // b is 4x2 instead of 4x1
    let a_data = vec![0.0, 1.0, 1.0, 1.0, 2.0, 1.0, 3.0, 1.0];
    let a = CausalTensor::new(a_data, vec![4, 2]).unwrap();

    let b_data = vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 5.0, 5.0];
    let b = CausalTensor::new(b_data, vec![4, 2]).unwrap();

    let result = CausalTensor::solve_least_squares_cholsky(&a, &b);
    assert!(matches!(result, Err(CausalTensorError::ShapeMismatch)));
}

#[test]
fn test_solve_least_squares_cholsky_singular_matrix() {
    // Columns of A are linearly dependent, so A^T A will be singular
    let a_data = vec![1.0, 2.0, 1.0, 2.0, 1.0, 2.0];
    let a = CausalTensor::new(a_data, vec![3, 2]).unwrap();

    let b_data = vec![3.0, 3.0, 3.0];
    let b = CausalTensor::new(b_data, vec![3, 1]).unwrap();

    // A^T A = [[3, 6], [6, 12]], which is singular.
    let result = CausalTensor::solve_least_squares_cholsky(&a, &b);
    assert!(matches!(result, Err(CausalTensorError::SingularMatrix)));
}

#[test]
fn test_cholesky_decomposition_another_success() {
    // Another valid positive-definite matrix
    let data = vec![4.0, 12.0, -16.0, 12.0, 37.0, -43.0, -16.0, -43.0, 98.0];
    let a = CausalTensor::new(data, vec![3, 3]).unwrap();

    let expected_l_data = vec![2.0, 0.0, 0.0, 6.0, 1.0, 0.0, -8.0, 5.0, 3.0];
    let expected_l = CausalTensor::new(expected_l_data, vec![3, 3]).unwrap();

    let l = a.cholesky_decomposition().unwrap();
    assert_tensor_approx_eq(&l, &expected_l, 1e-9);
}

#[test]
fn test_cholesky_decomposition_1x1_success() {
    let a = CausalTensor::new(vec![25.0], vec![1, 1]).unwrap();
    let expected_l = CausalTensor::new(vec![5.0], vec![1, 1]).unwrap();
    let l = a.cholesky_decomposition().unwrap();
    assert_tensor_approx_eq(&l, &expected_l, 1e-9);
}

#[test]
fn test_cholesky_decomposition_1x1_not_positive() {
    let a = CausalTensor::new(vec![-25.0], vec![1, 1]).unwrap();
    let result = a.cholesky_decomposition();
    assert!(matches!(result, Err(CausalTensorError::SingularMatrix)));
}
