/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The branches the rest of the suite never reaches.
//!
//! Coverage measurement found these, and each turned out to be worth a test on its own terms rather
//! than as a number: the eigendecomposition's rotation loop had never executed, because every eigen
//! test used a diagonal matrix and a diagonal matrix needs no rotation. A test that only ever
//! exercises the trivial path is a test that would pass against an algorithm which does nothing.

use deep_causality_linear::{
    DenseMatrix, DenseVector, LinearError, Lu, MatrixBuild, MatrixView, PackedGf2, Truncation,
    determinant_exact, eigen_hermitian, image_basis_gf2, inverse, kernel_basis_gf2, matrix_norm_l1,
    qr, rank_exact, rref, singular_values, solve, solve_lower, solve_upper, svd_truncated,
};
use deep_causality_num::Gf2;

fn dm(d: &[f64], r: usize, c: usize) -> DenseMatrix<f64> {
    DenseMatrix::from_vec(d.to_vec(), r, c).unwrap()
}

// ---- the eigendecomposition actually rotating ---------------------------------------------------

#[test]
fn test_eigen_of_a_matrix_that_needs_rotation() {
    // [2 1; 1 2] has eigenvalues 3 and 1, and needs a rotation to find them. A diagonal matrix does
    // not, which is why every earlier eigen test left the rotation loop unexecuted.
    let m = dm(&[2.0, 1.0, 1.0, 2.0], 2, 2);
    let (vals, _) = eigen_hermitian(&m).unwrap();
    let mut got = [vals.get(0).unwrap(), vals.get(1).unwrap()];
    got.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        (got[0] - 1.0).abs() < 1e-9,
        "smaller eigenvalue was {}",
        got[0]
    );
    assert!(
        (got[1] - 3.0).abs() < 1e-9,
        "larger eigenvalue was {}",
        got[1]
    );
}

#[test]
fn test_eigen_vectors_satisfy_their_definition() {
    // A v = lambda v, which is what an eigenvector is and what a rotation loop must produce.
    let m = dm(&[2.0, 1.0, 1.0, 2.0], 2, 2);
    let (vals, vecs) = eigen_hermitian(&m).unwrap();
    for k in 0..2 {
        let lambda = vals.get(k).unwrap();
        for i in 0..2 {
            let mut av = 0.0;
            for j in 0..2 {
                av += m.get(i, j).unwrap() * vecs.get(j, k).unwrap();
            }
            let lv = lambda * vecs.get(i, k).unwrap();
            assert!(
                (av - lv).abs() < 1e-9,
                "eigenpair {k} fails at row {i}: {av} vs {lv}"
            );
        }
    }
}

#[test]
fn test_eigen_of_a_three_by_three_with_off_diagonal_mass() {
    // The trace is preserved by any similarity transform, so it pins the eigenvalues collectively
    // without needing them individually.
    let m = dm(&[4.0, 1.0, 1.0, 1.0, 3.0, 1.0, 1.0, 1.0, 2.0], 3, 3);
    let (vals, _) = eigen_hermitian(&m).unwrap();
    let sum: f64 = (0..3).map(|i| vals.get(i).unwrap()).sum();
    assert!(
        (sum - 9.0).abs() < 1e-9,
        "trace must be preserved: got {sum}"
    );
}

#[test]
fn test_eigen_of_the_zero_matrix() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(3, 3);
    let (vals, _) = eigen_hermitian(&m).unwrap();
    for i in 0..3 {
        assert_eq!(vals.get(i).unwrap(), 0.0);
    }
}

// ---- the fraction-free path actually swapping ---------------------------------------------------

#[test]
fn test_the_integer_determinant_swaps_when_the_diagonal_is_zero() {
    // det = -1 for this permutation, and reaching it needs the pivot swap and the sign flip that
    // follows. Every earlier integer test used a matrix whose diagonal was already usable.
    let m = DenseMatrix::from_vec(vec![0i64, 1, 1, 0], 2, 2).unwrap();
    assert_eq!(determinant_exact(&m).unwrap(), -1);
}

