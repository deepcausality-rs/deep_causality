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
