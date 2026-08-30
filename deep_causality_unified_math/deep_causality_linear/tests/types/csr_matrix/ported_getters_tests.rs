/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `CsrMatrix` read accessors, ported from `deep_causality_sparse`.
//!
//! Covers the three array getters that expose the compressed-sparse-row layout — `row_indices`,
//! `col_indices`, `values` — together with `shape` and the entry lookup `get_value_at`, including
//! the lookup's behaviour on a structural zero, on a position outside the shape, and on the empty
//! matrix.

use deep_causality_linear::CsrMatrix;

#[test]
fn test_row_indices_are_the_cumulative_per_row_counts() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

#[test]
fn test_col_indices_are_ordered_by_row_then_by_column() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
}

#[test]
fn test_values_follow_the_column_indices_position_for_position() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
}

#[test]
fn test_shape_reports_the_declared_dimensions() {
    let matrix: CsrMatrix<f64> = CsrMatrix::with_capacity(5, 10, 0);
    assert_eq!(matrix.shape(), (5, 10));

    let matrix_from_triplets: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
    assert_eq!(matrix_from_triplets.shape(), (2, 3));
}

#[test]
fn test_get_value_at_returns_a_stored_entry() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.get_value_at(0, 0), 1.0);
    assert_eq!(matrix.get_value_at(0, 2), 2.0);
    assert_eq!(matrix.get_value_at(1, 1), 3.0);
}

#[test]
fn test_get_value_at_returns_zero_for_a_structural_zero() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    assert_eq!(matrix.get_value_at(0, 1), 0.0);
    assert_eq!(matrix.get_value_at(1, 0), 0.0);
    assert_eq!(matrix.get_value_at(1, 2), 0.0);
}

#[test]
fn test_get_value_at_returns_zero_outside_the_shape() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
    assert_eq!(matrix.get_value_at(2, 0), 0.0, "past the last row");
    assert_eq!(matrix.get_value_at(0, 3), 0.0, "past the last column");
    assert_eq!(matrix.get_value_at(3, 3), 0.0, "past both");
}

#[test]
fn test_get_value_at_returns_zero_on_the_empty_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(matrix.get_value_at(0, 0), 0.0);
}
