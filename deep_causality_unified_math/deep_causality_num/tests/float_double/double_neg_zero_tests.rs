/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Sign-of-zero tests for the `Float` implementation on `Float106`.
//!
//! IEEE 754 keeps the two zeros apart: every odd elementary function maps `-0.0` to `-0.0`,
//! `abs` clears the sign bit, `signum` reads it, and the sign survives a square or cube root.
//! The double-double implementation lost it in two ways. A sign test written `self.hi < 0.0`
//! is false for `-0.0`, so `abs`, `signum`, `sqrt`, `cbrt` and `tanh` treated a negative zero
//! as positive; and the double-double renormalisation ends in `quick_two_sum`, where
//! `-0.0 + 0.0` is `+0.0`, so any function that reached zero through the series or the
//! algebra came back with the sign cleared.
//!
//! Every expectation here is the value `f64` returns for the same argument.

use deep_causality_num::{Float, Float106};

/// A result that must be a negative zero: zero in both words, sign bit set.
fn assert_neg_zero(got: Float106, what: &str) {
    assert_eq!(got.hi(), 0.0, "{what}: must be zero, got {:?}", got.hi());
    assert_eq!(got.lo(), 0.0, "{what}: low word must be zero");
    assert!(
        got.hi().is_sign_negative(),
        "{what}: sign bit must be set, got +0.0"
    );
}

/// A result that must be a positive zero: zero in both words, sign bit clear.
fn assert_pos_zero(got: Float106, what: &str) {
    assert_eq!(got.hi(), 0.0, "{what}: must be zero, got {:?}", got.hi());
    assert_eq!(got.lo(), 0.0, "{what}: low word must be zero");
    assert!(
        got.hi().is_sign_positive(),
        "{what}: sign bit must be clear, got -0.0"
    );
}

fn neg_zero() -> Float106 {
    <Float106 as Float>::neg_zero()
}

fn pos_zero() -> Float106 {
    Float106::from(0.0)
}

// =============================================================================
// The reported defect: tanh
// =============================================================================

#[test]
fn test_tanh_neg_zero() {
    // `let negative = self.hi < 0.0` is false for -0.0, so the negation was skipped.
    assert_neg_zero(Float::tanh(neg_zero()), "tanh(-0.0)");
    assert!(f64::tanh(-0.0).is_sign_negative());
}

#[test]
fn test_tanh_pos_zero() {
    assert_pos_zero(Float::tanh(pos_zero()), "tanh(+0.0)");
}

#[test]
fn test_tanh_stays_odd_away_from_zero() {
    let x = Float106::from(0.75);
    let a = Float::tanh(x);
    let b = Float::tanh(-x);
    assert_eq!(a.hi(), -b.hi(), "tanh must stay odd");
    assert_eq!(a.lo(), -b.lo(), "tanh must stay odd in the low word");
}

// =============================================================================
// The same sign test, copied into its neighbours
// =============================================================================

#[test]
fn test_abs_neg_zero_clears_the_sign() {
    assert_pos_zero(Float::abs(neg_zero()), "abs(-0.0)");
    assert!(f64::abs(-0.0).is_sign_positive());
}

#[test]
fn test_signum_neg_zero_is_minus_one() {
    // The trait documents -1.0 for a negative number, -0.0 included; f32, f64 and BFloat16
    // all return it. The old zero branch answered 0.0 for both signs.
    assert_eq!(Float::signum(neg_zero()).hi(), -1.0);
    assert_eq!(f64::signum(-0.0), -1.0);
}

#[test]
fn test_signum_pos_zero_is_one() {
    assert_eq!(Float::signum(pos_zero()).hi(), 1.0);
    assert_eq!(f64::signum(0.0), 1.0);
}

#[test]
fn test_sqrt_neg_zero() {
    assert_neg_zero(Float::sqrt(neg_zero()), "sqrt(-0.0)");
    assert!(f64::sqrt(-0.0).is_sign_negative());
}

#[test]
fn test_sqrt_negative_is_still_nan() {
    assert!(Float::sqrt(Float106::from(-4.0)).is_nan());
}

#[test]
fn test_cbrt_neg_zero() {
    assert_neg_zero(Float::cbrt(neg_zero()), "cbrt(-0.0)");
    assert!(f64::cbrt(-0.0).is_sign_negative());
}

#[test]
fn test_cbrt_negative_is_still_negative() {
    assert_eq!(Float::cbrt(Float106::from(-8.0)).hi(), -2.0);
}

