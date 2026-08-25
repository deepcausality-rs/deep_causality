/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cholesky and the least-squares solve built on it.
//!
//! The reference values are computed independently — the 3x3 factor by NumPy's `linalg.cholesky`,
//! the regression by `linalg.lstsq` — rather than by this crate.

use deep_causality_algebra::ConjugateScalar;
use deep_causality_linear::{
    DenseMatrix, DenseVector, LinearError, LinearErrorEnum, MatrixView, cholesky,
    solve_least_squares,
};
use deep_causality_num_complex::Complex;

fn m(d: Vec<f64>, n: usize) -> DenseMatrix<f64> {
    DenseMatrix::from_vec(d, n, n).unwrap()
}

#[test]
fn test_the_factor_of_a_known_matrix() {
    // [[4, 12, -16], [12, 37, -43], [-16, -43, 98]] factors as
    // L = [[2, 0, 0], [6, 1, 0], [-8, 5, 3]].
    let a = m(
        vec![4.0, 12.0, -16.0, 12.0, 37.0, -43.0, -16.0, -43.0, 98.0],
        3,
    );
    let l = cholesky(&a).expect("symmetric positive definite");
    let expected = [2.0, 0.0, 0.0, 6.0, 1.0, 0.0, -8.0, 5.0, 3.0];
    for i in 0..3 {
        for j in 0..3 {
            assert!(
                (l.get(i, j).unwrap() - expected[i * 3 + j]).abs() < 1e-12,
                "at ({i}, {j}): {} vs {}",
                l.get(i, j).unwrap(),
                expected[i * 3 + j]
            );
        }
    }
}

#[test]
fn test_the_factor_reconstructs_the_input() {
    let data = vec![4.0, 12.0, -16.0, 12.0, 37.0, -43.0, -16.0, -43.0, 98.0];
    let a = m(data.clone(), 3);
    let l = cholesky(&a).unwrap();
    // L L^T = A.
    for i in 0..3 {
        for j in 0..3 {
            let mut acc = 0.0;
            for k in 0..3 {
                acc += l.get(i, k).unwrap() * l.get(j, k).unwrap();
            }
            assert!((acc - data[i * 3 + j]).abs() < 1e-12, "at ({i}, {j})");
        }
    }
}

#[test]
fn test_the_strict_upper_triangle_is_exactly_zero() {
    // Lower-triangular is a claim about the factor, not an approximation of one.
    let a = m(vec![2.0, 1.0, 1.0, 2.0], 2);
    let l = cholesky(&a).unwrap();
    assert_eq!(l.get(0, 1).unwrap(), 0.0);
}

#[test]
fn test_the_diagonal_is_positive() {
    let a = m(vec![2.0, 1.0, 1.0, 2.0], 2);
    let l = cholesky(&a).unwrap();
    for i in 0..2 {
        assert!(l.get(i, i).unwrap() > 0.0, "diagonal entry {i}");
    }
}

#[test]
fn test_an_indefinite_matrix_is_rejected_at_the_index_that_proves_it() {
    // diag(1, -1) is invertible and indefinite: the failure is not singularity, and the second
    // diagonal entry is where it shows.
    let a = m(vec![1.0, 0.0, 0.0, -1.0], 2);
    assert!(matches!(
        cholesky(&a),
        Err(LinearError(LinearErrorEnum::NotPositiveDefinite {
            at_index: 1
        }))
    ));
}

#[test]
fn test_a_negative_leading_entry_is_rejected_at_index_zero() {
    let a = m(vec![-4.0, 0.0, 0.0, 1.0], 2);
    assert!(matches!(
        cholesky(&a),
        Err(LinearError(LinearErrorEnum::NotPositiveDefinite {
            at_index: 0
        }))
    ));
}

#[test]
fn test_a_singular_positive_semi_definite_matrix_is_rejected() {
    // [[1, 1], [1, 1]] is PSD but not positive *definite*: the second radicand is exactly zero.
    let a = m(vec![1.0, 1.0, 1.0, 1.0], 2);
    assert!(matches!(
        cholesky(&a),
        Err(LinearError(LinearErrorEnum::NotPositiveDefinite {
            at_index: 1
        }))
    ));
}

#[test]
fn test_a_rectangular_matrix_is_rejected() {
    let a = DenseMatrix::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2, 3).unwrap();
    assert!(matches!(
        cholesky(&a),
        Err(LinearError(LinearErrorEnum::NotSquare { shape: (2, 3) }))
    ));
}