#[test]
fn test_the_integer_determinant_of_a_larger_matrix_needing_a_swap() {
    // [0 1 0; 1 0 0; 0 0 2]: one swap, then a diagonal. det = -2.
    let m = DenseMatrix::from_vec(vec![0i64, 1, 0, 1, 0, 0, 0, 0, 2], 3, 3).unwrap();
    assert_eq!(determinant_exact(&m).unwrap(), -2);
}

#[test]
fn test_the_integer_determinant_is_zero_when_a_column_is_entirely_zero_below() {
    // No pivot exists, so the determinant is zero and the search reports it rather than dividing.
    let m = DenseMatrix::from_vec(vec![0i64, 1, 0, 0, 0, 1, 0, 0, 1], 3, 3).unwrap();
    assert_eq!(determinant_exact(&m).unwrap(), 0);
}

#[test]
fn test_exact_rank_skips_a_column_with_no_pivot() {
    // The middle column is entirely zero, so it contributes no rank and elimination moves on.
    let m = DenseMatrix::from_vec(vec![1i64, 0, 0, 0, 0, 1], 2, 3).unwrap();
    assert_eq!(rank_exact(&m).unwrap(), 2);
}

#[test]
fn test_exact_rank_of_an_empty_matrix_in_each_orientation() {
    for (r, c) in [(0usize, 0usize), (0, 3), (3, 0)] {
        let m: DenseMatrix<i64> = DenseMatrix::zeros(r, c);
        assert_eq!(rank_exact(&m).unwrap(), 0, "shape ({r}, {c})");
    }
}

#[test]
fn test_the_integer_determinant_of_the_empty_matrix_is_one() {
    let m: DenseMatrix<i64> = DenseMatrix::zeros(0, 0);
    assert_eq!(determinant_exact(&m).unwrap(), 1);
}

// ---- the exact elimination path -----------------------------------------------------------------

#[test]
fn test_exact_rref_skips_a_zero_column_and_still_reduces() {
    // The  when a column has no pivot, on the exact path rather than the stable one.
    let mut m = dm(&[1.0, 0.0, 2.0, 0.0, 0.0, 3.0], 2, 3);
    let reduced = rref(&mut m).unwrap();
    assert_eq!(reduced.rank(), 2);
    assert_eq!(
        reduced.pivot_columns(),
        &[0, 2],
        "column 1 is entirely zero"
    );
}

#[test]
fn test_exact_rref_leaves_an_already_zero_entry_alone() {
    // The  when the elimination factor is already zero, so no axpy is issued.
    let mut m: DenseMatrix<f64> = DenseMatrix::identity(3);
    let reduced = rref(&mut m).unwrap();
    assert_eq!(reduced.rank(), 3);
}

#[test]
fn test_exact_rref_stops_when_the_rows_run_out_before_the_columns() {
    let mut m = dm(&[1.0, 0.0, 0.0, 0.0], 1, 4);
    let reduced = rref(&mut m).unwrap();
    assert_eq!(reduced.rank(), 1, "one row can carry at most one pivot");
}

// ---- solve, its triangular forms, and inverse ---------------------------------------------------

#[test]
fn test_triangular_substitution_rejects_a_non_square_matrix() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0]);
    assert!(matches!(
        solve_lower(&a, &b),
        Err(LinearError::NotSquare { shape: (2, 3) })
    ));
    assert!(matches!(
        solve_upper(&a, &b),
        Err(LinearError::NotSquare { shape: (2, 3) })
    ));
}

#[test]
fn test_triangular_substitution_rejects_a_right_hand_side_of_the_wrong_length() {
    let a: DenseMatrix<f64> = DenseMatrix::identity(3);
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0]);
    assert!(matches!(
        solve_lower(&a, &b),
        Err(LinearError::LengthMismatch {
            expected: 3,
            found: 1
        })
    ));
    assert!(matches!(
        solve_upper(&a, &b),
        Err(LinearError::LengthMismatch {
            expected: 3,
            found: 1
        })
    ));
}

