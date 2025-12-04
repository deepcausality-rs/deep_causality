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

// New tests for Cholesky Decomposition
#[test]
fn test_cholesky_decomposition_2x2_ok() {
    // Symmetric positive-definite matrix
    // A = [[4, 2], [2, 10]]
    let matrix = CausalTensor::new(vec![4.0, 2.0, 2.0, 10.0], vec![2, 2]).unwrap();
    let l = matrix.cholesky_decomposition().unwrap();

    // Expected L = [[2, 0], [1, 3]]
    let expected_l_data = [2.0, 0.0, 1.0, 3.0];
    assert_eq!(l.shape(), &[2, 2]);
    for (i, &val) in expected_l_data.iter().enumerate() {
        assert_approx_eq(l.as_slice()[i], val, 1e-9);
    }
}

#[test]
fn test_cholesky_decomposition_3x3_ok() {
    // Symmetric positive-definite matrix
    // A = [[25, 15, -5], [15, 18, 0], [-5, 0, 11]]
    let matrix = CausalTensor::new(
        vec![
            25.0, 15.0, -5.0, //
            15.0, 18.0, 0.0, //
            -5.0, 0.0, 11.0, //
        ],
        vec![3, 3],
    )
    .unwrap();
    let l = matrix.cholesky_decomposition().unwrap();

    // Expected L = [[5, 0, 0], [3, 3, 0], [-1, 1, 3]]
    let expected_l_data = vec![
        5.0, 0.0, 0.0, //
        3.0, 3.0, 0.0, //
        -1.0, 1.0, 3.0, //
    ];
    assert_eq!(l.shape(), &[3, 3]);
    for (i, &val) in expected_l_data.iter().enumerate() {
        assert_approx_eq(l.as_slice()[i], val, 1e-9);
    }
}

#[test]
fn test_cholesky_decomposition_non_2d_tensor_error() {
    let tensor_1d = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![4]).unwrap(); // 1D tensor
    let err = tensor_1d.cholesky_decomposition().unwrap_err();
    assert_eq!(err, CausalTensorError::DimensionMismatch);

    let tensor_3d = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap(); // 3D tensor
    let err_3d = tensor_3d.cholesky_decomposition().unwrap_err();
    assert_eq!(err_3d, CausalTensorError::DimensionMismatch);
}

#[test]
fn test_cholesky_decomposition_non_square_matrix_error() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], vec![2, 3]).unwrap();
    let err = tensor.cholesky_decomposition().unwrap_err();
    assert_eq!(err, CausalTensorError::ShapeMismatch);
}

#[test]
fn test_cholesky_decomposition_singular_matrix_error() {
    // Not positive-definite, will result in sqrt of negative or zero diagonal element
    let singular_matrix_1 = CausalTensor::new(vec![1.0, 1.0, 1.0, 1.0], vec![2, 2]).unwrap(); // Det = 0
    let err_1 = singular_matrix_1.cholesky_decomposition().unwrap_err();
    assert_eq!(err_1, CausalTensorError::SingularMatrix);

    let singular_matrix_2 = CausalTensor::new(vec![1.0, 2.0, 2.0, 3.0], vec![2, 2]).unwrap(); // Not positive definite, det = -1
    let err_2 = singular_matrix_2.cholesky_decomposition().unwrap_err();
    assert_eq!(err_2, CausalTensorError::SingularMatrix);

    let singular_matrix_3 = CausalTensor::new(vec![0.0, 0.0, 0.0, 0.0], vec![2, 2]).unwrap(); // All zeros
    let err_3 = singular_matrix_3.cholesky_decomposition().unwrap_err();
    assert_eq!(err_3, CausalTensorError::SingularMatrix);
}

// New tests for Solve Least Squares with Cholesky
#[test]
fn test_solve_least_squares_cholsky_2x1_ok() {
    // Simple case: A = [[1], [2]], b = [[3], [4]]
    // A^T A = [1 2] * [[1], [2]] = [5]
    // A^T b = [1 2] * [[3], [4]] = [11]
    // 5x = 11 => x = 2.2
    let a = CausalTensor::new(vec![1.0, 2.0], vec![2, 1]).unwrap();
    let b = CausalTensor::new(vec![3.0, 4.0], vec![2, 1]).unwrap();
    let x = CausalTensor::solve_least_squares_cholsky(&a, &b).unwrap();
    assert_eq!(x.shape(), &[1, 1]);
    assert_approx_eq(x.as_slice()[0], 2.2, 1e-9);
}