#[test]
fn test_a_hermitian_complex_matrix_factors() {
    type C = Complex<f64>;
    let c = |re: f64, im: f64| Complex::new(re, im);
    // [[2, 1-i], [1+i, 3]] is Hermitian positive definite: its eigenvalues are 1 and 4.
    let a: DenseMatrix<C> = DenseMatrix::from_vec(
        vec![c(2.0, 0.0), c(1.0, -1.0), c(1.0, 1.0), c(3.0, 0.0)],
        2,
        2,
    )
    .unwrap();
    let l = cholesky(&a).expect("Hermitian positive definite");

    // The diagonal of a Cholesky factor is real and positive, complex input or not.
    for i in 0..2 {
        let d = l.get(i, i).unwrap();
        assert!(d.im.abs() < 1e-14, "diagonal {i} must be real, got {d:?}");
        assert!(d.re > 0.0);
    }
    // L L^H = A.
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = c(0.0, 0.0);
            for k in 0..2 {
                acc += l.get(i, k).unwrap() * l.get(j, k).unwrap().conjugate();
            }
            let want = a.get(i, j).unwrap();
            let d = acc - want;
            assert!(
                (d.re * d.re + d.im * d.im).sqrt() < 1e-12,
                "at ({i}, {j}): {acc:?} vs {want:?}"
            );
        }
    }
}

// ---- least squares -----------------------------------------------------------------------------

#[test]
fn test_the_least_squares_fit_of_a_known_regression() {
    // Fitting y = a + b x to (1, 6), (2, 5), (3, 7), (4, 10) gives a = 3.5, b = 1.4.
    let a = DenseMatrix::from_vec(vec![1.0, 1.0, 1.0, 2.0, 1.0, 3.0, 1.0, 4.0], 4, 2).unwrap();
    let b = DenseVector::from_vec(vec![6.0_f64, 5.0, 7.0, 10.0]);
    let x = solve_least_squares(&a, &b).expect("full column rank");
    assert_eq!(x.len(), 2);
    assert!((x.as_slice()[0] - 3.5).abs() < 1e-10, "{:?}", x.as_slice());
    assert!((x.as_slice()[1] - 1.4).abs() < 1e-10, "{:?}", x.as_slice());
}

#[test]
fn test_an_exactly_determined_system_is_solved_exactly() {
    // A square full-rank system has residual zero, so least squares returns the exact solution.
    // [[2, 1], [1, 3]] x = [5, 10] has x = [1, 3].
    let a = DenseMatrix::from_vec(vec![2.0, 1.0, 1.0, 3.0], 2, 2).unwrap();
    let b = DenseVector::from_vec(vec![5.0_f64, 10.0]);
    let x = solve_least_squares(&a, &b).unwrap();
    assert!((x.as_slice()[0] - 1.0).abs() < 1e-10, "{:?}", x.as_slice());
    assert!((x.as_slice()[1] - 3.0).abs() < 1e-10, "{:?}", x.as_slice());
}

#[test]
fn test_the_residual_is_orthogonal_to_the_column_space() {
    // The defining property: A^T (A x - b) = 0. Checking it proves the answer minimises the
    // residual rather than merely being close to one that does.
    let a = DenseMatrix::from_vec(vec![1.0, 1.0, 1.0, 2.0, 1.0, 3.0, 1.0, 4.0], 4, 2).unwrap();
    let b = DenseVector::from_vec(vec![6.0_f64, 5.0, 7.0, 10.0]);
    let x = solve_least_squares(&a, &b).unwrap();

    let mut resid = [0.0f64; 4];
    for (i, slot) in resid.iter_mut().enumerate() {
        let mut acc = 0.0;
        for j in 0..2 {
            acc += a.get(i, j).unwrap() * x.as_slice()[j];
        }
        *slot = acc - b.as_slice()[i];
    }
    for j in 0..2 {
        let mut acc = 0.0;
        for (i, r) in resid.iter().enumerate() {
            acc += a.get(i, j).unwrap() * r;
        }
        assert!(acc.abs() < 1e-10, "column {j} not orthogonal: {acc}");
    }
}

#[test]
fn test_a_right_hand_side_of_the_wrong_length_is_rejected() {
    let a = DenseMatrix::from_vec(vec![1.0, 1.0, 1.0, 2.0, 1.0, 3.0, 1.0, 4.0], 4, 2).unwrap();
    let b = DenseVector::from_vec(vec![6.0_f64, 5.0]);
    assert!(matches!(
        solve_least_squares(&a, &b),
        Err(LinearError(LinearErrorEnum::LengthMismatch {
            expected: 4,
            found: 2
        }))
    ));
}

#[test]
fn test_linearly_dependent_columns_are_rejected() {
    // The second column is twice the first, so A^T A is singular and the problem has no unique
    // solution. That surfaces as the Cholesky failing, which is where it should.
    let a = DenseMatrix::from_vec(vec![1.0, 2.0, 2.0, 4.0, 3.0, 6.0], 3, 2).unwrap();
    let b = DenseVector::from_vec(vec![1.0_f64, 2.0, 3.0]);
    assert!(matches!(
        solve_least_squares(&a, &b),
        Err(LinearError(LinearErrorEnum::NotPositiveDefinite { .. }))
    ));
}