#[test]
fn test_inverse_rejects_a_non_square_matrix() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    assert!(matches!(
        inverse(&a),
        Err(LinearError::NotSquare { shape: (2, 3) })
    ));
}

#[test]
fn test_inverse_rejects_a_singular_matrix() {
    let a = dm(&[1.0, 2.0, 2.0, 4.0], 2, 2);
    assert!(matches!(inverse(&a), Err(LinearError::Singular { .. })));
}

#[test]
fn test_the_inverse_multiplies_back_to_the_identity() {
    let a = dm(&[4.0, 7.0, 2.0, 6.0], 2, 2);
    let inv = inverse(&a).unwrap();
    for i in 0..2 {
        for j in 0..2 {
            let mut acc = 0.0;
            for k in 0..2 {
                acc += a.get(i, k).unwrap() * inv.get(k, j).unwrap();
            }
            let expected = if i == j { 1.0 } else { 0.0 };
            assert!(
                (acc - expected).abs() < 1e-9,
                "A*A^-1 wrong at ({i}, {j}): {acc}"
            );
        }
    }
}

#[test]
fn test_lu_factor_rejects_a_non_square_matrix() {
    let a: DenseMatrix<f64> = DenseMatrix::zeros(3, 2);
    assert!(matches!(
        Lu::factor(&a),
        Err(LinearError::NotSquare { shape: (3, 2) })
    ));
}

#[test]
fn test_lu_apply_rejects_a_right_hand_side_of_the_wrong_length() {
    let a: DenseMatrix<f64> = DenseMatrix::identity(3);
    let lu = Lu::factor(&a).unwrap();
    let b: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0]);
    assert!(matches!(
        lu.apply(&b),
        Err(LinearError::LengthMismatch {
            expected: 3,
            found: 2
        })
    ));
}

#[test]
fn test_solve_of_a_one_by_one_system() {
    let a = dm(&[4.0], 1, 1);
    let b: DenseVector<f64> = DenseVector::from_vec(vec![8.0]);
    assert!((solve(&a, &b).unwrap().get(0).unwrap() - 2.0).abs() < 1e-12);
}

// ---- the truncation variants --------------------------------------------------------------------

#[test]
fn test_truncating_to_rank_zero_keeps_nothing() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(3);
    let (_, s, _) = svd_truncated(&m, &Truncation::Rank(0)).unwrap();
    assert_eq!(s.len(), 0);
}

#[test]
fn test_truncating_by_a_rank_past_the_available_count_keeps_them_all() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(2);
    let (_, s, _) = svd_truncated(&m, &Truncation::Rank(99)).unwrap();
    assert_eq!(s.len(), 2, "the cap cannot invent components");
}

#[test]
fn test_truncating_by_a_tolerance_above_every_value_keeps_nothing() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(2);
    let (_, s, _) = svd_truncated(&m, &Truncation::Tolerance(10.0)).unwrap();
    assert_eq!(s.len(), 0);
}

#[test]
fn test_the_singular_values_of_a_non_square_matrix() {
    let m = dm(&[1.0, 0.0, 0.0, 0.0, 2.0, 0.0], 2, 3);
    let s = singular_values(&m).unwrap();
    let mut got: Vec<f64> = (0..s.len()).map(|i| s.get(i).unwrap()).collect();
    got.sort_by(|a, b| b.partial_cmp(a).unwrap());
    assert!((got[0] - 2.0).abs() < 1e-9);
    assert!((got[1] - 1.0).abs() < 1e-9);
}

#[test]
fn test_qr_of_a_matrix_with_a_zero_column() {
    // The  branch, where a column contributes no direction.
    let m = dm(&[1.0, 0.0, 0.0, 0.0], 2, 2);
    let (q, r) = qr(&m).unwrap();
    assert_eq!(q.shape(), (2, 2));
    assert_eq!(r.shape(), (2, 2));
    assert!(
        r.get(1, 1).unwrap().abs() < 1e-12,
        "a zero column gives a zero diagonal entry"
    );
}

// ---- the GF(2) kernel with genuine free variables -----------------------------------------------

