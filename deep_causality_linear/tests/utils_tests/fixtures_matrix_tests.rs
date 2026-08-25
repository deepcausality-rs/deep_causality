/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The fixtures are the suite's oracle, so they are checked against arithmetic done by hand rather
//! than against this crate. A fixture that encodes a wrong answer makes every test using it agree
//! with a defect.

use deep_causality_linear::utils_tests::fixtures_matrix::*;

#[test]
fn test_rank_deficient_3x3_third_row_is_the_sum_of_the_first_two() {
    let (d, r, c) = rank_deficient_3x3();
    assert_eq!((r, c), (3, 3));
    assert_eq!(d.len(), 9);
    for j in 0..3 {
        assert_eq!(
            d[6 + j],
            d[j] + d[3 + j],
            "row 3 must be row 1 + row 2 at {j}"
        );
    }
    // The dependency puts the rank at 2 or below. A non-singular minor over the first two rows
    // puts it at 2 or above. Together they force the constant, rather than restating it.
    let independent_rows = if d[0] * d[4] - d[1] * d[3] == 0.0 {
        1
    } else {
        2
    };
    assert_eq!(RANK_DEFICIENT_3X3_RANK, independent_rows);
}

#[test]
fn test_unit_determinant_3x3_is_upper_unitriangular() {
    let (d, r, c) = unit_determinant_3x3();
    assert_eq!((r, c), (3, 3));
    for i in 0..3 {
        assert_eq!(d[i * 3 + i], 1.0, "diagonal must be one");
        for j in 0..i {
            assert_eq!(d[i * 3 + j], 0.0, "below the diagonal must be zero");
        }
    }
    // The determinant of a triangular matrix is the product of its diagonal.
    assert_eq!(UNIT_DETERMINANT_3X3, d[0] * d[4] * d[8]);
}

#[test]
fn test_zero_leading_entry_3x3_has_a_zero_at_0_0_and_is_non_singular() {
    let (d, _, _) = zero_leading_entry_3x3();
    assert_eq!(
        d[0], 0.0,
        "the (0,0) entry is what an unpivoted elimination trips on"
    );
    // A permutation matrix exchanging two rows. Expanded along the first row rather than read off
    // the constant it is checking.
    let m = |i: usize, j: usize| d[i * 3 + j];
    let det = m(0, 0) * (m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1))
        - m(0, 1) * (m(1, 0) * m(2, 2) - m(1, 2) * m(2, 0))
        + m(0, 2) * (m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0));
    assert_eq!(ZERO_LEADING_ENTRY_DETERMINANT, det);
    assert!(det < 0.0, "the exchange is what makes the sign negative");
}

#[test]
fn test_near_zero_pivot_2x2_has_a_much_larger_candidate_below() {
    let (d, _, _): (Vec<f64>, _, _) = near_zero_pivot_2x2();
    assert!(d[0].abs() < 1e-15, "first candidate is near zero");
    assert!(d[2].abs() > 0.5, "the candidate below it is not");
    assert!(
        d[2].abs() / d[0].abs() > 1e15,
        "the ratio is what makes it a conditioning test"
    );
}

#[test]
fn test_singular_2x2_second_row_is_twice_the_first() {
    let (d, _, _) = singular_2x2();
    assert_eq!(d[2], 2.0 * d[0]);
    assert_eq!(d[3], 2.0 * d[1]);
    // ad - bc = 1*4 - 2*2 = 0
    assert_eq!(d[0] * d[3] - d[1] * d[2], 0.0);
}

#[test]
fn test_boundary_alphabet_3x3_uses_only_minus_one_zero_one() {
    let (d, _, _) = boundary_alphabet_3x3();
    for v in &d {
        assert!(
            (-1..=1).contains(v),
            "outside the boundary-operator alphabet: {v}"
        );
    }
    for j in 0..3 {
        assert_eq!(d[6 + j], d[j] + d[3 + j]);
    }
    let independent_rows = if d[0] * d[4] - d[1] * d[3] == 0 { 1 } else { 2 };
    assert_eq!(BOUNDARY_ALPHABET_3X3_RANK, independent_rows);
}

#[test]
fn test_ranks_disagree_3x3_has_determinant_two() {
    let (d, _, _) = ranks_disagree_3x3();
    let m = |i: usize, j: usize| d[i * 3 + j];
    let det = m(0, 0) * (m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1))
        - m(0, 1) * (m(1, 0) * m(2, 2) - m(1, 2) * m(2, 0))
        + m(0, 2) * (m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0));
    assert_eq!(det, RANKS_DISAGREE_DETERMINANT);
    assert_eq!(det, 2, "non-zero over Q, so full rank there");
    assert_eq!(det % 2, 0, "zero mod 2, so the rank drops over F2");
}

#[test]
fn test_ranks_disagree_3x3_has_an_even_weight_dependency() {
    let (d, _, _) = ranks_disagree_3x3();
    // r1 + r2 + r3 is componentwise even, which is a dependency over F2 and not over Q.
    for j in 0..3 {
        let col_sum = d[j] + d[3 + j] + d[6 + j];
        assert_eq!(col_sum % 2, 0, "column {j} sum must be even");
        assert_ne!(
            col_sum, 0,
            "and must not be zero, or the dependency would hold over Q too"
        );
    }
    // Over Q the determinant is non-zero, so the rank is full; over F2 the dependency drops it by
    // one, and a minor that survives reduction mod 2 stops it dropping further.
    let m = |i: usize, j: usize| d[i * 3 + j];
    let det = m(0, 0) * (m(1, 1) * m(2, 2) - m(1, 2) * m(2, 1))
        - m(0, 1) * (m(1, 0) * m(2, 2) - m(1, 2) * m(2, 0))
        + m(0, 2) * (m(1, 0) * m(2, 1) - m(1, 1) * m(2, 0));
    // A 3x3 with non-zero determinant has full rank; a zero one would have dropped below it.
    let rational_rank = if det == 0 { 2 } else { 3 };
    assert_eq!(RANKS_DISAGREE_RATIONAL_RANK, rational_rank);

    let minor_mod2 = (m(0, 0) * m(1, 1) - m(0, 1) * m(1, 0)).rem_euclid(2);
    assert_ne!(minor_mod2, 0, "a 2x2 minor must survive reduction mod 2");
    assert_eq!(RANKS_DISAGREE_GF2_RANK, RANKS_DISAGREE_RATIONAL_RANK - 1);
}

#[test]
fn test_integer_determinant_4x4_is_tridiagonal_with_determinant_five() {
    let (d, r, c) = integer_determinant_4x4();
    assert_eq!((r, c), (4, 4));
    for i in 0usize..4 {
        for j in 0usize..4 {
            let expected = match i.abs_diff(j) {
                0 => 2,
                1 => 1,
                _ => 0,
            };
            assert_eq!(d[i * 4 + j], expected, "at ({i}, {j})");
        }
    }
    // D_n = 2*D_{n-1} - D_{n-2} from D_1 = 2, D_2 = 3, so D_n = n + 1 and D_4 = 5.
    let (mut prev, mut cur) = (1i64, 2i64);
    for _ in 2..=4 {
        let next = 2 * cur - prev;
        prev = cur;
        cur = next;
    }
    assert_eq!(cur, INTEGER_DETERMINANT_4X4);
    assert_eq!(cur, 5);
}
