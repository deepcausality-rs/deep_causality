/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Written against the unimplemented API. Every test here must fail with the unimplemented panic
//! until phase 4, and must then pass without its assertion being weakened.

use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{DenseMatrix, LinearError, LinearErrorEnum, MatrixBuild, MatrixView};

#[test]
fn test_from_vec_carries_the_shape() {
    let (d, r, c) = unit_determinant_3x3();
    let m = DenseMatrix::from_vec(d, r, c).expect("shape agrees with the buffer");
    assert_eq!(m.rows(), 3);
    assert_eq!(m.cols(), 3);
    assert_eq!(m.shape(), (3, 3));
}

#[test]
fn test_from_vec_rejects_a_buffer_that_does_not_match_the_shape() {
    let e = DenseMatrix::from_vec(vec![1.0, 2.0, 3.0], 2, 2).unwrap_err();
    assert!(
        matches!(e, LinearError(LinearErrorEnum::ShapeMismatch { .. })),
        "got {e:?}"
    );
}

#[test]
fn test_get_reads_row_major() {
    let (d, r, c) = unit_determinant_3x3();
    let m = DenseMatrix::from_vec(d, r, c).unwrap();
    assert_eq!(m.get(0, 1).unwrap(), 2.0);
    assert_eq!(m.get(1, 2).unwrap(), 4.0);
    assert_eq!(m.get(2, 0).unwrap(), 0.0);
}

#[test]
fn test_get_rejects_an_index_outside_the_shape() {
    let (d, r, c) = unit_determinant_3x3();
    let m = DenseMatrix::from_vec(d, r, c).unwrap();
    let e = m.get(3, 0).unwrap_err();
    assert!(
        matches!(
            e,
            LinearError(LinearErrorEnum::IndexOutOfBounds {
                index: (3, 0),
                shape: (3, 3)
            })
        ),
        "got {e:?}"
    );
    assert!(m.get(0, 3).is_err());
}

#[test]
fn test_row_returns_a_contiguous_slice() {
    let (d, r, c) = unit_determinant_3x3();
    let m = DenseMatrix::from_vec(d, r, c).unwrap();
    assert_eq!(m.row(1).unwrap(), &[0.0, 1.0, 4.0]);
    assert!(m.row(3).is_err());
}

#[test]
fn test_zeros_has_the_shape_and_no_content() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    assert_eq!(m.shape(), (2, 3));
    for i in 0..2 {
        for j in 0..3 {
            assert_eq!(m.get(i, j).unwrap(), 0.0);
        }
    }
}

#[test]
fn test_identity_is_one_on_the_diagonal_and_zero_elsewhere() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(3);
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_eq!(m.get(i, j).unwrap(), expected, "at ({i}, {j})");
        }
    }
}

#[test]
fn test_set_writes_and_rejects_out_of_bounds() {
    let mut m: DenseMatrix<f64> = DenseMatrix::zeros(2, 2);
    m.set(1, 0, 7.0).unwrap();
    assert_eq!(m.get(1, 0).unwrap(), 7.0);
    assert!(m.set(2, 0, 1.0).is_err());
}

// ---- corner cases: the shapes that hold nothing -------------------------------------------------

#[test]
fn test_zero_by_zero_is_empty_and_square() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(0, 0);
    assert!(m.is_empty());
    assert!(m.is_square(), "0x0 is square; the empty product is one");
    assert_eq!(m.len(), 0);
}

#[test]
fn test_zero_by_n_and_n_by_zero_are_distinct_and_both_empty() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(0, 3);
    let b: DenseMatrix<f64> = DenseMatrix::zeros(3, 0);
    assert_eq!(a.shape(), (0, 3));
    assert_eq!(b.shape(), (3, 0));
    assert!(a.is_empty() && b.is_empty());
    assert!(!a.is_square() && !b.is_square());
}

#[test]
fn test_one_by_one() {
    let mut m: DenseMatrix<f64> = DenseMatrix::zeros(1, 1);
    m.set(0, 0, 5.0).unwrap();
    assert!(m.is_square());
    assert_eq!(m.get(0, 0).unwrap(), 5.0);
}

#[test]
fn test_non_square_in_both_orientations() {
    let wide: DenseMatrix<f64> = DenseMatrix::zeros(2, 5);
    let tall: DenseMatrix<f64> = DenseMatrix::zeros(5, 2);
    assert!(!wide.is_square() && !tall.is_square());
    assert_eq!(wide.len(), 10);
    assert_eq!(tall.len(), 10);
}

#[test]
fn test_a_degenerate_shape_never_panics_on_access() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(0, 0);
    // A typed error naming the shape it was checked against, not a panic and not a fabricated zero.
    assert!(matches!(
        m.get(0, 0),
        Err(LinearError(LinearErrorEnum::IndexOutOfBounds {
            index: (0, 0),
            shape: (0, 0)
        }))
    ));
}
