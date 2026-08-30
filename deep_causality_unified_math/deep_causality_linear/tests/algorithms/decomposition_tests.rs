/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The delegation tests.
//!
//! Each records what `CausalTensor`'s method must still return after phase 5 reduces it to a call
//! into here. The baseline these are written against was captured from the tensor crate before any
//! of its bodies moved, and is recorded in `openspec/notes/archive/linear/DELEGATION-BASELINE.md`.

use deep_causality_linear::{
    DenseMatrix, LinearError, LinearErrorEnum, MatrixBuild, MatrixView, Truncation,
    eigen_hermitian, qr, singular_values, svd, svd_truncated,
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
        Err(LinearError(LinearErrorEnum::NotSquare { .. }))
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

// =============================================================================
// The ConjugateScalar band: complex matrices decompose here too.
// =============================================================================

mod conjugate_scalar {
    use deep_causality_algebra::ConjugateScalar;
    use deep_causality_linear::{DenseMatrix, MatrixView, eigen_hermitian, qr};
    use deep_causality_num_complex::Complex;

    type C = Complex<f64>;

    fn c(re: f64, im: f64) -> C {
        Complex::new(re, im)
    }

    fn cm(entries: Vec<C>, r: usize, cols: usize) -> DenseMatrix<C> {
        DenseMatrix::from_vec(entries, r, cols).unwrap()
    }

    /// Frobenius distance between two same-shaped matrices.
    fn dist(a: &DenseMatrix<C>, b: &DenseMatrix<C>) -> f64 {
        assert_eq!(a.shape(), b.shape());
        let (r, cl) = a.shape();
        let mut acc = 0.0;
        for i in 0..r {
            for j in 0..cl {
                let d = a.get(i, j).unwrap() - b.get(i, j).unwrap();
                acc += d.re * d.re + d.im * d.im;
            }
        }
        acc.sqrt()
    }

    #[test]
    fn test_a_hermitian_complex_matrix_has_real_eigenvalues() {
        // [[2, 1-i], [1+i, 3]] is Hermitian. Its characteristic polynomial is
        // λ² - 5λ + (6 - |1-i|²) = λ² - 5λ + 4, so the eigenvalues are 1 and 4 -- both real, which
        // is the property that makes a density matrix's spectrum a probability distribution.
        let m = cm(
            vec![c(2.0, 0.0), c(1.0, -1.0), c(1.0, 1.0), c(3.0, 0.0)],
            2,
            2,
        );
        let (vals, _) = eigen_hermitian(&m).expect("a Hermitian complex matrix decomposes");

        let mut got: Vec<f64> = vals.as_slice().iter().map(|z| z.re).collect();
        got.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((got[0] - 1.0).abs() < 1e-12, "got {got:?}");
        assert!((got[1] - 4.0).abs() < 1e-12, "got {got:?}");

        for z in vals.as_slice() {
            assert!(
                z.im.abs() < 1e-12,
                "a Hermitian eigenvalue must be real, got {z:?}"
            );
        }
    }

    #[test]
    fn test_the_complex_eigendecomposition_reconstructs_the_matrix() {
        // A = V diag(l) V^H is the claim; checking it is what proves the eigenvectors are the
        // eigenvectors and not merely that the eigenvalues are right.
        let m = cm(
            vec![c(2.0, 0.0), c(1.0, -1.0), c(1.0, 1.0), c(3.0, 0.0)],
            2,
            2,
        );
        let (vals, v) = eigen_hermitian(&m).unwrap();

        let n = 2;
        let mut back = vec![c(0.0, 0.0); n * n];
        for i in 0..n {
            for j in 0..n {
                let mut acc = c(0.0, 0.0);
                for k in 0..n {
                    // V[i,k] * l[k] * conj(V[j,k])
                    let vjk = v.get(j, k).unwrap();
                    acc += v.get(i, k).unwrap() * vals.as_slice()[k] * vjk.conjugate();
                }
                back[i * n + j] = acc;
            }
        }
        assert!(
            dist(&cm(back, 2, 2), &m) < 1e-12,
            "V diag(l) V^H must return the input"
        );
    }

    #[test]
    fn test_the_complex_qr_factors_reconstruct_and_q_is_unitary() {
        let m = cm(
            vec![
                c(1.0, 1.0),
                c(2.0, 0.0),
                c(0.0, -1.0),
                c(3.0, 2.0),
                c(1.0, 0.0),
                c(0.0, 1.0),
            ],
            3,
            2,
        );
        let (q, r) = qr(&m).expect("a complex matrix factorises");
        assert_eq!(q.shape(), (3, 2), "thin Q is m x min(m, n)");
        assert_eq!(r.shape(), (2, 2), "thin R is min(m, n) x n");

        // Q R = A.
        let mut prod = vec![c(0.0, 0.0); 3 * 2];
        for i in 0..3 {
            for j in 0..2 {
                let mut acc = c(0.0, 0.0);
                for k in 0..2 {
                    acc += q.get(i, k).unwrap() * r.get(k, j).unwrap();
                }
                prod[i * 2 + j] = acc;
            }
        }
        assert!(dist(&cm(prod, 3, 2), &m) < 1e-12, "Q R must return A");

        // Q^H Q = I: the columns are orthonormal under the conjugated inner product, which is what
        // makes the factor unitary rather than merely orthogonal.
        for a in 0..2 {
            for b in 0..2 {
                let mut acc = c(0.0, 0.0);
                for i in 0..3 {
                    acc += q.get(i, a).unwrap().conjugate() * q.get(i, b).unwrap();
                }
                let expected = if a == b { 1.0 } else { 0.0 };
                assert!(
                    (acc.re - expected).abs() < 1e-12 && acc.im.abs() < 1e-12,
                    "Q^H Q at ({a}, {b}) was {acc:?}"
                );
            }
        }

        // R is upper-triangular, exactly.
        assert_eq!(r.get(1, 0).unwrap(), c(0.0, 0.0));
    }

    #[test]
    fn test_a_real_matrix_gets_the_same_answer_through_the_widened_bound() {
        // The widening must not move the real path. A real symmetric matrix embedded in the
        // complex plane has to give what the real one gives.
        let real: DenseMatrix<f64> = DenseMatrix::from_vec(vec![2.0, 1.0, 1.0, 3.0], 2, 2).unwrap();
        let (rv, _) = eigen_hermitian(&real).unwrap();
        let mut real_vals: Vec<f64> = rv.as_slice().to_vec();
        real_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let embedded = cm(
            vec![c(2.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(3.0, 0.0)],
            2,
            2,
        );
        let (cv, _) = eigen_hermitian(&embedded).unwrap();
        let mut cplx_vals: Vec<f64> = cv.as_slice().iter().map(|z| z.re).collect();
        cplx_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for (a, b) in real_vals.iter().zip(&cplx_vals) {
            assert!((a - b).abs() < 1e-13, "{real_vals:?} vs {cplx_vals:?}");
        }
    }

    #[test]
    fn test_the_eigendecomposition_still_refuses_a_rectangular_matrix() {
        let m = cm(
            vec![
                c(1.0, 0.0),
                c(2.0, 0.0),
                c(3.0, 0.0),
                c(4.0, 0.0),
                c(5.0, 0.0),
                c(6.0, 0.0),
            ],
            2,
            3,
        );
        assert!(matches!(
            eigen_hermitian(&m),
            Err(deep_causality_linear::LinearError(
                deep_causality_linear::LinearErrorEnum::NotSquare { shape: (2, 3) }
            ))
        ));
    }
}

// =============================================================================
// The SVD is thin, and it admits complex.
// =============================================================================

mod svd_shape_and_complex {
    use deep_causality_algebra::ConjugateScalar;
    use deep_causality_linear::{
        DenseMatrix, MatrixView, Truncation, svd, svd_sorted, svd_truncated,
    };
    use deep_causality_num_complex::Complex;

    type C = Complex<f64>;
    fn c(re: f64, im: f64) -> C {
        Complex::new(re, im)
    }

    #[test]
    fn test_a_wide_matrix_gets_min_dimension_many_singular_values() {
        // A 2x3 matrix has 2 singular values, not 3. Returning three means returning a zero
        // dressed as a spectrum, and a U of 2x3 whose columns cannot be orthonormal.
        let m: DenseMatrix<f64> =
            DenseMatrix::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 2, 3).unwrap();
        let (u, s, vt) = svd(&m).unwrap();
        assert_eq!(u.shape(), (2, 2), "U is m x min(m, n)");
        assert_eq!(s.len(), 2, "min(m, n) singular values");
        assert_eq!(vt.shape(), (2, 3), "Vt is min(m, n) x n");
    }

    #[test]
    fn test_a_tall_matrix_keeps_the_same_shape_it_always_had() {
        let m: DenseMatrix<f64> =
            DenseMatrix::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0], 3, 2).unwrap();
        let (u, s, vt) = svd(&m).unwrap();
        assert_eq!(u.shape(), (3, 2));
        assert_eq!(s.len(), 2);
        assert_eq!(vt.shape(), (2, 2));
    }

    #[test]
    fn test_a_wide_matrix_reconstructs_from_its_thin_factors() {
        // The shape is only right if U S Vt still returns A.
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let m: DenseMatrix<f64> = DenseMatrix::from_vec(data.clone(), 2, 3).unwrap();
        let (u, s, vt) = svd(&m).unwrap();
        for i in 0..2 {
            for j in 0..3 {
                let mut acc = 0.0;
                for k in 0..s.len() {
                    acc += u.get(i, k).unwrap() * s.as_slice()[k] * vt.get(k, j).unwrap();
                }
                assert!(
                    (acc - data[i * 3 + j]).abs() < 1e-12,
                    "at ({i}, {j}): {acc} vs {}",
                    data[i * 3 + j]
                );
            }
        }
    }

    #[test]
    fn test_the_singular_values_are_the_known_ones() {
        // [[3, 0], [0, -2]] has singular values 3 and 2, descending.
        let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![3.0, 0.0, 0.0, -2.0], 2, 2).unwrap();
        let (_, s, _) = svd_sorted(&m).unwrap();
        assert!((s[0] - 3.0).abs() < 1e-12, "{s:?}");
        assert!((s[1] - 2.0).abs() < 1e-12, "{s:?}");
    }

    #[test]
    fn test_svd_sorted_returns_the_values_in_descending_order() {
        let m: DenseMatrix<f64> =
            DenseMatrix::from_vec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 10.0], 3, 3)
                .unwrap();
        let (_, s, _) = svd_sorted(&m).unwrap();
        for w in s.windows(2) {
            assert!(w[0] >= w[1], "not descending: {s:?}");
        }
    }

    #[test]
    fn test_a_complex_matrix_decomposes_and_reconstructs() {
        let data = vec![c(1.0, 1.0), c(2.0, 0.0), c(0.0, -1.0), c(3.0, 2.0)];
        let m: DenseMatrix<C> = DenseMatrix::from_vec(data.clone(), 2, 2).unwrap();
        let (u, s, vt) = svd_sorted(&m).expect("a complex matrix decomposes");

        assert_eq!(s.len(), 2);
        for v in &s {
            assert!(*v >= 0.0, "a singular value is non-negative, got {v}");
        }
        for w in s.windows(2) {
            assert!(w[0] >= w[1]);
        }

        // U diag(s) Vt = A, with the real singular values injected back.
        for i in 0..2 {
            for j in 0..2 {
                let mut acc = c(0.0, 0.0);
                for (k, &sk) in s.iter().enumerate() {
                    acc += u.get(i, k).unwrap() * C::from_real(sk) * vt.get(k, j).unwrap();
                }
                let d = acc - data[i * 2 + j];
                assert!(
                    (d.re * d.re + d.im * d.im).sqrt() < 1e-12,
                    "at ({i}, {j}): {acc:?}"
                );
            }
        }
    }

    #[test]
    fn test_the_complex_left_factor_has_orthonormal_columns() {
        let m: DenseMatrix<C> = DenseMatrix::from_vec(
            vec![
                c(1.0, 1.0),
                c(2.0, 0.0),
                c(0.0, -1.0),
                c(3.0, 2.0),
                c(1.0, 0.0),
                c(0.0, 1.0),
            ],
            3,
            2,
        )
        .unwrap();
        let (u, _, _) = svd_sorted(&m).unwrap();
        for a in 0..2 {
            for b in 0..2 {
                let mut acc = c(0.0, 0.0);
                for i in 0..3 {
                    acc += u.get(i, a).unwrap().conjugate() * u.get(i, b).unwrap();
                }
                let expected = if a == b { 1.0 } else { 0.0 };
                assert!(
                    (acc.re - expected).abs() < 1e-10 && acc.im.abs() < 1e-10,
                    "U^H U at ({a}, {b}) was {acc:?}"
                );
            }
        }
    }

    #[test]
    fn test_truncation_takes_a_real_tolerance_and_keeps_a_prefix() {
        // diag(5, 3, 0.5): a tolerance of 1 keeps the first two.
        let m: DenseMatrix<f64> =
            DenseMatrix::from_vec(vec![5.0, 0.0, 0.0, 0.0, 3.0, 0.0, 0.0, 0.0, 0.5], 3, 3).unwrap();
        let (u, s, vt) = svd_truncated(&m, &Truncation::Tolerance(1.0)).unwrap();
        assert_eq!(s.len(), 2, "0.5 is below the tolerance");
        assert_eq!(u.shape(), (3, 2));
        assert_eq!(vt.shape(), (2, 3));
        assert!((s.as_slice()[0] - 5.0).abs() < 1e-12);
        assert!((s.as_slice()[1] - 3.0).abs() < 1e-12);

        // And by rank.
        let (_, s2, _) = svd_truncated(&m, &Truncation::Rank(1)).unwrap();
        assert_eq!(s2.len(), 1);
        assert!((s2.as_slice()[0] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn test_a_complex_truncation_compares_against_a_real_tolerance() {
        let m: DenseMatrix<C> = DenseMatrix::from_vec(
            vec![c(4.0, 0.0), c(0.0, 0.0), c(0.0, 0.0), c(0.0, 0.25)],
            2,
            2,
        )
        .unwrap();
        // Singular values 4 and 0.25; a real tolerance of 1 keeps one.
        let (u, s, vt) = svd_truncated(&m, &Truncation::Tolerance(1.0)).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(u.shape(), (2, 1));
        assert_eq!(vt.shape(), (1, 2));
    }
}

// ---- mutation-driven: the tests above use diagonal matrices ------------------------------------
//
// Every SVD case in this file was a diagonal matrix. A diagonal matrix is symmetric, so U and V
// coincide and several decisions in `svd_sorted` and `svd_truncated` become invisible. Mutation
// testing surfaced ten surviving mutants across those two functions. The tests below are written
// against the specific decisions the mutants flipped.

/// `svd_sorted` returns empty factors when EITHER dimension is zero, not only when both are.
///
/// The guard is `rows == 0 || cols == 0`. Replacing the `||` with `&&` survived every test,
/// because the only empty case under test was `0 x 0`, where both readings agree.
#[test]
fn test_the_svd_of_a_matrix_with_one_zero_dimension_is_empty() {
    let tall: DenseMatrix<f64> = DenseMatrix::zeros(3, 0);
    let (u, s, vt) = svd(&tall).unwrap();
    assert_eq!(s.len(), 0, "a 3x0 matrix has no singular values");
    assert_eq!(u.rows(), 3);
    assert_eq!(u.cols(), 0);
    assert_eq!(vt.rows(), 0);

    let wide: DenseMatrix<f64> = DenseMatrix::zeros(0, 4);
    let (u, s, vt) = svd(&wide).unwrap();
    assert_eq!(s.len(), 0, "a 0x4 matrix has no singular values");
    assert_eq!(u.rows(), 0);
    assert_eq!(vt.cols(), 4);
}

/// A square matrix is decomposed directly, and its `U` and `Vᴴ` are not interchangeable.
///
/// `svd_sorted` transposes when `rows < cols`. Widening that to `rows <= cols` sends a square
/// matrix down the transposed path, which returns `U` and `V` swapped. The swap still reconstructs
/// the input — `V Σ Uᴴ = (U Σ Vᴴ)ᴴ` and the input is its own double adjoint — so a reconstruction
/// check cannot see it. Only an assertion about `U` itself can.
///
/// The matrix has to be genuinely asymmetric under the swap. `[[0, 2], [1, 0]]` is not: its left
/// and right singular vectors are both signed unit vectors, and exchanging them leaves every
/// magnitude in place. `A = [[4, 1], [2, 3]]` has `A Aᴴ = [[17, 11], [11, 13]]` against
/// `Aᴴ A = [[20, 10], [10, 10]]`, so the two factors are different matrices and the swap shows.
#[test]
fn test_a_square_matrix_keeps_its_left_and_right_factors_distinct() {
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![4.0, 1.0, 2.0, 3.0], 2, 2).unwrap();
    let (u, s, vt) = svd(&m).unwrap();

    assert!((s.as_slice()[0] - 5.116_672_736_016_927).abs() < 1e-12);
    assert!((s.as_slice()[1] - 1.954_395_075_848_547_8).abs() < 1e-12);

    // The leading left and right singular vectors have different leading components, so reading
    // one where the other is meant is visible here and nowhere in a reconstruction check.
    assert!(
        (u.get(0, 0).unwrap().abs() - 0.767_751_730_118_527).abs() < 1e-9,
        "|U[0][0]| must be 0.7678 (the left factor), got {}",
        u.get(0, 0).unwrap().abs()
    );
    assert!(
        (vt.get(0, 0).unwrap().abs() - 0.850_650_808_352_039_9).abs() < 1e-9,
        "|Vᴴ[0][0]| must be 0.8507 (the right factor), got {}",
        vt.get(0, 0).unwrap().abs()
    );
}

/// The tolerance gate is strict: a singular value exactly equal to the tolerance is dropped.
///
/// `filter(|s| *s > t)` versus `>= t` differ only at equality, and no test placed a singular value
/// on the boundary. `diag(3, 2, 1)` at tolerance `2.0` keeps one value under the documented
/// convention and two under the other.
#[test]
fn test_a_singular_value_equal_to_the_tolerance_is_truncated_away() {
    let m: DenseMatrix<f64> =
        DenseMatrix::from_vec(vec![3.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0], 3, 3).unwrap();

    let (_, s, _) = svd_truncated(&m, &Truncation::Tolerance(2.0)).unwrap();
    assert_eq!(s.len(), 1, "only the value strictly above 2.0 survives");
    assert!((s.as_slice()[0] - 3.0).abs() < 1e-12);

    // Just below the boundary keeps both.
    let (_, s, _) = svd_truncated(&m, &Truncation::Tolerance(1.999_999)).unwrap();
    assert_eq!(s.len(), 2);
}

/// `RankAndTolerance` applies the tolerance as well as the rank.
///
/// The rank was always the binding half in the existing cases, so the comparison in the tolerance
/// filter was never observed. Three separate mutants of it survived. Here the rank is generous and
/// the tolerance does the work.
#[test]
fn test_rank_and_tolerance_lets_the_tolerance_bind_when_the_rank_is_generous() {
    let m: DenseMatrix<f64> =
        DenseMatrix::from_vec(vec![4.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.5], 3, 3).unwrap();

    let (_, s, _) = svd_truncated(
        &m,
        &Truncation::RankAndTolerance {
            rank: 3,
            tolerance: 1.0,
        },
    )
    .unwrap();
    assert_eq!(
        s.len(),
        2,
        "the tolerance drops 0.5 while the rank allows 3"
    );
    assert!((s.as_slice()[0] - 4.0).abs() < 1e-12);
    assert!((s.as_slice()[1] - 2.0).abs() < 1e-12);

    // And the rank still binds when it is the smaller of the two.
    let (_, s, _) = svd_truncated(
        &m,
        &Truncation::RankAndTolerance {
            rank: 1,
            tolerance: 1.0,
        },
    )
    .unwrap();
    assert_eq!(s.len(), 1);

    // The gate is strict here too: a singular value exactly equal to the tolerance is dropped.
    // Without a value on the boundary the comparison is unobserved, and three separate mutants of
    // it survived.
    let boundary: DenseMatrix<f64> =
        DenseMatrix::from_vec(vec![4.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 1.0], 3, 3).unwrap();
    let (_, s, _) = svd_truncated(
        &boundary,
        &Truncation::RankAndTolerance {
            rank: 3,
            tolerance: 2.0,
        },
    )
    .unwrap();
    assert_eq!(
        s.len(),
        1,
        "2.0 sits on the tolerance and is dropped, leaving only 4.0"
    );
}

/// Truncation copies the right columns of `U`, not a reindexed shuffle of them.
///
/// `u_out[i * keep + j] = u[i * k_full + j]` walks two different row strides. Replacing either
/// multiplication survived, because every truncation case had `keep == k_full`, a single row, or a
/// diagonal input where the stride error lands on a zero. Truncating a 3x3 to rank one exercises
/// both strides against a full decomposition of the same matrix.
#[test]
fn test_truncation_keeps_the_leading_columns_of_the_full_factor() {
    let m: DenseMatrix<f64> =
        DenseMatrix::from_vec(vec![4.0, 1.0, 0.0, 1.0, 3.0, 1.0, 0.0, 1.0, 2.0], 3, 3).unwrap();

    let (u_full, s_full, _) = svd(&m).unwrap();
    let (u_cut, s_cut, vt_cut) = svd_truncated(&m, &Truncation::Rank(1)).unwrap();

    assert_eq!(s_cut.len(), 1);
    assert!((s_cut.as_slice()[0] - s_full.as_slice()[0]).abs() < 1e-12);
    assert_eq!(u_cut.rows(), 3);
    assert_eq!(u_cut.cols(), 1);
    assert_eq!(vt_cut.rows(), 1);

    for i in 0..3 {
        assert!(
            (u_cut.get(i, 0).unwrap() - u_full.get(i, 0).unwrap()).abs() < 1e-12,
            "row {i} of the truncated U must be the full U's leading column: expected {}, got {}",
            u_full.get(i, 0).unwrap(),
            u_cut.get(i, 0).unwrap()
        );
    }

    // Rank two as well, and that is the case that pins the output stride. At `keep == 1` the
    // stride is one and `i * keep` cannot be told from `i / keep`; the mutant on it survived a
    // rank-one check. Rank two gives output rows at 0, 2, 4 against 0, 0, 1 under the mutation.
    let (u_two, s_two, vt_two) = svd_truncated(&m, &Truncation::Rank(2)).unwrap();
    assert_eq!(s_two.len(), 2);
    assert_eq!(u_two.cols(), 2);
    assert_eq!(vt_two.rows(), 2);
    for i in 0..3 {
        for j in 0..2 {
            assert!(
                (u_two.get(i, j).unwrap() - u_full.get(i, j).unwrap()).abs() < 1e-12,
                "U[{i}][{j}] of the rank-two truncation must match the full factor: expected {}, got {}",
                u_full.get(i, j).unwrap(),
                u_two.get(i, j).unwrap()
            );
        }
    }
}

// ---- mutation-driven: every eigen case above is 2x2 --------------------------------------------

/// The eigendecomposition of a 5x5, checked against the three properties that define one.
///
/// Every `eigen_hermitian` case in this file is 2x2, and one of those is diagonal, where the
/// rotation body never executes. Mutation testing left 22 survivors inside `sym_eig`'s Jacobi
/// rotation as a result.
///
/// A 2x2 cannot see them. The angle enters only through `t`, and `c = 1/√(t²+1)` with `s = t·c`
/// gives `c² + s² = 1` whatever `t` is, so any mutation of the angle leaves the rotation
/// orthogonal and the sweep still converges on a matrix that small. At 5x5 the cyclic sweep has to
/// undo fill-in from earlier rotations, and a wrong angle stops converging: the mutation
/// `app - aqq` to `app / aqq` reconstructs to 3.12 here against 1.1e-14 clean, and the index
/// mutation `a[p*n + p]` to `a[p + n + p]` reconstructs to 1.4e-5.
///
/// The reconstruction is what catches them. Both mutations preserve the trace and leave `V`
/// orthogonal to 1e-14, because an orthogonal similarity does that no matter which angle it turns
/// through. Only `A = V Λ Vᵀ` distinguishes a converged sweep from an unconverged one.
#[test]
fn test_the_eigendecomposition_of_a_five_by_five_satisfies_its_defining_properties() {
    const N: usize = 5;
    #[rustfmt::skip]
    let entries = vec![
        6.0, 2.0, 1.0, 3.0, 1.0,
        2.0, 7.0, 2.0, 1.0, 4.0,
        1.0, 2.0, 8.0, 2.0, 1.0,
        3.0, 1.0, 2.0, 9.0, 3.0,
        1.0, 4.0, 1.0, 3.0, 5.0,
    ];
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(entries.clone(), N, N).unwrap();
    let (vals, v) = eigen_hermitian(&m).unwrap();

    // 1. A = V Λ Vᵀ. The definition, and the only one of the three that sees an unconverged sweep.
    for i in 0..N {
        for j in 0..N {
            let mut acc = 0.0_f64;
            for k in 0..N {
                acc += v.get(i, k).unwrap() * vals.as_slice()[k] * v.get(j, k).unwrap();
            }
            assert!(
                (acc - entries[i * N + j]).abs() < 1e-12,
                "(V Λ Vᵀ)[{i}][{j}] expected {}, got {acc}",
                entries[i * N + j]
            );
        }
    }

    // 2. The eigenvectors are orthonormal: VᵀV = I.
    for i in 0..N {
        for j in 0..N {
            let mut acc = 0.0_f64;
            for k in 0..N {
                acc += v.get(k, i).unwrap() * v.get(k, j).unwrap();
            }
            let want = if i == j { 1.0 } else { 0.0 };
            assert!(
                (acc - want).abs() < 1e-12,
                "(VᵀV)[{i}][{j}] expected {want}, got {acc}"
            );
        }
    }

    // 3. The trace is the sum of the eigenvalues.
    let trace: f64 = (0..N).map(|i| entries[i * N + i]).sum();
    let sum: f64 = vals.as_slice().iter().sum();
    assert!(
        (trace - sum).abs() < 1e-12,
        "trace {trace} against the eigenvalue sum {sum}"
    );
}