#[test]
fn test_the_gf2_kernel_sets_a_pivot_variable_from_a_free_column() {
    // [1 1] over F2: rank 1, one free column, and the kernel vector is [1, 1].
    let m: PackedGf2<u8> = PackedGf2::from_slice(&[Gf2::ONE, Gf2::ONE], 1, 2).unwrap();
    let k = kernel_basis_gf2(&m).unwrap();
    assert_eq!(k.cols(), 1);
    assert_eq!(
        k.get(0, 0).unwrap(),
        Gf2::ONE,
        "the pivot variable is set from the free column"
    );
    assert_eq!(k.get(1, 0).unwrap(), Gf2::ONE, "the free variable is one");
}

#[test]
fn test_the_gf2_image_basis_of_a_matrix_with_a_zero_column() {
    let m: PackedGf2<u8> =
        PackedGf2::from_slice(&[Gf2::ONE, Gf2::ZERO, Gf2::ZERO, Gf2::ZERO], 2, 2).unwrap();
    assert_eq!(image_basis_gf2(&m).unwrap().cols(), 1);
}

#[test]
fn test_the_gf2_kernel_of_a_full_rank_matrix_is_empty() {
    let m: PackedGf2<u8> = PackedGf2::identity(3);
    assert_eq!(kernel_basis_gf2(&m).unwrap().cols(), 0);
}

// ---- norms and conversions at their edges -------------------------------------------------------

#[test]
fn test_matrix_norms_of_an_empty_matrix() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(0, 0);
    assert_eq!(matrix_norm_l1(&m).unwrap(), 0.0);
}

#[test]
fn test_a_norm_over_a_sparse_matrix_reads_through_the_trait() {
    use deep_causality_linear::CsrMatrix;
    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, -3.0), (1, 1, 4.0)]).unwrap();
    assert_eq!(
        matrix_norm_l1(&m).unwrap(),
        4.0,
        "largest column sum of moduli"
    );
}

// ---- the last reachable error paths -------------------------------------------------------------

#[test]
fn test_the_sparse_build_trait_rejects_an_out_of_bounds_write() {
    use deep_causality_linear::CsrMatrix;
    let mut m: CsrMatrix<f64> = CsrMatrix::zeros(2, 2);
    assert!(matches!(
        m.set(5, 0, 1.0),
        Err(LinearError::IndexOutOfBounds {
            index: (5, 0),
            shape: (2, 2)
        })
    ));
    assert!(matches!(
        m.set(0, 5, 1.0),
        Err(LinearError::IndexOutOfBounds { .. })
    ));
}

#[test]
fn test_the_sparse_build_trait_replaces_an_existing_entry_in_place() {
    // The `replaced` branch: a write to a position that already stores something must overwrite it
    // rather than append a second entry at the same position.
    use deep_causality_linear::CsrMatrix;
    let mut m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0), (1, 1, 2.0)]).unwrap();
    m.set(1, 1, 9.0).unwrap();
    assert_eq!(m.values().len(), 2, "no duplicate position");
    assert_eq!(m.get_value_at(1, 1), 9.0);
    assert_eq!(m.get_value_at(0, 0), 1.0, "the other entry is untouched");
}

#[test]
fn test_the_sparse_build_trait_appends_a_new_entry() {
    // The `!replaced` branch: writing where nothing is stored adds a position.
    use deep_causality_linear::CsrMatrix;
    let mut m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.0)]).unwrap();
    m.set(1, 1, 5.0).unwrap();
    assert_eq!(m.values().len(), 2);
    assert_eq!(m.get_value_at(1, 1), 5.0);
}

#[test]
fn test_the_packed_matrix_rejects_an_out_of_bounds_write() {
    let mut m: PackedGf2<u8> = PackedGf2::zeros(2, 2);
    assert!(matches!(
        m.set(9, 0, Gf2::ONE),
        Err(LinearError::IndexOutOfBounds {
            index: (9, 0),
            shape: (2, 2)
        })
    ));
    assert!(matches!(
        m.set(0, 9, Gf2::ONE),
        Err(LinearError::IndexOutOfBounds { .. })
    ));
}

