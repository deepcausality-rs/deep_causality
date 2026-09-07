/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CsrMatrix<i8>::vec_mult_real` — a sign matrix against a vector of reals.
//!
//! # The oracle
//!
//! Products written out by hand. The matrix used throughout is the `2×3` sign matrix
//!
//! ```text
//! [  1   0  -1 ]
//! [  0   1   1 ]
//! ```
//!
//! against `v = (2, 3, 5)`, giving `(2 − 5, 3 + 5) = (−3, 8)`. Nothing here calls the function to
//! obtain its own expectation, and the arithmetic is small enough to check by eye.
//!
//! # Corner cases, enumerated before the suite
//!
//! The hand-computed product; a wrong-length vector in both directions, which is the behaviour
//! change this replaces; an empty shape; a row storing nothing; a matrix of all zeros; a `0`
//! stored explicitly, which is a stored entry that contributes nothing; negative signs; and the
//! result at a non-`f64` scalar, since the sign is lifted into whatever `R` the caller works in.

use deep_causality_linear::{CsrMatrix, LinearErrorEnum};
use deep_causality_num::Float106;

/// `[[1, 0, -1], [0, 1, 1]]` as a sparse sign matrix.
fn signs() -> CsrMatrix<i8> {
    CsrMatrix::from_triplets(2, 3, &[(0, 0, 1), (0, 2, -1), (1, 1, 1), (1, 2, 1)]).unwrap()
}

#[test]
fn test_the_product_against_its_hand_computed_value() {
    // (1·2 + 0·3 − 1·5, 0·2 + 1·3 + 1·5) = (−3, 8).
    let out = signs().vec_mult_real(&[2.0f64, 3.0, 5.0]).unwrap();
    assert_eq!(out, vec![-3.0, 8.0]);
}

#[test]
fn test_a_short_vector_is_refused_rather_than_truncated() {
    // The behaviour this replaces: the callers guarded with `if col < vector.len()` and dropped the
    // term, returning (2, 3) here instead of refusing — a plausible answer to a malformed question.
    let err = signs().vec_mult_real(&[2.0f64, 3.0]).unwrap_err();
    assert!(
        matches!(
            err.kind(),
            LinearErrorEnum::LengthMismatch {
                expected: 3,
                found: 2
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn test_a_long_vector_is_refused_too() {
    let err = signs().vec_mult_real(&[2.0f64, 3.0, 5.0, 7.0]).unwrap_err();
    assert!(
        matches!(
            err.kind(),
            LinearErrorEnum::LengthMismatch {
                expected: 3,
                found: 4
            }
        ),
        "got {err:?}"
    );
}

#[test]
fn test_an_empty_vector_against_a_matrix_with_columns_is_refused() {
    let err = signs().vec_mult_real::<f64>(&[]).unwrap_err();
    assert!(
        matches!(err.kind(), LinearErrorEnum::LengthMismatch { .. }),
        "got {err:?}"
    );
}

#[test]
fn test_a_row_storing_nothing_contributes_zero() {
    // [[1], [], [1]] over one column: the middle row has no stored entry and must still appear.
    let m = CsrMatrix::from_triplets(3, 1, &[(0, 0, 1), (2, 0, 1)]).unwrap();
    assert_eq!(m.vec_mult_real(&[4.0f64]).unwrap(), vec![4.0, 0.0, 4.0]);
}

#[test]
fn test_a_matrix_of_all_zeros_gives_a_zero_vector() {
    let m: CsrMatrix<i8> = CsrMatrix::from_triplets(2, 2, &[]).unwrap();
    assert_eq!(m.vec_mult_real(&[3.0f64, 4.0]).unwrap(), vec![0.0, 0.0]);
}

#[test]
fn test_a_zero_triplet_contributes_nothing() {
    // `from_triplets` drops an explicit zero rather than storing it — a structural zero and a
    // stored zero are the same matrix — so this checks the product is right either way: the term
    // `0·9` must not appear in the sum whether or not the entry survived construction.
    let m = CsrMatrix::from_triplets(1, 2, &[(0, 0, 0), (0, 1, 1)]).unwrap();
    assert_eq!(m.values().len(), 1, "the zero triplet is not stored");
    assert_eq!(m.vec_mult_real(&[9.0f64, 4.0]).unwrap(), vec![4.0]);
}

#[test]
fn test_an_empty_shape_gives_an_empty_result() {
    let m: CsrMatrix<i8> = CsrMatrix::from_triplets(0, 0, &[]).unwrap();
    assert_eq!(m.vec_mult_real::<f64>(&[]).unwrap(), Vec::<f64>::new());
}

#[test]
fn test_the_sign_is_lifted_into_the_callers_scalar() {
    // The same hand-computed product at a scalar that is not `f64`: the sign carries across, and
    // the result is exact because every value involved is a small integer.
    let out = signs()
        .vec_mult_real(&[
            Float106::from(2.0f64),
            Float106::from(3.0),
            Float106::from(5.0),
        ])
        .unwrap();
    assert_eq!(out[0], Float106::from(-3.0f64));
    assert_eq!(out[1], Float106::from(8.0f64));
}

#[test]
fn test_signs_larger_than_one_are_carried_faithfully() {
    // The type is `i8`, and although discrete exterior calculus only produces ±1, nothing in the
    // signature promises that. 3·2 + (−4)·5 = −14.
    let m = CsrMatrix::from_triplets(1, 2, &[(0, 0, 3), (0, 1, -4)]).unwrap();
    assert_eq!(m.vec_mult_real(&[2.0f64, 5.0]).unwrap(), vec![-14.0]);
}
