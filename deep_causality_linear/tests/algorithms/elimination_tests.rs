/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Elimination, and the Cayley-Menger regression that makes pivoting non-negotiable.

use deep_causality_linear::utils_tests::fixtures_cayley_menger::*;
use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{
    DenseMatrix, LinearError, LinearErrorEnum, MatrixBuild, MatrixView, determinant, image_basis,
    kernel_basis, rank, rank_stable, rref, rref_stable,
};

fn dense(f: (Vec<f64>, usize, usize)) -> DenseMatrix<f64> {
    let (d, r, c) = f;
    DenseMatrix::from_vec(d, r, c).unwrap()
}

// ---- rank --------------------------------------------------------------------------------------

#[test]
fn test_rank_of_a_known_rank_deficient_matrix() {
    let m = dense(rank_deficient_3x3());
    assert_eq!(rank_stable(&m).unwrap(), RANK_DEFICIENT_3X3_RANK);
}

#[test]
fn test_rank_of_the_identity_is_its_order() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(4);
    assert_eq!(rank_stable(&m).unwrap(), 4);
}

#[test]
fn test_rank_of_a_zero_matrix_is_zero() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(3, 3);
    assert_eq!(rank_stable(&m).unwrap(), 0);
}

#[test]
fn test_rank_of_an_empty_matrix_is_zero_rather_than_an_error() {
    for shape in [(0usize, 0usize), (0, 3), (3, 0)] {
        let m: DenseMatrix<f64> = DenseMatrix::zeros(shape.0, shape.1);
        assert_eq!(rank_stable(&m).unwrap(), 0, "shape {shape:?}");
    }
}

#[test]
fn test_a_zero_row_and_a_zero_column_do_not_contribute_rank() {
    let mut m: DenseMatrix<f64> = DenseMatrix::zeros(3, 3);
    m.set(0, 0, 1.0).unwrap();
    m.set(1, 1, 1.0).unwrap();
    // Row 2 and column 2 are entirely zero.
    assert_eq!(rank_stable(&m).unwrap(), 2);
}

#[test]
fn test_rank_of_a_non_square_matrix_in_both_orientations() {
    let wide: DenseMatrix<f64> =
        DenseMatrix::from_vec(vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0], 2, 3).unwrap();
    let tall: DenseMatrix<f64> =
        DenseMatrix::from_vec(vec![1.0, 0.0, 0.0, 1.0, 0.0, 0.0], 3, 2).unwrap();
    assert_eq!(rank_stable(&wide).unwrap(), 2);
    assert_eq!(rank_stable(&tall).unwrap(), 2);
}

// ---- rref --------------------------------------------------------------------------------------

#[test]
fn test_rref_reports_the_rank_and_the_pivot_columns_from_one_pass() {
    let mut m = dense(rank_deficient_3x3());
    let reduced = rref_stable(&mut m).unwrap();
    assert_eq!(reduced.rank(), 2);
    assert_eq!(reduced.pivot_columns().len(), 2);
    // Ascending, and each within the shape.
    let pivots = reduced.pivot_columns();
    assert!(
        pivots.windows(2).all(|w| w[0] < w[1]),
        "pivots must ascend: {pivots:?}"
    );
}

#[test]
fn test_rref_of_the_identity_is_the_identity() {
    use deep_causality_linear::MatrixView;
    let mut m: DenseMatrix<f64> = DenseMatrix::identity(3);
    let reduced = rref_stable(&mut m).unwrap();
    assert_eq!(reduced.rank(), 3);
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_eq!(m.get(i, j).unwrap(), expected);
        }
    }
}

// ---- the pivot rule ----------------------------------------------------------------------------

#[test]
fn test_a_near_zero_leading_pivot_does_not_become_the_pivot() {
    // The conditioning case: eliminating on 1e-18 loses the second row to rounding.
    let m = dense(near_zero_pivot_2x2());
    let det = determinant(&m).unwrap();
    // 1e-18 * 1 - 1 * 1, which is -1.0 in f64.
    assert!((det - (-1.0)).abs() < 1e-12, "determinant was {det}");
}

#[test]
fn test_a_matrix_with_a_zero_leading_entry_is_still_non_singular() {
    let m = dense(zero_leading_entry_3x3());
    let det = determinant(&m).unwrap();
    assert!(
        (det - ZERO_LEADING_ENTRY_DETERMINANT).abs() < 1e-12,
        "determinant was {det}"
    );
    assert_ne!(det, 0.0, "an unpivoted elimination returns zero here");
}

// ---- Cayley-Menger: the regression -------------------------------------------------------------