#[test]
fn test_solve_least_squares_cholsky_2x2_ok() {
    // A = [[1, 1], [1, 2]], b = [[3], [4]]
    // (A^T A)x = A^T b
    // A^T A = [[2, 3], [3, 5]]
    // A^T b = [[7], [11]]
    // Solution x = [[2], [1]]
    let a = CausalTensor::new(vec![1.0, 1.0, 1.0, 2.0], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![3.0, 4.0], vec![2, 1]).unwrap();
    let x = CausalTensor::solve_least_squares_cholsky(&a, &b).unwrap();
    assert_eq!(x.shape(), &[2, 1]);
    assert_approx_eq(x.as_slice()[0], 2.0, 1e-9);
    assert_approx_eq(x.as_slice()[1], 1.0, 1e-9);
}

#[test]
fn test_solve_least_squares_cholsky_3x2_ok() {
    // Overdetermined system
    // A = [[1, 0], [0, 1], [1, 1]], b = [[1], [2], [3]]
    // Solution x = [[0.5], [2.5]]
    let a = CausalTensor::new(vec![1.0, 0.0, 0.0, 1.0, 1.0, 1.0], vec![3, 2]).unwrap();
    let b = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3, 1]).unwrap();
    let x = CausalTensor::solve_least_squares_cholsky(&a, &b).unwrap();
    assert_eq!(x.shape(), &[2, 1]);
    assert_approx_eq(x.as_slice()[0], 0.5, 1e-9);
    assert_approx_eq(x.as_slice()[1], 2.5, 1e-9);
}

#[test]
fn test_solve_least_squares_cholsky_a_non_2d_error() {
    let a_1d = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let b = CausalTensor::new(vec![1.0, 2.0], vec![2, 1]).unwrap();
    let err = CausalTensor::solve_least_squares_cholsky(&a_1d, &b).unwrap_err();
    assert_eq!(err, CausalTensorError::DimensionMismatch);

    let a_3d = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();
    let err_3d = CausalTensor::solve_least_squares_cholsky(&a_3d, &b).unwrap_err();
    assert_eq!(err_3d, CausalTensorError::DimensionMismatch);
}

#[test]
fn test_solve_least_squares_cholsky_b_non_2d_error() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let b_1d = CausalTensor::new(vec![1.0, 2.0], vec![2]).unwrap();
    let err = CausalTensor::solve_least_squares_cholsky(&a, &b_1d).unwrap_err();
    assert_eq!(err, CausalTensorError::DimensionMismatch);

    let b_3d = CausalTensor::new(vec![1.0; 8], vec![2, 2, 2]).unwrap();
    let err_3d = CausalTensor::solve_least_squares_cholsky(&a, &b_3d).unwrap_err();
    assert_eq!(err_3d, CausalTensorError::DimensionMismatch);
}

#[test]
fn test_solve_least_squares_cholsky_b_shape_mismatch_error() {
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    // b is not a column vector
    let b_row = CausalTensor::new(vec![1.0, 2.0], vec![1, 2]).unwrap();
    let err_row = CausalTensor::solve_least_squares_cholsky(&a, &b_row).unwrap_err();
    assert_eq!(err_row, CausalTensorError::ShapeMismatch);

    // b has wrong number of rows
    let b_wrong_rows = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3, 1]).unwrap();
    let err_wrong_rows = CausalTensor::solve_least_squares_cholsky(&a, &b_wrong_rows).unwrap_err();
    assert_eq!(err_wrong_rows, CausalTensorError::ShapeMismatch);
}

#[test]
fn test_solve_least_squares_cholsky_singular_matrix_error() {
    // A^T A will be singular
    let a = CausalTensor::new(vec![1.0, 1.0, 1.0, 1.0], vec![2, 2]).unwrap(); // Results in A^T A = [[2,2],[2,2]] (singular)
    let b = CausalTensor::new(vec![1.0, 2.0], vec![2, 1]).unwrap();
    let err = CausalTensor::solve_least_squares_cholsky(&a, &b).unwrap_err();
    assert_eq!(err, CausalTensorError::SingularMatrix);
}
