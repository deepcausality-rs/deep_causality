/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Construction of a `CsrMatrix` from `(row, col, value)` triplets.
//!
//! Ported from `deep_causality_sparse`'s `from_triplets_tests.rs`. Covers the three CSR arrays a
//! build produces, the coordinate-list conventions the build applies — sort by `(row, col)`, sum
//! duplicate positions, store no zero — and the bounds check on every triplet.

use deep_causality_linear::{CsrMatrix, LinearError};

#[test]
fn test_from_triplets_builds_the_three_csr_arrays() {
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();

    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
    assert_eq!(matrix.shape(), (2, 3));
}

#[test]
fn test_from_triplets_sums_two_triplets_at_the_same_position() {
    // A coordinate list may name a position more than once, and the value at that position is the
    // sum. One position holds one stored entry: (0, 0) is 1.0 + 0.5.
    let triplets = vec![(0, 0, 1.0), (0, 0, 0.5), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.values(), &vec![1.5, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 1, 2]);
    assert_eq!(matrix.shape(), (2, 2));
}

#[test]
fn test_from_triplets_drops_a_position_whose_duplicates_cancel() {
    // 1.0 + (-1.0) is zero at (0, 0), so row 0 stores nothing and its two row pointers are equal.
    let triplets = vec![(0, 0, 1.0), (0, 0, -1.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.values(), &vec![3.0]);
    assert_eq!(matrix.col_indices(), &vec![1]);
    assert_eq!(matrix.row_indices(), &vec![0, 0, 1]);
    assert_eq!(matrix.shape(), (2, 2));
}

#[test]
fn test_from_triplets_pads_the_row_pointers_of_a_tall_one_column_matrix() {
    let hodge_value = 2.912903333333333_f64;
    let triplets = vec![(0, 0, hodge_value)];
    let matrix = CsrMatrix::from_triplets(4, 1, &triplets).unwrap();

    assert_eq!(matrix.values().len(), 1);
    assert_eq!(matrix.values()[0], hodge_value);
    assert_eq!(matrix.col_indices(), &vec![0]);
    // One entry in row 0; rows 1 through 3 store nothing, so their pointers repeat.
    assert_eq!(matrix.row_indices(), &vec![0, 1, 1, 1, 1]);
    assert_eq!(matrix.shape(), (4, 1));
}

#[test]
fn test_from_triplets_of_nothing_gives_all_zero_row_pointers() {
    let triplets: Vec<(usize, usize, f64)> = vec![];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert!(matrix.values().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert_eq!(matrix.row_indices(), &vec![0, 0, 0]);
    assert_eq!(matrix.shape(), (2, 2));
}

#[test]
fn test_from_triplets_rejects_a_row_far_past_the_row_count() {
    let triplets = vec![(0, 0, 1.0), (5, 1, 2.0)];
    let err = CsrMatrix::from_triplets(2, 2, &triplets).unwrap_err();
    assert_eq!(
        err,
        LinearError::IndexOutOfBounds {
            index: (5, 1),
            shape: (2, 2),
        }
    );
}

#[test]
fn test_from_triplets_builds_a_two_by_three_matrix_with_a_gap() {
    // [1.0, 0.0, 2.0]
    // [0.0, 3.0, 0.0]
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();

    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

#[test]
fn test_from_triplets_sums_duplicates_alongside_a_distinct_entry_in_the_same_row() {
    // [1.5, 1.0]
    // [0.0, 3.0]
    let triplets = vec![(0, 0, 1.0), (0, 0, 0.5), (1, 1, 3.0), (0, 1, 1.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![1.5, 1.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

#[test]
fn test_from_triplets_drops_a_cancelled_position_and_an_explicit_zero_together() {
    // [0.0, 0.0]
    // [0.0, 3.0]
    let triplets = vec![(0, 0, 1.0), (0, 0, -1.0), (1, 1, 3.0), (0, 1, 0.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![3.0]);
    assert_eq!(matrix.col_indices(), &vec![1]);
    // Row 0 stores nothing, so its opening and closing pointers are equal.
    assert_eq!(matrix.row_indices(), &vec![0, 0, 1]);
}

#[test]
fn test_from_triplets_rejects_a_row_one_past_the_row_count() {
    let triplets = vec![(0, 0, 1.0), (2, 1, 2.0)];
    let err = CsrMatrix::from_triplets(2, 2, &triplets).unwrap_err();
    assert_eq!(
        err,
        LinearError::IndexOutOfBounds {
            index: (2, 1),
            shape: (2, 2),
        }
    );
}

#[test]
fn test_from_triplets_rejects_a_column_one_past_the_column_count() {
    let triplets = vec![(0, 0, 1.0), (1, 2, 2.0)];
    let err = CsrMatrix::from_triplets(2, 2, &triplets).unwrap_err();
    assert_eq!(
        err,
        LinearError::IndexOutOfBounds {
            index: (1, 2),
            shape: (2, 2),
        }
    );
}

#[test]
fn test_from_triplets_of_only_zeros_stores_nothing() {
    let triplets = vec![(0, 0, 0.0), (1, 1, 0.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.shape(), (2, 2));
    assert!(matrix.values().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert_eq!(matrix.row_indices(), &vec![0, 0, 0]);
}

#[test]
fn test_from_triplets_stores_the_non_zeros_and_drops_the_zero() {
    let triplets = vec![(0, 0, 1.0), (0, 1, 0.0), (1, 1, 2.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![1.0, 2.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 1, 2]);
}

#[test]
fn test_from_triplets_sorts_unordered_input_by_row_then_column() {
    let triplets = vec![(1, 1, 3.0), (0, 0, 1.0), (0, 2, 2.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();

    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

#[test]
fn test_from_triplets_checks_a_row_against_the_row_count_of_a_wide_matrix() {
    // 3x10: row 5 is past the three rows, and the column is inside the ten columns.
    let triplets = vec![(5, 1, 1.0)];
    let err = CsrMatrix::from_triplets(3, 10, &triplets).unwrap_err();
    assert_eq!(
        err,
        LinearError::IndexOutOfBounds {
            index: (5, 1),
            shape: (3, 10),
        }
    );
}

#[test]
fn test_from_triplets_checks_a_column_against_the_column_count_of_a_tall_matrix() {
    // 10x3: column 5 is past the three columns, and the row is inside the ten rows.
    let triplets = vec![(1, 5, 1.0)];
    let err = CsrMatrix::from_triplets(10, 3, &triplets).unwrap_err();
    assert_eq!(
        err,
        LinearError::IndexOutOfBounds {
            index: (1, 5),
            shape: (10, 3),
        }
    );
}

#[test]
fn test_from_triplets_reports_both_coordinates_and_the_shape_it_checked_them_against() {
    // 5x100: row 6 is the offender and column 50 is inside the shape. The error carries the whole
    // position and the whole shape, so the reader can see which coordinate failed.
    let triplets = vec![(6, 50, 1.0)];
    let err = CsrMatrix::from_triplets(5, 100, &triplets).unwrap_err();
    assert_eq!(
        err,
        LinearError::IndexOutOfBounds {
            index: (6, 50),
            shape: (5, 100),
        }
    );
}