#[test]
fn test_the_tetrahedron_cayley_menger_determinant_is_four() {
    let m = dense(regular_unit_tetrahedron());
    let det = determinant(&m).unwrap();
    assert!(
        (det - TETRAHEDRON_CM_DETERMINANT).abs() < 1e-9,
        "determinant was {det}, must be 4.0; a determinant that takes the diagonal as its pivot returns 0.0 here"
    );
}

#[test]
fn test_the_tetrahedron_volume_is_root_two_over_twelve() {
    let m = dense(regular_unit_tetrahedron());
    let det = determinant(&m).unwrap();
    let vol = cm_determinant_to_volume_squared(det, 5).sqrt();
    assert!(
        (vol - TETRAHEDRON_VOLUME).abs() < 1e-12,
        "volume was {vol}, must be sqrt(2)/12; an unpivoted elimination gives NaN here"
    );
    assert!(!vol.is_nan(), "an unpivoted elimination produces NaN");
}

#[test]
fn test_the_right_triangle_cayley_menger_determinant_is_minus_four() {
    let m = dense(right_triangle());
    let det = determinant(&m).unwrap();
    assert!(
        (det - RIGHT_TRIANGLE_CM_DETERMINANT).abs() < 1e-9,
        "determinant was {det}"
    );
}

// ---- determinant -------------------------------------------------------------------------------

#[test]
fn test_determinant_of_a_triangular_matrix_is_the_product_of_its_diagonal() {
    let m = dense(unit_determinant_3x3());
    assert!((determinant(&m).unwrap() - UNIT_DETERMINANT_3X3).abs() < 1e-12);
}

#[test]
fn test_determinant_of_a_singular_matrix_is_zero() {
    let m = dense(singular_2x2());
    assert!(determinant(&m).unwrap().abs() < 1e-12);
}

#[test]
fn test_determinant_of_the_empty_matrix_is_one() {
    // The empty product.
    let m: DenseMatrix<f64> = DenseMatrix::zeros(0, 0);
    assert_eq!(determinant(&m).unwrap(), 1.0);
}

#[test]
fn test_determinant_rejects_a_non_square_matrix() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(2, 3);
    let e = determinant(&m).unwrap_err();
    assert!(
        matches!(e, LinearError(LinearErrorEnum::NotSquare { shape: (2, 3) })),
        "got {e:?}"
    );
}

#[test]
fn test_determinant_of_a_one_by_one_is_its_entry() {
    let mut m: DenseMatrix<f64> = DenseMatrix::zeros(1, 1);
    m.set(0, 0, -3.0).unwrap();
    assert_eq!(determinant(&m).unwrap(), -3.0);
}

#[test]
fn test_a_six_by_six_determinant_uses_elimination_rather_than_expansion() {
    // Cubic rather than factorial: at order 6 a Laplace expansion is 720 terms.
    let m: DenseMatrix<f64> = DenseMatrix::identity(6);
    assert!((determinant(&m).unwrap() - 1.0).abs() < 1e-12);
}

// ---- the exact entry points --------------------------------------------------------------------

#[test]
fn test_the_exact_and_stable_ranks_agree_on_a_well_conditioned_matrix() {
    let m = dense(rank_deficient_3x3());
    assert_eq!(rank(&m).unwrap(), rank_stable(&m).unwrap());
}

#[test]
fn test_rref_over_an_unordered_field_needs_no_epsilon() {
    // The exact entry point admits a field with no ordering. Rational<i64> is not a NormedScalar,
    // so it can only reach elimination through the exact rule -- which is the point of the split.
    use deep_causality_num_rational::Rational;
    let d: Vec<Rational<i64>> = vec![
        Rational::new(1, 2),
        Rational::new(1, 3),
        Rational::new(1, 3),
        Rational::new(1, 4),
    ];
    let mut m = DenseMatrix::from_vec(d, 2, 2).unwrap();
    let reduced = rref(&mut m).unwrap();
    assert_eq!(reduced.rank(), 2, "the Hilbert-like 2x2 is non-singular");
}

// ---- kernel and image, generic over the row-operation seam --------------------------------------

#[test]
fn test_the_kernel_basis_annihilates_and_is_the_right_size() {
    let m = dense(rank_deficient_3x3());
    let kernel: DenseMatrix<f64> = kernel_basis(&m).unwrap();
    let rank = rank(&m).unwrap();
    assert_eq!(kernel.cols(), m.cols() - rank, "cols - rank vectors");
    for k in 0..kernel.cols() {
        for i in 0..m.rows() {
            let mut acc = 0.0;
            for j in 0..m.cols() {
                acc += m.get(i, j).unwrap() * kernel.get(j, k).unwrap();
            }
            assert!(
                acc.abs() < 1e-9,
                "kernel vector {k} not annihilated at row {i}: {acc}"
            );
        }
    }
}

