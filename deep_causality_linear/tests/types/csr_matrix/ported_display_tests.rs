/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `Display` rendering of `CsrMatrix`, ported from `deep_causality_sparse`.
//!
//! The rendering is dense: every position inside the shape is printed, so a structural zero and a
//! stored zero look alike, and the reader sees the matrix the CSR arrays stand for rather than the
//! arrays themselves. Covered here are the header line, the `[Empty]` short form for a degenerate
//! shape, the per-row bracketed layout, the eight-column right-aligned field, and the three-decimal
//! rounding.
//!
//! # These tests do not compile yet
//!
//! `deep_causality_linear::CsrMatrix` carries no `impl Display`, so every assertion below fails with
//! E0277. The values are the ones `deep_causality_sparse` produces, and a probe that drove the same
//! formatting algorithm through `shape` and `get_value_at` reproduced all seven strings from
//! linear's own data. The gap is the impl alone; porting the sparse body verbatim turns the file
//! green without touching a single expectation here.

use deep_causality_linear::CsrMatrix;

#[test]
fn test_display_marks_a_zero_dimension_matrix_as_empty() {
    let matrix: CsrMatrix<f64> = CsrMatrix::new();
    let expected = "CsrMatrix (0x0) [Empty]";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_renders_a_single_entry_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap();
    let expected = "CsrMatrix (1x1)\n[   1.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_prints_structural_zeros_alongside_stored_entries() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    // [[1.0, 0.0, 2.0],
    //  [0.0, 3.0, 0.0]]
    let expected =
        "CsrMatrix (2x3)\n[   1.000,    0.000,    2.000]\n[   0.000,    3.000,    0.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_prints_a_row_that_stores_nothing() {
    let matrix: CsrMatrix<f64> =
        CsrMatrix::from_triplets(3, 2, &[(0, 0, 1.0), (2, 1, 4.0)]).unwrap();
    // [[1.0, 0.0],
    //  [0.0, 0.0],
    //  [0.0, 4.0]]
    let expected =
        "CsrMatrix (3x2)\n[   1.000,    0.000]\n[   0.000,    0.000]\n[   0.000,    4.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_aligns_every_column_of_a_larger_matrix() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(
        4,
        4,
        &[
            (0, 0, 1.123),
            (0, 3, 2.0),
            (1, 1, 3.45),
            (2, 0, 5.0),
            (2, 2, 6.789),
            (3, 3, 7.0),
        ],
    )
    .unwrap();
    let expected = "CsrMatrix (4x4)\n[   1.123,    0.000,    0.000,    2.000]\n[   0.000,    3.450,    0.000,    0.000]\n[   5.000,    0.000,    6.789,    0.000]\n[   0.000,    0.000,    0.000,    7.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_keeps_the_shape_when_every_triplet_is_zero() {
    let triplets = vec![(0, 0, 0.0), (1, 1, 0.0)];
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(2, 2, &triplets).unwrap();
    // The zeros are dropped on construction; the declared 2x2 shape still governs the rendering.
    let expected = "CsrMatrix (2x2)\n[   0.000,    0.000]\n[   0.000,    0.000]\n";
    assert_eq!(format!("{}", matrix), expected);
}

#[test]
fn test_display_rounds_to_three_decimal_places() {
    let matrix: CsrMatrix<f64> = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.234567)]).unwrap();
    let expected = "CsrMatrix (1x1)\n[   1.235]\n";
    assert_eq!(format!("{}", matrix), expected);
}
