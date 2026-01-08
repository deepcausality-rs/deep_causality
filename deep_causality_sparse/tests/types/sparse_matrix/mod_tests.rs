/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;

#[test]
fn test_new_matrix_is_empty() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(matrix.shape(), (0, 0));
    assert!(matrix.row_indices().is_empty());
    assert!(matrix.col_indices().is_empty());
    assert!(matrix.values().is_empty());
}

#[test]
fn test_with_capacity_initializes_correctly() {
    let rows = 5;
    let cols = 10;
    let capacity = 20;
    let matrix: CsrMatrix<f64> = CsrMatrix::with_capacity(rows, cols, capacity);
    assert_eq!(matrix.shape(), (rows, cols));
    assert_eq!(matrix.row_indices().len(), rows + 1);
    assert_eq!(matrix.row_indices(), &vec![0; rows + 1]); // All zeros initially
    assert!(matrix.col_indices().capacity() >= capacity);
    assert!(matrix.values().capacity() >= capacity);
    assert!(matrix.col_indices().is_empty());
    assert!(matrix.values().is_empty());
}