#[test]
fn test_copysign_neg_zero_magnitude() {
    // copysign goes through abs, so the cleared sign bit has to survive it.
    assert_pos_zero(
        Float::copysign(neg_zero(), Float106::from(1.0)),
        "copysign(-0.0, +1.0)",
    );
    assert_neg_zero(
        Float::copysign(pos_zero(), Float106::from(-1.0)),
        "copysign(+0.0, -1.0)",
    );
}

// =============================================================================
// Odd functions that reached zero through the series or the algebra
// =============================================================================

#[test]
fn test_sin_neg_zero() {
    assert_neg_zero(Float::sin(neg_zero()), "sin(-0.0)");
}

#[test]
fn test_cos_neg_zero_is_one() {
    let c = Float::cos(neg_zero());
    assert_eq!(c.hi(), 1.0);
    assert_eq!(c.lo(), 0.0);
}

#[test]
fn test_tan_neg_zero() {
    assert_neg_zero(Float::tan(neg_zero()), "tan(-0.0)");
}

#[test]
fn test_asin_neg_zero() {
    assert_neg_zero(Float::asin(neg_zero()), "asin(-0.0)");
}

#[test]
fn test_atan_neg_zero() {
    assert_neg_zero(Float::atan(neg_zero()), "atan(-0.0)");
}

#[test]
fn test_atan2_neg_zero_over_positive() {
    assert_neg_zero(
        Float::atan2(neg_zero(), Float106::from(1.0)),
        "atan2(-0.0, 1.0)",
    );
    assert_pos_zero(
        Float::atan2(pos_zero(), Float106::from(1.0)),
        "atan2(+0.0, 1.0)",
    );
}

#[test]
fn test_atan2_zero_over_negative_is_signed_pi() {
    let minus_pi = Float::atan2(neg_zero(), Float106::from(-1.0));
    assert_eq!(minus_pi.hi(), f64::atan2(-0.0, -1.0));
    let plus_pi = Float::atan2(pos_zero(), Float106::from(-1.0));
    assert_eq!(plus_pi.hi(), f64::atan2(0.0, -1.0));
}

#[test]
fn test_sinh_neg_zero() {
    assert_neg_zero(Float::sinh(neg_zero()), "sinh(-0.0)");
}

#[test]
fn test_asinh_neg_zero() {
    assert_neg_zero(Float::asinh(neg_zero()), "asinh(-0.0)");
}

#[test]
fn test_atanh_neg_zero() {
    assert_neg_zero(Float::atanh(neg_zero()), "atanh(-0.0)");
}

#[test]
fn test_exp_m1_neg_zero() {
    assert_neg_zero(Float::exp_m1(neg_zero()), "exp_m1(-0.0)");
}

#[test]
fn test_ln_1p_neg_zero() {
    assert_neg_zero(Float::ln_1p(neg_zero()), "ln_1p(-0.0)");
}

#[test]
fn test_to_degrees_neg_zero() {
    assert_neg_zero(Float::to_degrees(neg_zero()), "to_degrees(-0.0)");
}

#[test]
fn test_to_radians_neg_zero() {
    assert_neg_zero(Float::to_radians(neg_zero()), "to_radians(-0.0)");
}

// =============================================================================
// The zero-high-word / negative-low-word pair
// =============================================================================

#[test]
fn test_normalisation_forbids_a_zero_high_word_with_a_non_zero_low_word() {
    // A canonical `Float106` cannot hold `hi == 0.0` alongside a non-zero `lo`: `new` runs
    // `quick_two_sum`, and a sum of zero means the two words cancelled exactly, which leaves
    // the correction at zero. So the `hi == 0.0 && lo < 0.0` clause in the sign tests is
    // reachable only through `from_raw`, which puts the invariant on the caller.
    let x = Float106::new(0.0, -1.0e-30);
    assert_eq!(x.hi(), -1.0e-30, "the low word became the high word");
    assert_eq!(x.lo(), 0.0);

    let y = Float106::from(0.0) - Float106::from(0.0);
    assert_eq!(y.lo(), 0.0);
}

#[test]
fn test_from_raw_negative_low_word_reads_as_negative() {
    // Out-of-contract input, kept working because the sign tests already carried the clause.
    let x = Float106::from_raw(0.0, -1.0e-30);
    assert_eq!(Float::signum(x).hi(), -1.0);
    assert!(
        Float::tanh(x).hi().is_sign_negative(),
        "tanh must follow the low word when the high word is +0.0"
    );
}
