/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The arithmetic surface of `CsrMatrix`, ported from `deep_causality_sparse`.
//!
//! Covers entrywise addition, scalar multiplication, the matrix–vector and matrix–matrix products,
//! and the transpose — including the array-level shape of the result, which is where a CSR
//! reimplementation is most likely to drift from the original.
//!
//! `sub_matrix` has no counterpart here: `deep_causality_linear` offers subtraction through the
//! `Sub` operator alone, so the three sparse tests for it are recorded as a divergence rather than
//! rewritten against the operator.

use deep_causality_linear::{CsrMatrix, LinearError};

#[test]
fn test_add_matrix_sums_two_disjoint_patterns() {
    // A = [[1.0, 0.0], [0.0, 2.0]]
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    // B = [[0.0, 3.0], [4.0, 0.0]]
    let b = CsrMatrix::from_triplets(2, 2, &[(0, 1, 3.0), (1, 0, 4.0)]).unwrap();

    // C = A + B = [[1.0, 3.0], [4.0, 2.0]]
    let c = a.add_matrix(&b).unwrap();

    assert_eq!(c.get_value_at(0, 0), 1.0);
    assert_eq!(c.get_value_at(0, 1), 3.0);
    assert_eq!(c.get_value_at(1, 0), 4.0);
    assert_eq!(c.get_value_at(1, 1), 2.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_add_matrix_sums_entries_that_share_a_position() {
    // A = [[1.0, 2.0], [0.0, 0.0]]
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (0, 1, 2.0)]).unwrap();
    // B = [[3.0, 0.0], [0.0, 4.0]]
    let b = CsrMatrix::from_triplets(2, 2, &[(0, 0, 3.0), (1, 1, 4.0)]).unwrap();

    // C = A + B = [[4.0, 2.0], [0.0, 4.0]]
    let c = a.add_matrix(&b).unwrap();

    assert_eq!(c.get_value_at(0, 0), 4.0);
    assert_eq!(c.get_value_at(0, 1), 2.0);
    assert_eq!(c.get_value_at(1, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 4.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_add_matrix_drops_an_entry_that_cancels_to_zero() {
    let a = CsrMatrix::from_triplets(1, 1, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(1, 1, &[(0, 0, -1.0)]).unwrap();

    let c = a.add_matrix(&b).unwrap();

    assert_eq!(c.get_value_at(0, 0), 0.0);
    assert!(
        c.values().is_empty(),
        "a cancelled entry leaves the stored pattern"
    );
}

#[test]
fn test_add_matrix_rejects_two_different_shapes() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();

    let err = a.add_matrix(&b).unwrap_err();

    assert_eq!(
        err,
        LinearError::ShapeMismatch {
            left: (2, 2),
            right: (2, 3)
        }
    );
}

#[test]
fn test_scalar_mult_scales_every_stored_entry() {
    // A = [[1.0, 0.0], [0.0, 2.0]]
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();

    // C = 3 * A = [[3.0, 0.0], [0.0, 6.0]]
    let c = a.scalar_mult(3.0);

    assert_eq!(c.get_value_at(0, 0), 3.0);
    assert_eq!(c.get_value_at(0, 1), 0.0);
    assert_eq!(c.get_value_at(1, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 6.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_scalar_mult_by_zero_keeps_the_sparsity_pattern() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();

    let c = a.scalar_mult(0.0);

    assert_eq!(c.get_value_at(0, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 0.0);
    // The entries are now zero and the three arrays still describe two stored positions. Scaling
    // maps values and runs no pruning pass; a caller who wants the pattern tightened rebuilds.
    assert_eq!(c.values(), &[0.0, 0.0]);
    assert_eq!(c.col_indices(), &[0, 1]);
    assert_eq!(c.row_indices(), &[0, 1, 2]);
}

#[test]
fn test_vec_mult_computes_the_matrix_vector_product() {
    // A = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();

    let x = vec![1.0, 2.0, 3.0];
    // y = Ax = [1*1 + 0*2 + 2*3, 0*1 + 3*2 + 0*3] = [7.0, 6.0]
    let y = a.vec_mult(&x).unwrap();

    assert_eq!(y, vec![7.0, 6.0]);
}

#[test]
fn test_vec_mult_rejects_a_vector_of_the_wrong_length() {
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0)]).unwrap();
    let x_invalid = vec![1.0, 2.0];

    let err = a.vec_mult(&x_invalid).unwrap_err();

    assert_eq!(
        err,
        LinearError::LengthMismatch {
            expected: 3,
            found: 2
        }
    );
}

#[test]
fn test_mat_mult_computes_the_matrix_product() {
    // A (2x3) = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();
    // B (3x2) = [[4.0, 0.0], [0.0, 5.0], [6.0, 0.0]]
    let b = CsrMatrix::from_triplets(3, 2, &[(0, 0, 4.0), (1, 1, 5.0), (2, 0, 6.0)]).unwrap();

    // C = A * B (2x2) = [[1*4 + 2*6, 0.0], [0.0, 3*5]] = [[16.0, 0.0], [0.0, 15.0]]
    let c = a.mat_mult(&b).unwrap();

    assert_eq!(c.get_value_at(0, 0), 16.0);
    assert_eq!(c.get_value_at(0, 1), 0.0);
    assert_eq!(c.get_value_at(1, 0), 0.0);
    assert_eq!(c.get_value_at(1, 1), 15.0);
    assert_eq!(c.shape(), (2, 2));
}

#[test]
fn test_mat_mult_rejects_inner_dimensions_that_do_not_meet() {
    let a = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    let b = CsrMatrix::from_triplets(3, 2, &[(0, 0, 1.0)]).unwrap();

    let err = a.mat_mult(&b).unwrap_err();

    assert_eq!(
        err,
        LinearError::InnerDimensionMismatch {
            left_cols: 2,
            right_rows: 3
        }
    );
}

#[test]
fn test_transpose_swaps_the_shape_and_rebuilds_the_three_arrays() {
    // A (2x3) = [[1.0, 0.0, 2.0], [0.0, 3.0, 0.0]]
    let a = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (0, 2, 2.0), (1, 1, 3.0)]).unwrap();

    // A^T (3x2) = [[1.0, 0.0], [0.0, 3.0], [2.0, 0.0]]
    let a_t = a.transpose();

    assert_eq!(a_t.shape(), (3, 2));
    assert_eq!(a_t.get_value_at(0, 0), 1.0);
    assert_eq!(a_t.get_value_at(0, 1), 0.0);
    assert_eq!(a_t.get_value_at(1, 0), 0.0);
    assert_eq!(a_t.get_value_at(1, 1), 3.0);
    assert_eq!(a_t.get_value_at(2, 0), 2.0);
    assert_eq!(a_t.get_value_at(2, 1), 0.0);

    // The original column indices become the new row boundaries.
    assert_eq!(a_t.row_indices(), &[0, 1, 2, 3]);
    assert_eq!(a_t.col_indices(), &[0, 1, 0]);
    assert_eq!(a_t.values(), &[1.0, 3.0, 2.0]);
}

#[test]
fn test_transpose_of_an_empty_matrix_is_empty() {
    let a: CsrMatrix<f64> = CsrMatrix::new();

    let a_t = a.transpose();

    assert_eq!(a_t.shape(), (0, 0));
    assert!(a_t.values().is_empty());
}
