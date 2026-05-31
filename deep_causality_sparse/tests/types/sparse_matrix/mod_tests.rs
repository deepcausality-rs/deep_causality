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

#[test]
fn test_into_parts() {
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();

    let (row_indices, col_indices, values, shape) = matrix.into_parts();
    assert_eq!(row_indices, vec![0, 2, 3]);
    assert_eq!(col_indices, vec![0, 2, 1]);
    assert_eq!(values, vec![1.0, 2.0, 3.0]);
    assert_eq!(shape, (2, 3));
}

#[test]
fn test_map_values_transforms_values_and_preserves_structure() {
    let triplets = vec![(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)];
    let matrix = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();

    // Capture the original structure before consuming the matrix.
    let row_indices = matrix.row_indices().clone();
    let col_indices = matrix.col_indices().clone();
    let shape = matrix.shape();

    // Map f64 values to their string representation, changing the value type.
    let mapped: CsrMatrix<String> = matrix.map_values(|v| format!("v={}", v));

    // Structure is preserved exactly.
    assert_eq!(mapped.row_indices(), &row_indices);
    assert_eq!(mapped.col_indices(), &col_indices);
    assert_eq!(mapped.shape(), shape);

    // Only the stored values are transformed.
    assert_eq!(
        mapped.values(),
        &vec!["v=1".to_string(), "v=2".to_string(), "v=3".to_string()]
    );
}

#[test]
fn test_map_values_on_empty_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    let mapped: CsrMatrix<i64> = matrix.map_values(|v| v as i64);

    assert_eq!(mapped.shape(), (0, 0));
    assert!(mapped.row_indices().is_empty());
    assert!(mapped.col_indices().is_empty());
    assert!(mapped.values().is_empty());
}
