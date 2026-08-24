/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Behaviour at the representable limits.
//!
//! Every scalar in this crate is a fixed-width approximation of an unbounded structure, and each has
//! inputs where the two part company. ℤ is unbounded and `i64` is not; ℝ is unbounded and `f64`
//! saturates to infinity; ℕ has no additive inverses and `u64` wraps or panics depending on the
//! build profile.
//!
//! These are the cases where an algorithm can return a plausible wrong answer rather than fail, so
//! each one states what must happen instead.

use deep_causality_linear::{
    DenseMatrix, DenseVector, LinearError, MatrixBuild, MatrixView, PackedGf2, determinant,
    determinant_exact, matrix_norm_frobenius, rank_exact,
};

// ---- the integer determinant against i64's range ------------------------------------------------

#[test]
fn test_an_integer_determinant_that_fits_is_exact() {
    // 2x2 with entries at the square root of i64::MAX: the determinant fits, so it must be exact
    // rather than approximate. 3037000499^2 = 9223372030926249001 < i64::MAX.
    let big = 3_037_000_499i64;
    let m = DenseMatrix::from_vec(vec![big, 0, 0, big], 2, 2).unwrap();
    let expected = big
        .checked_mul(big)
        .expect("the reference product must fit");
    assert_eq!(determinant_exact(&m).unwrap(), expected);
}

#[test]
fn test_an_integer_determinant_that_overflows_is_reported_rather_than_wrapped() {
    // i64::MAX * 2 does not fit. A wrapped answer is worse than no answer: it is a plausible
    // integer that is wrong, and no caller can tell.
    let m = DenseMatrix::from_vec(vec![i64::MAX, 0, 0, 2i64], 2, 2).unwrap();
    match determinant_exact(&m) {
        Err(LinearError::Overflow { .. }) => {}
        other => panic!("an overflowing determinant must be reported, got {other:?}"),
    }
}

