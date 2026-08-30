/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Construction and the structural accessors on `CsrMatrix`.
//!
//! Ported from `deep_causality_sparse::tests::types::sparse_matrix::mod_tests`. Covers `new`,
//! `with_capacity`, `into_parts` and `map_values` — the part of the surface that carries the three
//! CSR arrays around without doing arithmetic on them.

use deep_causality_linear::CsrMatrix;

#[test]
fn test_new_carries_no_row_pointers_and_stores_nothing() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    assert_eq!(matrix.shape(), (0, 0));

    // Empty, matching the crate this moves from. A zero-row matrix has no row whose pointer could
    // be read, so `rows + 1 == 1` is an invariant with nothing to protect here, and `row_indices()`
    // is public — a caller reading it must keep seeing what it saw before.
    assert!(matrix.row_indices().is_empty());

    assert!(matrix.col_indices().is_empty());
    assert!(matrix.values().is_empty());
}

#[test]
fn test_with_capacity_sets_the_shape_and_reserves_without_storing() {
    let rows = 5;
    let cols = 10;
    let capacity = 20;
    let matrix: CsrMatrix<f64> = CsrMatrix::with_capacity(rows, cols, capacity);

    assert_eq!(matrix.shape(), (rows, cols));
    assert_eq!(matrix.row_indices().len(), rows + 1);
    assert_eq!(
        matrix.row_indices(),
        &vec![0; rows + 1],
        "all zeros initially"
    );

    assert!(matrix.col_indices().capacity() >= capacity);
    assert!(matrix.values().capacity() >= capacity);

    // Reserved room is not content.
    assert!(matrix.col_indices().is_empty());
    assert!(matrix.values().is_empty());
}

#[test]
fn test_into_parts_returns_the_three_arrays_and_the_shape() {
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
    let mapped: CsrMatrix<String> = matrix.map_values(|v| format!("v={v}"));

    // Structure is preserved exactly.
    assert_eq!(mapped.row_indices(), &row_indices);
    assert_eq!(mapped.col_indices(), &col_indices);
    assert_eq!(mapped.shape(), shape);

    // Only the stored values are transformed.
    assert_eq!(
        mapped.values(),
        &["v=1".to_string(), "v=2".to_string(), "v=3".to_string()]
    );
}

#[test]
fn test_map_values_on_an_empty_matrix_keeps_it_empty() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    let mapped: CsrMatrix<i64> = matrix.map_values(|v| v as i64);

    assert_eq!(mapped.shape(), (0, 0));

    // `map_values` moves the row pointers across untouched, so whatever `new` produced survives —
    // and `new` produces an empty vector.
    assert!(mapped.row_indices().is_empty());

    assert!(mapped.col_indices().is_empty());
    assert!(mapped.values().is_empty());
}