#[test]
fn test_a_full_rank_matrix_has_an_empty_kernel() {
    let m: DenseMatrix<f64> = DenseMatrix::identity(3);
    let kernel: DenseMatrix<f64> = kernel_basis(&m).unwrap();
    assert_eq!(kernel.cols(), 0);
}

#[test]
fn test_the_image_basis_has_rank_columns_taken_from_the_original() {
    let m = dense(rank_deficient_3x3());
    let image: DenseMatrix<f64> = image_basis(&m).unwrap();
    let rank = rank(&m).unwrap();
    assert_eq!(image.cols(), rank);
    assert_eq!(image.rows(), m.rows());
    // The columns are columns of the original, so the first is recognisable.
    for i in 0..m.rows() {
        assert_eq!(image.get(i, 0).unwrap(), m.get(i, 0).unwrap());
    }
}

#[test]
fn test_the_zero_matrix_has_a_full_kernel_and_an_empty_image() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(3, 4);
    let kernel: DenseMatrix<f64> = kernel_basis(&m).unwrap();
    let image: DenseMatrix<f64> = image_basis(&m).unwrap();
    assert_eq!(kernel.cols(), 4);
    assert_eq!(image.cols(), 0);
}

#[test]
fn test_rref_puts_the_matrix_in_reduced_row_echelon_form() {
    let mut m = dense(rank_deficient_3x3());
    let reduced = rref_stable(&mut m).unwrap();
    // Each pivot column holds a one in its own row and zeros elsewhere.
    for (row, &col) in reduced.pivot_columns().iter().enumerate() {
        assert!(
            (m.get(row, col).unwrap() - 1.0).abs() < 1e-9,
            "pivot at ({row}, {col}) not one"
        );
        for other in 0..m.rows() {
            if other != row {
                assert!(
                    m.get(other, col).unwrap().abs() < 1e-9,
                    "column {col} not cleared"
                );
            }
        }
    }
}

#[test]
fn test_the_determinant_of_a_singular_matrix_above_the_closed_forms_is_zero() {
    // Order four and above leaves the closed forms and runs elimination, where rank deficiency
    // shows up as a column with no pivot rather than as a product of pivots. Row 1 is twice row 0,
    // so the determinant is zero exactly.
    #[rustfmt::skip]
    let m = DenseMatrix::from_vec(
        vec![
            1.0, 2.0, 3.0, 4.0,
            2.0, 4.0, 6.0, 8.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ],
        4,
        4,
    )
    .unwrap();
    assert_eq!(rank_stable(&m).unwrap(), 3, "the fixture is rank deficient");
    assert_eq!(determinant(&m).unwrap(), 0.0);
}

#[test]
fn test_a_kernel_vector_leaves_a_pivot_variable_at_zero_when_the_free_column_does_not_couple_to_it()
{
    // The kernel of [[1,0,0],[0,0,1]] is spanned by e₂: x₀ and x₂ are pinned to zero and x₁ is
    // free. Neither pivot row carries a coefficient in the free column, so no pivot entry is
    // written and the basis vector is the free variable alone.
    let m = DenseMatrix::from_vec(vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0], 2, 3).unwrap();
    let kernel: DenseMatrix<f64> = kernel_basis(&m).unwrap();
    assert_eq!(kernel.shape(), (3, 1), "cols - rank vectors of length cols");
    assert_eq!(kernel.get(0, 0).unwrap(), 0.0);
    assert_eq!(kernel.get(1, 0).unwrap(), 1.0);
    assert_eq!(kernel.get(2, 0).unwrap(), 0.0);
    // And it is annihilated, which is what a kernel vector has to be.
    for i in 0..m.rows() {
        let mut acc = 0.0;
        for j in 0..m.cols() {
            acc += m.get(i, j).unwrap() * kernel.get(j, 0).unwrap();
        }
        assert_eq!(acc, 0.0, "row {i}");
    }
}

// ---- mutation-driven: both 3x3 determinant fixtures are sparse ---------------------------------

