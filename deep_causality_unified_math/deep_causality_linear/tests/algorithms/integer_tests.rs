/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The integer path: exact, cubic, and never leaving ℤ.

use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{
    DenseMatrix, LinearError, LinearErrorEnum, determinant_exact, rank_exact,
};

fn dense_i64(f: (Vec<i64>, usize, usize)) -> DenseMatrix<i64> {
    let (d, r, c) = f;
    DenseMatrix::from_vec(d, r, c).unwrap()
}

#[test]
fn test_an_integer_determinant_is_an_integer() {
    let m = dense_i64(integer_determinant_4x4());
    let det: i64 = determinant_exact(&m).unwrap();
    assert_eq!(det, INTEGER_DETERMINANT_4X4);
    assert_eq!(det, 5);
}

#[test]
fn test_the_integer_determinant_matches_the_recurrence() {
    // D_n = 2 D_{n-1} - D_{n-2}, computed independently of this crate.
    let m = dense_i64(integer_determinant_4x4());
    let (mut prev, mut cur) = (1i64, 2i64);
    for _ in 2..=4 {
        let next = 2 * cur - prev;
        prev = cur;
        cur = next;
    }
    assert_eq!(determinant_exact(&m).unwrap(), cur);
}

#[test]
fn test_the_integer_determinant_agrees_with_the_float_path() {
    use deep_causality_linear::determinant;
    let (d, r, c) = integer_determinant_4x4();
    let exact = determinant_exact(&dense_i64((d.clone(), r, c))).unwrap();
    let as_float: Vec<f64> = d.iter().map(|&v| v as f64).collect();
    let float = determinant(&DenseMatrix::from_vec(as_float, r, c).unwrap()).unwrap();
    assert!(
        (float - exact as f64).abs() < 1e-9,
        "exact {exact}, float {float}"
    );
}

#[test]
fn test_the_integer_determinant_of_a_singular_matrix_is_zero() {
    let m = DenseMatrix::from_vec(vec![1i64, 2, 2, 4], 2, 2).unwrap();
    assert_eq!(determinant_exact(&m).unwrap(), 0);
}

#[test]
fn test_the_integer_determinant_rejects_a_non_square_matrix() {
    let m = DenseMatrix::from_vec(vec![1i64, 2, 3, 4, 5, 6], 2, 3).unwrap();
    assert!(matches!(
        determinant_exact(&m),
        Err(LinearError(LinearErrorEnum::NotSquare { .. }))
    ));
}

#[test]
fn test_exact_rank_of_the_boundary_alphabet() {
    let m = dense_i64(boundary_alphabet_3x3());
    assert_eq!(rank_exact(&m).unwrap(), BOUNDARY_ALPHABET_3X3_RANK);
}

#[test]
fn test_exact_rank_carries_no_tolerance_and_gets_the_hard_case_right() {
    // A matrix whose scaling puts a singular value on the wrong side of a 1e-5 threshold. The exact
    // path has no threshold to be on the wrong side of.
    let m = DenseMatrix::from_vec(vec![1i64, 0, 0, 0, 1, 0, 0, 0, 1], 3, 3).unwrap();
    assert_eq!(rank_exact(&m).unwrap(), 3);
}

#[test]
fn test_the_integer_and_rational_ranks_agree() {
    // Rank is a fraction-field notion, so rank over Z equals rank over Q. The integer path computes
    // the characteristic-zero rank without leaving Z.
    use deep_causality_num_rational::Rational;
    let (d, r, c) = ranks_disagree_3x3();
    let integer = rank_exact(&dense_i64((d.clone(), r, c))).unwrap();
    let as_rational: Vec<Rational<i64>> = d.iter().map(|&v| Rational::new(v, 1)).collect();
    let mut rational_m = DenseMatrix::from_vec(as_rational, r, c).unwrap();
    let rational = deep_causality_linear::rref(&mut rational_m).unwrap().rank();
    assert_eq!(integer, rational);
    assert_eq!(integer, RANKS_DISAGREE_RATIONAL_RANK);
}

