/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;

#[test]
fn test_row_indices() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

#[test]
fn test_col_indices() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
}

#[test]
fn test_values() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_shape() {
    let matrix: CsrMatrix<f64> = CsrMatrix::with_capacity(5, 10, 0);
    assert_eq!(matrix.shape(), (5, 10));

    let matrix_from_triplets: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
    assert_eq!(matrix_from_triplets.shape(), (2, 3));
}

#[test]
fn test_get_value_at_present() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.get_value_at(0, 0), 1.0);
    assert_eq!(matrix.get_value_at(0, 2), 2.0);
    assert_eq!(matrix.get_value_at(1, 1), 3.0);
}

#[test]
fn test_get_value_at_zero() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.get_value_at(0, 1), 0.0);
    assert_eq!(matrix.get_value_at(1, 0), 0.0);
    assert_eq!(matrix.get_value_at(1, 2), 0.0);
}

#[test]
fn test_get_value_at_out_of_bounds() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
    assert_eq!(matrix.get_value_at(2, 0), 0.0); // Out of row bounds
    assert_eq!(matrix.get_value_at(0, 3), 0.0); // Out of col bounds
    assert_eq!(matrix.get_value_at(3, 3), 0.0); // Out of both bounds
}

#[test]
fn test_get_value_at_empty_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(matrix.get_value_at(0, 0), 0.0);
}