/// The 3x3 closed form, on a matrix where every term of it contributes.
///
/// `determinant` uses a closed form at order three: `a(ej − fi) − b(dj − fh) + c(di − eh)`. The
/// two 3x3 fixtures are `unit_determinant_3x3`, which is upper triangular, and
/// `zero_leading_entry_3x3`, a permutation matrix. Between them they leave most of those nine
/// products multiplying a zero, so mutating the closed form's arithmetic changed nothing anything
/// asserted. Five separate mutants of it survived, along with deletion of the whole arm.
///
/// The first matrix written here for that was `[[2, −3, 1], [2, 0, −1], [1, 4, 5]]`, which has a
/// zero at `e` and so left `e·j` and `e·h` still multiplying it. Two mutants survived the fix.
/// `[[2, 3, 5], [7, 11, 13], [17, 19, 23]]` has no zero anywhere and no two products equal:
/// `2(11·23 − 13·19) − 3(7·23 − 13·17) + 5(7·19 − 11·17) = 12 + 180 − 270 = −78`.
#[test]
fn test_the_three_by_three_determinant_on_a_dense_matrix() {
    let m: DenseMatrix<f64> =
        DenseMatrix::from_vec(vec![2.0, 3.0, 5.0, 7.0, 11.0, 13.0, 17.0, 19.0, 23.0], 3, 3)
            .unwrap();
    let det = determinant(&m).unwrap();
    assert!((det + 78.0).abs() < 1e-12, "expected -78, got {det}");
}

/// The closed form and the general elimination agree.
///
/// The arm exists for speed, not for a different answer, so deleting it must not change the
/// result. A 4x4 goes through elimination; embedding the 3x3 above as the leading block of a 4x4
/// with a unit trailing entry keeps the determinant at −78 and exercises the other path.
#[test]
fn test_the_three_by_three_closed_form_agrees_with_elimination() {
    #[rustfmt::skip]
    let embedded: DenseMatrix<f64> = DenseMatrix::from_vec(
        vec![
             2.0,  3.0,  5.0, 0.0,
             7.0, 11.0, 13.0, 0.0,
            17.0, 19.0, 23.0, 0.0,
             0.0,  0.0,  0.0, 1.0,
        ],
        4, 4,
    )
    .unwrap();
    let det = determinant(&embedded).unwrap();
    assert!(
        (det + 78.0).abs() < 1e-9,
        "the elimination path must agree with the closed form: expected -78, got {det}"
    );
}

// ---- mutation-driven: the pivot rule and the noise floor ---------------------------------------

/// The stable rule has to pick the largest candidate, not merely a non-zero one.
///
/// `pivot_stable_with` searches for the largest modulus at or below the row. Any non-zero pivot
/// gives a mathematically valid elimination, so nothing that only checks a result against exact
/// arithmetic can tell the rules apart, and three mutants of the selection survived: keep the
/// first candidate, keep the last, keep the smallest.
///
/// What separates them is round-off. Here row 1 is exactly twice row 2 in binary — `2 * 0.1` is
/// `0.2` to the bit, and likewise for 20 and 2000 — so the exact rank is 2. The largest entry in
/// column 0 is `20`, and pivoting there reports 2. Each of the three mutants pivots somewhere else:
/// keeping the first candidate or the smallest takes `-0.1`, keeping the last takes `10`. All three
/// come back with rank 3, because the dependency that should cancel exactly is left holding a
/// residue above the noise floor instead.
#[test]
fn test_the_stable_pivot_rule_keeps_a_rank_deficient_matrix_from_inflating() {
    #[rustfmt::skip]
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(
        vec![
            -0.1,  0.01, -1000.0,
            20.0,  0.2,   2000.0,
            10.0,  0.1,   1000.0,
        ],
        3, 3,
    ).unwrap();
    // Row 1 is exactly 2 x row 2, so two of the three rows are independent and no more.
    assert_eq!(
        rank_stable(&m).unwrap(),
        2,
        "a pivot chosen for position rather than magnitude inflates this to 3"
    );
}

/// A pivot can be small and still be real, and the floor has to leave it alone.
///
/// `negligible_below` returns `scale * (eps * n)^2`, where `scale` is the largest squared modulus.
/// Both factors of `eps * n` are needed, because `scale` is squared and the comparison against it
/// is squared too. Dropping to a single factor raises the floor by about `1e15`, and no earlier
/// matrix noticed: every pivot in the suite was within a factor of a few of the largest entry, so
/// a floor `1e15` times too large still sat far below all of them.
///
/// `diag(1, 1e-9, 1)` has a pivot whose squared modulus is `1e-18`. The correct floor is near
/// `4e-31` and passes it. A floor of `scale * eps * n` is near `1e-15` and swallows it, reporting
/// rank 2 for a matrix that is plainly invertible.
#[test]
fn test_a_small_but_genuine_pivot_is_not_swallowed_by_the_noise_floor() {
    #[rustfmt::skip]
    let m: DenseMatrix<f64> = DenseMatrix::from_vec(
        vec![
            1.0, 0.0,   0.0,
            0.0, 1e-9,  0.0,
            0.0, 0.0,   1.0,
        ],
        3, 3,
    ).unwrap();
    assert_eq!(
        rank_stable(&m).unwrap(),
        3,
        "1e-9 is nine orders above the double-precision floor and is a real pivot"
    );
}
