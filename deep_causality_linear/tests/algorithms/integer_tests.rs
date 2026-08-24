/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The integer path: exact, cubic, and never leaving ℤ.

use deep_causality_linear::utils_tests::fixtures_matrix::*;
use deep_causality_linear::{DenseMatrix, LinearError, determinant_exact, rank_exact};

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
        Err(LinearError::NotSquare { .. })
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