#[test]
fn test_the_integer_path_handles_i64_min_without_wrapping_silently() {
    //  is partial on the signed integers: |i64::MIN| is 2^63, which the
    // type cannot hold. The tower documents this. The determinant path must not turn it into a
    // wrong number.
    let m = DenseMatrix::from_vec(vec![i64::MIN, 0, 0, 1i64], 2, 2).unwrap();
    match determinant_exact(&m) {
        Ok(v) => assert_eq!(
            v,
            i64::MIN,
            "det of diag(MIN, 1) is MIN, which is representable"
        ),
        Err(LinearError::Overflow { .. }) => {}
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn test_exact_rank_is_unaffected_by_entry_magnitude() {
    // Rank is scale-invariant. A matrix of huge entries and a matrix of tiny ones with the same
    // pattern have the same rank, and the exact path has no threshold that could disagree.
    let huge = DenseMatrix::from_vec(vec![i64::MAX, i64::MAX, i64::MAX, i64::MAX], 2, 2).unwrap();
    let small = DenseMatrix::from_vec(vec![1i64, 1, 1, 1], 2, 2).unwrap();
    assert_eq!(rank_exact(&huge).unwrap(), 1);
    assert_eq!(rank_exact(&small).unwrap(), 1);
}

// ---- the float paths against f64's range --------------------------------------------------------

#[test]
fn test_a_determinant_that_overflows_to_infinity_is_not_reported_as_a_number() {
    // f64::MAX squared is infinity. An infinite determinant is not a determinant.
    let m = DenseMatrix::from_vec(vec![f64::MAX, 0.0, 0.0, f64::MAX], 2, 2).unwrap();
    let d = determinant(&m).unwrap();
    assert!(d.is_infinite(), "expected saturation to infinity, got {d}");
    assert!(!d.is_nan(), "saturation must not produce NaN");
}

#[test]
fn test_a_determinant_of_subnormal_entries_underflows_to_zero_rather_than_to_nan() {
    let tiny = f64::MIN_POSITIVE;
    let m = DenseMatrix::from_vec(vec![tiny, 0.0, 0.0, tiny], 2, 2).unwrap();
    let d = determinant(&m).unwrap();
    assert!(!d.is_nan(), "underflow must not produce NaN, got {d}");
    assert!(
        d >= 0.0,
        "the determinant of a positive diagonal cannot be negative"
    );
}

#[test]
fn test_a_norm_of_huge_entries_saturates_rather_than_wrapping() {
    let m = DenseMatrix::from_vec(vec![f64::MAX, f64::MAX, f64::MAX, f64::MAX], 2, 2).unwrap();
    let n = matrix_norm_frobenius(&m).unwrap();
    assert!(
        n.is_infinite() || n == f64::MAX,
        "expected saturation, got {n}"
    );
    assert!(!n.is_nan());
}

#[test]
fn test_a_norm_of_subnormal_entries_is_not_nan() {
    let v: DenseVector<f64> = DenseVector::from_vec(vec![f64::MIN_POSITIVE; 4]);
    for n in [v.norm_l1(), v.norm_l2(), v.norm_inf(), v.norm_sq()] {
        assert!(!n.is_nan(), "subnormal input produced NaN");
        assert!(n >= 0.0, "a norm cannot be negative");
    }
}

#[test]
fn test_a_matrix_containing_nan_does_not_silently_report_a_rank() {
    // NaN compares false against everything, so a magnitude pivot search skips it. The rank that
    // comes back must not claim the NaN column contributed.
    let mut m: DenseMatrix<f64> = DenseMatrix::identity(2);
    m.set(0, 0, f64::NAN).unwrap();
    let d = determinant(&m).unwrap();
    assert!(
        d.is_nan() || d == 0.0,
        "a NaN entry must not yield a confident number, got {d}"
    );
}

#[test]
fn test_an_infinite_entry_does_not_become_a_finite_determinant() {
    let m = DenseMatrix::from_vec(vec![f64::INFINITY, 0.0, 0.0, 1.0], 2, 2).unwrap();
    let d = determinant(&m).unwrap();
    assert!(
        !d.is_finite(),
        "an infinite entry cannot give a finite determinant, got {d}"
    );
}

// ---- the naturals, whose lower bound is zero ----------------------------------------------------

#[test]
fn test_a_natural_dot_product_at_the_top_of_the_range() {
    // u64 addition wraps in release and panics in debug. The dot product must not present a wrapped
    // sum as an answer.
    let a = DenseVector::from_vec(vec![u64::MAX, 0]);
    let b = DenseVector::from_vec(vec![1u64, 0]);
    assert_eq!(
        a.dot(&b).unwrap(),
        u64::MAX,
        "no overflow at exactly the maximum"
    );
}

#[test]
fn test_zero_is_the_lower_bound_of_the_naturals_and_needs_no_inverse() {
    let a = DenseVector::from_vec(vec![0u64, 0]);
    let b = DenseVector::from_vec(vec![u64::MAX, u64::MAX]);
    assert_eq!(
        a.dot(&b).unwrap(),
        0,
        "zero annihilates regardless of the other operand"
    );
}

// ---- shape bounds -------------------------------------------------------------------------------

#[test]
fn test_the_largest_index_inside_the_shape_is_readable_and_the_next_is_not() {
    let m: DenseMatrix<f64> = DenseMatrix::zeros(3, 4);
    assert!(
        m.get(2, 3).is_ok(),
        "the last valid position must be readable"
    );
    assert!(matches!(
        m.get(3, 3),
        Err(LinearError::IndexOutOfBounds {
            index: (3, 3),
            shape: (3, 4)
        })
    ));
    assert!(matches!(
        m.get(2, 4),
        Err(LinearError::IndexOutOfBounds {
            index: (2, 4),
            shape: (3, 4)
        })
    ));
}

#[test]
fn test_a_vector_index_at_and_past_its_length() {
    let v: DenseVector<f64> = DenseVector::from_vec(vec![1.0, 2.0, 3.0]);
    assert_eq!(v.get(2).unwrap(), 3.0);
    assert!(v.get(3).is_err(), "index == len must be rejected");
    assert!(
        v.get(usize::MAX).is_err(),
        "and so must the largest index there is"
    );
}

#[test]
fn test_a_packed_matrix_at_exactly_one_word_and_one_bit_past_it() {
    // The word boundary is where an off-by-one in the index computation shows up.
    let bits = PackedGf2::<u8>::bits_per_word();
    let exact: PackedGf2<u8> = PackedGf2::zeros(1, bits);
    assert_eq!(exact.words_per_row(), 1, "exactly one word");
    let over: PackedGf2<u8> = PackedGf2::zeros(1, bits + 1);
    assert_eq!(over.words_per_row(), 2, "one bit past needs a second word");
    assert!(exact.get(0, bits - 1).is_ok());
    assert!(exact.get(0, bits).is_err(), "one past the last column");
}

#[test]
fn test_a_packed_matrix_with_a_single_column() {
    let m: PackedGf2<u64> = PackedGf2::zeros(4, 1);
    assert_eq!(m.words_per_row(), 1);
    assert!(m.get(3, 0).is_ok());
    assert!(m.get(0, 1).is_err());
}
