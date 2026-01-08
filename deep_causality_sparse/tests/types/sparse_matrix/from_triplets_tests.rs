/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_sparse::{CsrMatrix, SparseMatrixError};

#[test]
fn test_from_triplets_basic_f64() {
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();

    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
    assert_eq!(matrix.shape(), (2, 3));
}

#[test]
fn test_from_triplets_duplicates_f64() {
    let triplets = vec![(0, 0, 1.0), (0, 0, 0.5), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.values(), &vec![1.5, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 1, 2]);
    assert_eq!(matrix.shape(), (2, 2));
}

#[test]
fn test_from_triplets_zero_after_sum_f64() {
    let triplets = vec![(0, 0, 1.0), (0, 0, -1.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert_eq!(matrix.values(), &vec![3.0]);
    assert_eq!(matrix.col_indices(), &vec![1]);
    assert_eq!(matrix.row_indices(), &vec![0, 0, 1]); // Row 0 is empty after sum
    assert_eq!(matrix.shape(), (2, 2));
}

#[test]
fn test_from_triplets_hodge_case() {
    let hodge_val_debug = 2.912903333333333_f64;
    let triplets = vec![(0, 0, hodge_val_debug)];
    let matrix = CsrMatrix::from_triplets(4, 1, &triplets).unwrap(); // Shape (4,1)

    // Expected values: a single value corresponding to hodge_val_debug
    assert_eq!(matrix.values().len(), 1);
    assert_eq!(matrix.values()[0], hodge_val_debug);

    // Expected col_indices: [0]
    assert_eq!(matrix.col_indices(), &vec![0]);

    // Expected row_indices: [0, 1, 1, 1, 1] (1 non-zero in row 0, others empty)
    assert_eq!(matrix.row_indices(), &vec![0, 1, 1, 1, 1]);
    assert_eq!(matrix.shape(), (4, 1));
}

#[test]
fn test_from_triplets_empty_triplets() {
    let triplets: Vec<(usize, usize, f64)> = vec![];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();

    assert!(matrix.values().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert_eq!(matrix.row_indices(), &vec![0, 0, 0]);
    assert_eq!(matrix.shape(), (2, 2));
}

#[test]
fn test_from_triplets_index_out_of_bounds() {
    let triplets = vec![(0, 0, 1.0), (5, 1, 2.0)]; // Row 5 is out of bounds for 2 rows
    let result = CsrMatrix::from_triplets(2, 2, &triplets);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        deep_causality_sparse::SparseMatrixError::IndexOutOfBounds(5, 2)
    );
}

#[test]
fn test_from_triplets_basic_construction() {
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();
    // Expected:
    // [1.0, 0.0, 2.0]
    // [0.0, 3.0, 0.0]
    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]); // Sorted by row, then col
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]); // First row has 2 elements (0 and 2), second has 1 (1)
}

#[test]
fn test_from_triplets_with_duplicate_entries_summed() {
    let triplets = vec![(0, 0, 1.0), (0, 0, 0.5), (1, 1, 3.0), (0, 1, 1.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    // Expected:
    // [1.5, 1.0]
    // [0.0, 3.0]
    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![1.5, 1.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

#[test]
fn test_from_triplets_with_zero_valued_entries_after_summation() {
    let triplets = vec![(0, 0, 1.0), (0, 0, -1.0), (1, 1, 3.0), (0, 1, 0.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    // Expected:
    // [0.0, 0.0]
    // [0.0, 3.0]
    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![3.0]);
    assert_eq!(matrix.col_indices(), &vec![1]);
    // First row has no non-zeros, so row_ptrs[0] and row_ptrs[1] should be the same
    assert_eq!(matrix.row_indices(), &vec![0, 0, 1]);
}

#[test]
fn test_from_triplets_index_out_of_bounds_row() {
    let triplets = vec![(0, 0, 1.0), (2, 1, 2.0)]; // Row 2 is out of bounds for 2x2
    let err = CsrMatrix::from_triplets(2, 2, &triplets).unwrap_err();
    assert!(matches!(err, SparseMatrixError::IndexOutOfBounds(2, 2)));
}

#[test]
fn test_from_triplets_index_out_of_bounds_col() {
    let triplets = vec![(0, 0, 1.0), (1, 2, 2.0)]; // Col 2 is out of bounds for 2x2
    let err = CsrMatrix::from_triplets(2, 2, &triplets).unwrap_err();
    assert!(matches!(err, SparseMatrixError::IndexOutOfBounds(2, 2)));
}

#[test]
fn test_from_triplets_only_zero_values() {
    let triplets = vec![(0, 0, 0.0), (1, 1, 0.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    assert_eq!(matrix.shape(), (2, 2));
    assert!(matrix.values().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert_eq!(matrix.row_indices(), &vec![0, 0, 0]);
}

#[test]
fn test_from_triplets_mixed_valid_and_zero_values() {
    let triplets = vec![(0, 0, 1.0), (0, 1, 0.0), (1, 1, 2.0)];
    let matrix = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    assert_eq!(matrix.shape(), (2, 2));
    assert_eq!(matrix.values(), &vec![1.0, 2.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 1, 2]);
}

#[test]
fn test_from_triplets_unordered_input() {
    let triplets = vec![(1, 1, 3.0), (0, 0, 1.0), (0, 2, 2.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();
    // Should be sorted internally to: (0,0,1.0), (0,2,2.0), (1,1,3.0)
    assert_eq!(matrix.shape(), (2, 3));
    assert_eq!(matrix.values(), &vec![1.0, 2.0, 3.0]);
    assert_eq!(matrix.col_indices(), &vec![0, 2, 1]);
    assert_eq!(matrix.row_indices(), &vec![0, 2, 3]);
}

// Tests for non-square matrices (these would fail before the fix)
#[test]
fn test_from_triplets_nonsquare_row_out_of_bounds() {
    // 3x10 matrix: row 5 is out of bounds (only 3 rows)
    let triplets = vec![(5, 1, 1.0)];
    let err = CsrMatrix::from_triplets(3, 10, &triplets).unwrap_err();
    // Should report index=5, size=3 (not max'd with cols)
    assert_eq!(err, SparseMatrixError::IndexOutOfBounds(5, 3));
}

#[test]
fn test_from_triplets_nonsquare_col_out_of_bounds() {
    // 10x3 matrix: col 5 is out of bounds (only 3 cols)
    let triplets = vec![(1, 5, 1.0)];
    let err = CsrMatrix::from_triplets(10, 3, &triplets).unwrap_err();
    // Should report index=5, size=3 (not max'd with rows)
    assert_eq!(err, SparseMatrixError::IndexOutOfBounds(5, 3));
}

#[test]
fn test_from_triplets_nonsquare_misleading_case() {
    // 5x100 matrix: row 6 is out of bounds
    // Before fix: would report IndexOutOfBounds(50, 100) - wrong!
    // After fix: should report IndexOutOfBounds(6, 5)
    let triplets = vec![(6, 50, 1.0)];
    let err = CsrMatrix::from_triplets(5, 100, &triplets).unwrap_err();
    assert_eq!(err, SparseMatrixError::IndexOutOfBounds(6, 5));
}