#[test]
fn test_the_mod_two_constructor_rejects_a_buffer_of_the_wrong_length() {
    let e = PackedGf2::<u8>::from_i64_mod2(&[1, 0, 1], 2, 2).unwrap_err();
    assert!(matches!(e, LinearError::ShapeMismatch { .. }), "got {e:?}");
}

#[test]
fn test_scaling_a_packed_matrix_by_zero_in_place_clears_every_word() {
    let mut m: PackedGf2<u8> = PackedGf2::identity(3);
    m *= Gf2::ZERO;
    for i in 0..3 {
        for j in 0..3 {
            assert_eq!(m.get(i, j).unwrap(), Gf2::ZERO, "at ({i}, {j})");
        }
    }
}

#[test]
fn test_cg_converges_on_the_final_check_after_the_last_iteration() {
    // The residual test after the loop, rather than at the top of one: give exactly the number of
    // iterations the solve needs, so the loop ends and the closing check is what accepts it.
    use deep_causality_linear::cg_solve;
    let laplacian = |v: &[f64]| -> Vec<f64> {
        let n = v.len();
        (0..n)
            .map(|i| {
                let left = if i == 0 { 0.0 } else { v[i - 1] };
                let right = if i + 1 == n { 0.0 } else { v[i + 1] };
                2.0 * v[i] - left - right
            })
            .collect()
    };
    let b = vec![1.0, 0.0, 1.0];
    // Three iterations is exactly the Krylov dimension of this 3x3 system.
    let x = cg_solve(laplacian, &b, 1e-9, 3).expect("converges within the Krylov dimension");
    for v in &x {
        assert!((v - 1.0).abs() < 1e-8, "solution was {x:?}");
    }
}

#[test]
fn test_cg_reports_a_breakdown_on_an_operator_that_is_not_positive_definite() {
    use deep_causality_linear::{CgFailure, cg_solve};
    // Negated Laplacian: symmetric, negative definite, so the curvature guard must trip.
    let negated = |v: &[f64]| -> Vec<f64> {
        let n = v.len();
        (0..n)
            .map(|i| {
                let left = if i == 0 { 0.0 } else { v[i - 1] };
                let right = if i + 1 == n { 0.0 } else { v[i + 1] };
                -(2.0 * v[i] - left - right)
            })
            .collect()
    };
    let b = vec![1.0, 0.0, 1.0];
    assert!(matches!(
        cg_solve(negated, &b, 1e-12, 50),
        Err(CgFailure::NotPositiveDefinite { .. })
    ));
}

#[test]
fn test_the_sparse_display_renders_a_matrix_with_stored_and_structural_entries() {
    use deep_causality_linear::CsrMatrix;
    let m = CsrMatrix::from_triplets(2, 2, &[(0, 0, 1.5), (1, 1, -2.25)]).unwrap();
    let s = format!("{m}");
    assert!(s.contains("CsrMatrix (2x2)"), "{s}");
    assert!(s.contains("1.500"), "three decimal places: {s}");
    assert!(s.contains("-2.250"), "{s}");
    assert!(s.contains("0.000"), "structural zeros are printed too: {s}");
}

#[test]
fn test_the_sparse_display_marks_a_zero_dimension_matrix() {
    use deep_causality_linear::CsrMatrix;
    let m: CsrMatrix<f64> = CsrMatrix::zeros(0, 3);
    assert_eq!(format!("{m}"), "CsrMatrix (0x3) [Empty]");
}

#[test]
fn test_the_strict_packing_of_a_matrix_that_stores_nothing() {
    use deep_causality_linear::{CsrMatrix, csr_to_packed_gf2_strict};
    let m: CsrMatrix<i8> = CsrMatrix::zeros(2, 2);
    let packed: PackedGf2<u8> = csr_to_packed_gf2_strict(&m).unwrap();
    for i in 0..2 {
        for j in 0..2 {
            assert_eq!(packed.get(i, j).unwrap(), Gf2::ZERO);
        }
    }
}