#[test]
fn test_the_integer_rank_and_the_mod_two_rank_differ_on_the_same_matrix() {
    // The divergence G-02 records. Neither path converts the matrix to the other's field.
    use deep_causality_linear::{PackedGf2, rank_gf2};
    let (d, r, c) = ranks_disagree_3x3();
    let over_z = rank_exact(&dense_i64((d.clone(), r, c))).unwrap();
    let packed: PackedGf2<u64> = PackedGf2::from_i64_mod2(&d, r, c).unwrap();
    let over_f2 = rank_gf2(&packed).unwrap();
    assert_eq!(over_z, RANKS_DISAGREE_RATIONAL_RANK);
    assert_eq!(over_f2, RANKS_DISAGREE_GF2_RANK);
    assert_ne!(over_z, over_f2, "the two ranks are different questions");
}

// ---- mutation-driven: every input above is a 3x3 of {-1, 0, 1} ---------------------------------
//
// Those three matrices share more than their size. Every row has content 1, no row is zero, no
// pivot search ever has to look past the diagonal, and the Bareiss divisor `prev` is `±1` at every
// step. Twenty-one mutants of `integer.rs` survived on them, and they are not twenty-one problems:
// the eleven lines that reduce a row by its content never executed at all, the row swap in each
// function never executed, and a divisor of one divides nothing. The tests below supply the inputs
// that make each of those load-bearing.

/// Content reduction, on the matrix the docstring names it for.
///
/// `rank_exact` divides each row through by its content before eliminating, and the stated reason
/// is overflow rather than speed: the fraction-free intermediates are products of entries, so a
/// matrix of large entries overflows on products whose difference is zero. Nothing exercised it.
/// Every fixture had rows of content 1, where `content != one` is false and the block is skipped.
///
/// Row 0 here has content 1 and is left alone; row 1 is three times row 0 and has content 3.
/// Reduced, the two rows are equal and the rank is 1. Unreduced, the first product is
/// `9_000_000_003 * 3_000_000_000`, which is `2.7e19` against an `i64` ceiling of `9.2e18`, so
/// `checked_mul` returns `None` and the call comes back `Err(Overflow)` instead of a rank.
#[test]
fn test_exact_rank_reduces_a_row_by_its_content_before_eliminating() {
    let m = DenseMatrix::from_vec(
        vec![
            3_000_000_000i64,
            3_000_000_001,
            9_000_000_000,
            9_000_000_003,
        ],
        2,
        2,
    )
    .unwrap();
    assert_eq!(rank_exact(&m).unwrap(), 1);
}

/// A zero row and a pivot that is not on the diagonal, which no earlier input had.
///
/// Two things here that the `{-1, 0, 1}` fixtures never produced. The content of row 0 is zero, so
/// the `!content.is_zero()` half of the guard is what stops a division by zero rather than
/// decoration. And column 0 has its first non-zero at row 1, so the pivot search returns `p = 1`
/// against `row = 0` and the swap runs.
///
/// The rank is 2: rows 1 and 2 are independent, since `2 * [1, 2, 3]` is `[2, 4, 6]` and not
/// `[2, 4, 7]`.
#[test]
fn test_exact_rank_of_a_matrix_with_a_zero_row_and_an_off_diagonal_pivot() {
    let m = DenseMatrix::from_vec(vec![0i64, 0, 0, 1, 2, 3, 2, 4, 7], 3, 3).unwrap();
    assert_eq!(rank_exact(&m).unwrap(), 2);
}

/// Rectangular and empty shapes, neither of which had been passed to the integer path.
///
/// The wide case stops on columns, the tall case stops on rows through the `row >= rows` break, and
/// the empty shapes return zero rather than an error. `rank_exact` was only ever called on squares.
#[test]
fn test_exact_rank_of_rectangular_and_empty_shapes() {
    let wide = DenseMatrix::from_vec(vec![1i64, 2, 3, 4, 5, 2, 4, 6, 8, 11], 2, 5).unwrap();
    assert_eq!(rank_exact(&wide).unwrap(), 2);

    let tall = DenseMatrix::from_vec(vec![1i64, 2, 2, 4, 3, 7, 0, 0, 5, 10], 5, 2).unwrap();
    assert_eq!(rank_exact(&tall).unwrap(), 2);

    for (r, c) in [(0usize, 0usize), (0, 3), (3, 0)] {
        let empty: DenseMatrix<i64> = DenseMatrix::from_vec(Vec::new(), r, c).unwrap();
        assert_eq!(rank_exact(&empty).unwrap(), 0, "shape {r}x{c}");
    }
}

