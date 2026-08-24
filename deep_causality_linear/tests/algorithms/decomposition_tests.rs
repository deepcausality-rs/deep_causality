/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The delegation tests.
//!
//! Each records what `CausalTensor`'s method must still return after phase 5 reduces it to a call
//! into here. The baseline these are written against was captured from the tensor crate before any
//! of its bodies moved, and is recorded in `openspec/notes/linear/DELEGATION-BASELINE.md`.

use deep_causality_linear::{
    DenseMatrix, LinearError, MatrixBuild, MatrixView, Truncation, eigen_hermitian, qr,
    singular_values, svd, svd_truncated,
};

#[test]
fn test_svd_returns_the_shapes_the_tensor_surface_returns() {
    // Baseline: U[3,3] S[3] Vt[3,3]. S is rank-1.
    let m: DenseMatrix<f64> = DenseMatrix::identity(3);
    let (u, s, vt) = svd(&m).unwrap();
    assert_eq!(u.shape(), (3, 3));
    assert_eq!(s.len(), 3);
    assert_eq!(vt.shape(), (3, 3));
}

#[test]
fn test_the_singular_values_of_the_identity_are_all_one() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(3);
    let s = singular_values(&m).unwrap();
    assert_eq!(s.len(), 3);
    for i in 0..3 {
        assert!((s.get(i).unwrap() - 1.0).abs() < 1e-6);
    }
}

#[test]
fn test_singular_values_come_back_descending() {
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 0.0, 3.0], 2, 2).unwrap();
    let s = singular_values(&m).unwrap();
    assert!(s.get(0).unwrap() >= s.get(1).unwrap(), "must be descending");
    assert!((s.get(0).unwrap() - 3.0).abs() < 1e-6);
}

#[test]
fn test_a_rank_deficient_matrix_has_a_vanishing_singular_value() {
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 2.0, 2.0, 4.0], 2, 2).unwrap();
    let s = singular_values(&m).unwrap();
    assert!(
        s.get(1).unwrap().abs() < 1e-6,
        "the second singular value must vanish"
    );
}

#[test]
fn test_the_singular_values_agree_with_the_diagonal_of_s() {
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 0.0, 3.0], 2, 2).unwrap();
    let (_, s_factor, _) = svd(&m).unwrap();
    let s_vector = singular_values(&m).unwrap();
    for i in 0..2 {
        assert!(
            (s_factor.get(i).unwrap() - s_vector.get(i).unwrap()).abs() < 1e-12,
            "the convenience must agree with the S factor"
        );
    }
}

#[test]
fn test_truncating_by_rank_keeps_that_many_components() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(4);
    let (_, s, _) = svd_truncated(&m, &Truncation::Rank(2)).unwrap();
    assert_eq!(s.len(), 2);
}

#[test]
fn test_truncating_by_tolerance_drops_what_falls_below_it() {
    // Singular values 3 and 1; a tolerance of 2 keeps one.
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![3.0, 0.0, 0.0, 1.0], 2, 2).unwrap();
    let (_, s, _) = svd_truncated(&m, &Truncation::Tolerance(2.0)).unwrap();
    assert_eq!(s.len(), 1);
}

#[test]
fn test_rank_and_tolerance_apply_both() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(4);
    let (_, s, _) = svd_truncated(
        &m,
        &Truncation::RankAndTolerance {
            rank: 3,
            tolerance: 0.5,
        },
    )
    .unwrap();
    assert!(s.len() <= 3, "the rank cap must bind");
}

#[test]
fn test_qr_factors_multiply_back_to_the_original() {
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2).unwrap();
    let (q, r) = qr(&m).unwrap();
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0;
            for k in 0..2 {
                acc += q.get(i, k).unwrap() * r.get(k, j).unwrap();
            }
            assert!(
                (acc - m.get(i, j).unwrap()).abs() < 1e-6,
                "QR != A at ({i}, {j})"
            );
        }
    }
}

#[test]
fn test_qr_q_has_orthonormal_columns() {
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 2.0, 3.0, 4.0], 2, 2).unwrap();
    let (q, _) = qr(&m).unwrap();
    for a in 0..2 {
        for b in 0..2 {
            let mut dot = 0.0;
            for k in 0..2 {
                dot += q.get(k, a).unwrap() * q.get(k, b).unwrap();
            }
            let expected = if a == b { 1.0 } else { 0.0 };
            assert!(
                (dot - expected).abs() < 1e-6,
                "columns {a},{b} dot to {dot}"
            );
        }
    }
}

#[test]
fn test_eigen_of_a_diagonal_matrix_returns_its_diagonal() {
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![2.0, 0.0, 0.0, 5.0], 2, 2).unwrap();
    let (vals, _) = eigen_hermitian(&m).unwrap();
    let mut got = [vals.get(0).unwrap(), vals.get(1).unwrap()];
    got.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!((got[0] - 2.0).abs() < 1e-6);
    assert!((got[1] - 5.0).abs() < 1e-6);
}

#[test]
fn test_eigen_returns_a_vector_where_the_tensor_surface_returns_a_bare_vec() {
    // The one place the return shape differs from the tensor method. The delegating method
    // converts; recorded so phase 5 does not treat the difference as a regression.
    let m: DenseMatrix<f64> = DenseMatrix::identity(2);
    let (vals, vecs) = eigen_hermitian(&m).unwrap();
    assert_eq!(vals.len(), 2);
    assert_eq!(vecs.shape(), (2, 2));
}

#[test]
fn test_the_decompositions_reject_a_non_square_input_where_the_tensor_surface_does() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    assert!(matches!(
        eigen_hermitian(&m),
        Err(LinearError::NotSquare { .. })
    ));
}

#[test]
fn test_an_empty_matrix_is_decomposed_into_empty_factors_rather_than_rejected() {
    // Baseline: svd(0x0) = Ok(U[0,0] S[0] Vt[0,0]). Rejecting it would be a delegation regression.
    let m: DenseMatrix<f64> = DenseMatrix::zeros(0, 0);
    let (u, s, vt) = svd(&m).unwrap();
    assert_eq!(u.shape(), (0, 0));
    assert_eq!(s.len(), 0);
    assert_eq!(vt.shape(), (0, 0));
}