/// Two dense 4x4s, where the Bareiss divisor is no longer one.
///
/// At 3x3 with entries in `{-1, 0, 1}` the divisor `prev` is `±1` at every step, so `div_euclid`
/// divides nothing and the elimination is a subtraction. A 4x4 with entries in the tens reaches a
/// third step whose divisor is a genuine 2x2 minor, which is what makes both the divisor's index
/// and the explicit zeroing of the sub-column observable.
///
/// The first matrix has rank 3: its last row is `[26, 40, 50, 64]`, which is
/// `[12, 18, 24, 30] + [7, 11, 13, 17] + [7, 11, 13, 17]`. The second has rank 4. Both ranks come
/// from exact rational elimination, computed outside this crate.
#[test]
fn test_exact_rank_of_dense_four_by_four_matrices() {
    #[rustfmt::skip]
    let deficient = DenseMatrix::from_vec(
        vec![
            12i64, 18, 24, 30,
             7,    11, 13, 17,
            19,    23, 29, 31,
            26,    40, 50, 64,
        ],
        4, 4,
    ).unwrap();
    assert_eq!(rank_exact(&deficient).unwrap(), 3);

    #[rustfmt::skip]
    let full = DenseMatrix::from_vec(
        vec![
            -5i64, -13, -26,   0,
           -13,     13, -10, -16,
            26,     26, -30,  32,
            30,    -23,   0,  -7,
        ],
        4, 4,
    ).unwrap();
    assert_eq!(rank_exact(&full).unwrap(), 4);
}

/// The determinant's own pivot search, which no earlier matrix reached.
///
/// `integer_determinant_4x4` is tridiagonal with 2 on the diagonal and the singular 2x2 is
/// `[[1, 2], [2, 4]]`, so `a[k][k]` is non-zero at every step of both and the swap below the
/// diagonal never runs. Here the first Bareiss step drives `a[1][1]` to `6*2 - 4*3 = 0`, the search
/// finds the pivot at row 2, the rows swap and the sign flips. That last part matters on its own:
/// no test had ever produced a negative `sign_negative`.
///
/// The determinant is 15, by cofactor expansion:
/// `2(6*9 - 7*4) - 3(4*9 - 7*1) + 5(4*4 - 6*1) = 52 - 87 + 50`.
#[test]
fn test_the_integer_determinant_pivots_when_a_later_diagonal_entry_is_zero() {
    let m = DenseMatrix::from_vec(vec![2i64, 3, 5, 4, 6, 7, 1, 4, 9], 3, 3).unwrap();
    assert_eq!(determinant_exact(&m).unwrap(), 15);
}

/// A dense 4x4 determinant, where the pivot search has more than one row to reject.
///
/// At 3x3 the search below a zero diagonal entry has a single candidate, so reading the wrong cell
/// still lands on the only row that could be chosen. A 4x4 separates the two: the correct search
/// finds a pivot and returns 330, while a search that reads across rows instead of down the column
/// finds none and reports the matrix singular.
///
/// 330 is from exact rational elimination, computed outside this crate.
#[test]
fn test_the_integer_determinant_of_a_dense_four_by_four() {
    #[rustfmt::skip]
    let m = DenseMatrix::from_vec(
        vec![
            -9i64, -6, -3,  3,
             2,     6, -6,  9,
             4,     5, -2,  1,
             4,     7, -8,  0,
        ],
        4, 4,
    ).unwrap();
    assert_eq!(determinant_exact(&m).unwrap(), 330);
}

/// A row swap below the first row, which the zero-row matrix above does not produce.
///
/// That matrix swaps rows 0 and 1, and at `row = 0` the swap's own index arithmetic — `row * cols`
/// — is zero whichever way it is written, so two mutants of it survived. Column 1 here is zero at
/// rows 0, 1 and 3 and `-2` at row 2, so the second pivot is found at row 2 against `row = 1` and
/// the swap runs with a non-zero base. The same step is the first whose Bareiss divisor is read
/// from a row other than the first.
///
/// The rank is 4, from exact rational elimination computed outside this crate.
#[test]
fn test_exact_rank_when_the_second_pivot_needs_a_swap() {
    #[rustfmt::skip]
    let m = DenseMatrix::from_vec(
        vec![
            -6i64,  0, -1, 7,
             9,     0, -7, 2,
            -9,    -2, -1, 3,
             9,     0,  2, 3,
        ],
        4, 4,
    ).unwrap();
    assert_eq!(rank_exact(&m).unwrap(), 4);
}
