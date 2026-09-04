/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the arithmetic operators on `BFloat16`.
//!
//! Every operand below is exactly representable, which `bf` asserts, so each expected result is
//! the correctly rounded value of an exact real operation, derived in the comment beside it. The
//! family test at the end compares against the exact `f64` result rounded once; `round_from_f64`
//! is established independently in `bfloat16_tests.rs`.

use deep_causality_num::BFloat16;

fn bf(x: f32) -> BFloat16 {
    let v = BFloat16::from(x);
    assert_eq!(
        v.to_f32(),
        x,
        "test operand {x} is not exactly representable"
    );
    v
}

// =============================================================================
// Negation
// =============================================================================

#[test]
fn test_neg_flips_the_sign_bit_only() {
    assert_eq!((-BFloat16::ONE).to_bits(), 0xBF80);
    assert_eq!((-BFloat16::ZERO).to_bits(), 0x8000);
    assert_eq!((-BFloat16::NEG_ZERO).to_bits(), 0x0000);
    assert_eq!(-BFloat16::INFINITY, BFloat16::NEG_INFINITY);
    let neg_nan = -BFloat16::NAN;
    assert!(neg_nan.is_nan());
    assert!(neg_nan.is_sign_negative());
    assert_eq!(neg_nan.to_bits(), 0xFFC0);
    assert_eq!((-(-BFloat16::from_bits(0x4049))).to_bits(), 0x4049);
}

// =============================================================================
// Addition
// =============================================================================

#[test]
fn test_add_exact() {
    // 1.5 + 2.25 = 3.75 = 11.11b, four significant bits.
    assert_eq!(bf(1.5) + bf(2.25), bf(3.75));
    assert_eq!(bf(-3.0) + bf(5.0), bf(2.0));
    assert_eq!(bf(100.0) + bf(28.0), bf(128.0));
}

#[test]
fn test_add_ties_go_to_even() {
    // 1 + 2^-8: the tie between 1.0 (0x3F80, even) and 1 + 2^-7 (0x3F81, odd).
    assert_eq!((BFloat16::ONE + bf(0.00390625)).to_bits(), 0x3F80);
    // 1 + 3*2^-8: the tie between 1 + 2^-7 (0x3F81, odd) and 1 + 2^-6 (0x3F82, even).
    assert_eq!((BFloat16::ONE + bf(0.01171875)).to_bits(), 0x3F82);
    // 1 + 5*2^-8: the tie between 1 + 2^-6 (0x3F82, even) and 1 + 3*2^-7 (0x3F83, odd).
    assert_eq!((BFloat16::ONE + bf(0.01953125)).to_bits(), 0x3F82);
}

#[test]
fn test_add_rounds_to_nearest_off_a_tie() {
    // 1 + 9*2^-9 lies between 1 + 8*2^-9 (0x3F82) and 1 + 12*2^-9 (0x3F83), nearer the first.
    assert_eq!((BFloat16::ONE + bf(0.017578125)).to_bits(), 0x3F82);
    // 1 + 11*2^-9 lies in the same gap, nearer the second.
    assert_eq!((BFloat16::ONE + bf(0.021484375)).to_bits(), 0x3F83);
}

#[test]
fn test_add_special_values() {
    assert_eq!(BFloat16::MAX + BFloat16::MAX, BFloat16::INFINITY);
    assert_eq!(BFloat16::MIN + BFloat16::MIN, BFloat16::NEG_INFINITY);
    assert!((BFloat16::INFINITY + BFloat16::NEG_INFINITY).is_nan());
    assert!((BFloat16::NAN + BFloat16::ONE).is_nan());
    assert_eq!((BFloat16::NEG_ZERO + BFloat16::NEG_ZERO).to_bits(), 0x8000);
    assert_eq!((BFloat16::ZERO + BFloat16::NEG_ZERO).to_bits(), 0x0000);
    assert_eq!(BFloat16::INFINITY + bf(-1.0), BFloat16::INFINITY);
}

// =============================================================================
// Subtraction
// =============================================================================

#[test]
fn test_sub() {
    assert_eq!(bf(3.0) - bf(5.0), bf(-2.0));
    assert_eq!((bf(1.0) - bf(1.0)).to_bits(), 0x0000);
    assert_eq!(bf(3.75) - bf(0.75), bf(3.0));
    assert!((BFloat16::INFINITY - BFloat16::INFINITY).is_nan());
    assert_eq!(BFloat16::MIN - BFloat16::MAX, BFloat16::NEG_INFINITY);
}

#[test]
fn test_sub_ties_go_to_even() {
    // 1 - 2^-9 is the tie between 1 - 2^-8 (0x3F7F, odd) and 1.0 (0x3F80, even).
    assert_eq!((BFloat16::ONE - bf(0.001953125)).to_bits(), 0x3F80);
}

// =============================================================================
// Multiplication
// =============================================================================

#[test]
fn test_mul_exact() {
    assert_eq!(bf(2.0) * bf(3.0), bf(6.0));
    assert_eq!(bf(-1.5) * bf(4.0), bf(-6.0));
    assert_eq!(bf(0.5) * bf(0.5), bf(0.25));
    // 3 * 85 = 255 = 1.1111111b * 2^7, exactly eight significant bits.
    assert_eq!((bf(3.0) * bf(85.0)).to_f32(), 255.0);
}

#[test]
fn test_mul_ties_go_to_even() {
    // 7 * 37 = 259 = 1.00000011b * 2^8 needs nine bits: the tie between 258 (0x4381, odd
    // significand) and 260 (0x4382, even).
    assert_eq!((bf(7.0) * bf(37.0)).to_bits(), 0x4382);
    assert_eq!((bf(7.0) * bf(37.0)).to_f32(), 260.0);
}

#[test]
fn test_mul_rounds_in_the_subnormal_range() {
    // 0.75 * 2^-133 is three quarters of the smallest subnormal, nearer to it than to zero.
    assert_eq!((bf(0.75) * BFloat16::from_bits(0x0001)).to_bits(), 0x0001);
    // 2^-126 * 2^-126 = 2^-252 is far below half the smallest subnormal.
    assert_eq!(
        (BFloat16::MIN_POSITIVE * BFloat16::MIN_POSITIVE).to_bits(),
        0x0000
    );
}

#[test]
fn test_mul_special_values() {
    assert!((BFloat16::ZERO * BFloat16::INFINITY).is_nan());
    assert_eq!(BFloat16::MAX * bf(2.0), BFloat16::INFINITY);
    assert_eq!((bf(-1.0) * BFloat16::ZERO).to_bits(), 0x8000);
    assert_eq!(BFloat16::NEG_INFINITY * bf(-2.0), BFloat16::INFINITY);
    assert!((BFloat16::NAN * BFloat16::ZERO).is_nan());
}

// =============================================================================
// Division
// =============================================================================

#[test]
fn test_div() {
    assert_eq!(bf(6.0) / bf(3.0), bf(2.0));
    // 1/3 = 0.3333...; in [0.25, 0.5) the step is 2^-9 and (1/3 - 0.25) / 2^-9 = 42.67, so
    // 0.25 + 43/512 = 0.333984375 with significand 43 = 0x2B on exponent 0x7D: 0x3EAB.
    assert_eq!((BFloat16::ONE / bf(3.0)).to_bits(), 0x3EAB);
    // 2/3 = 0.6666...; in [0.5, 1) the step is 2^-8 and (2/3 - 0.5) / 2^-8 = 42.67: 0x3F2B.
    assert_eq!((bf(2.0) / bf(3.0)).to_bits(), 0x3F2B);
    // 10/3 = 3.333...; in [2, 4) the step is 2^-6 and (10/3 - 2) / 2^-6 = 85.33: 0x4055.
    assert_eq!((bf(10.0) / bf(3.0)).to_bits(), 0x4055);
    assert_eq!(bf(-1.0) / bf(4.0), bf(-0.25));
}

#[test]
fn test_div_special_values() {
    assert_eq!(BFloat16::ONE / BFloat16::ZERO, BFloat16::INFINITY);
    assert_eq!(bf(-1.0) / BFloat16::ZERO, BFloat16::NEG_INFINITY);
    assert_eq!(BFloat16::ONE / BFloat16::NEG_ZERO, BFloat16::NEG_INFINITY);
    assert!((BFloat16::ZERO / BFloat16::ZERO).is_nan());
    assert!((BFloat16::INFINITY / BFloat16::INFINITY).is_nan());
    assert_eq!((BFloat16::ONE / BFloat16::INFINITY).to_bits(), 0x0000);
    assert_eq!((bf(-1.0) / BFloat16::INFINITY).to_bits(), 0x8000);
}

#[test]
fn test_div_into_the_subnormal_range() {
    // 2^-126 / 2 = 2^-127: significand 0x40 with a zero exponent field.
    assert_eq!((BFloat16::MIN_POSITIVE / bf(2.0)).to_bits(), 0x0040);
    // 2^-133 / 2 = 2^-134 is the tie between 0 (even) and 2^-133 (odd).
    assert_eq!((BFloat16::from_bits(0x0001) / bf(2.0)).to_bits(), 0x0000);
    // 3 * 2^-133 / 2 = 1.5 * 2^-133 is the tie between 2^-133 (odd) and 2^-132 (even).
    assert_eq!((BFloat16::from_bits(0x0003) / bf(2.0)).to_bits(), 0x0002);
}

// =============================================================================
// Remainder
// =============================================================================

#[test]
fn test_rem_has_the_sign_of_the_dividend() {
    // 5.5 = 101.1b; 5.5 - 2 * 2 = 1.5.
    assert_eq!(bf(5.5) % bf(2.0), bf(1.5));
    assert_eq!(bf(7.0) % bf(3.0), bf(1.0));
    assert_eq!(bf(-7.0) % bf(3.0), bf(-1.0));
    assert_eq!(bf(7.0) % bf(-3.0), bf(1.0));
    assert_eq!((bf(6.0) % bf(3.0)).to_bits(), 0x0000);
    assert_eq!((bf(-6.0) % bf(3.0)).to_bits(), 0x8000);
}

#[test]
fn test_rem_special_values() {
    assert!((BFloat16::ONE % BFloat16::ZERO).is_nan());
    assert!((BFloat16::INFINITY % BFloat16::ONE).is_nan());
    assert_eq!(BFloat16::ONE % BFloat16::INFINITY, BFloat16::ONE);
    assert!((BFloat16::NAN % BFloat16::ONE).is_nan());
}

// =============================================================================
// Assignment operators
// =============================================================================

#[test]
fn test_assign_ops() {
    let mut x = bf(10.0);
    x += bf(5.0);
    assert_eq!(x, bf(15.0));
    x -= bf(3.0);
    assert_eq!(x, bf(12.0));
    x *= bf(0.5);
    assert_eq!(x, bf(6.0));
    x /= bf(4.0);
    assert_eq!(x, bf(1.5));
    x %= bf(1.0);
    assert_eq!(x, bf(0.5));
}

// =============================================================================
// Reference operators
// =============================================================================

// The references are the subject here: the test exists to exercise the `&BFloat16` operator
// implementations that `NumRef` bounds need, so the lint that would remove them is silenced.
#[test]
#[allow(clippy::op_ref)]
fn test_reference_ops_match_value_ops() {
    let a = bf(7.0);
    let b = bf(37.0);
    assert_eq!(&a + &b, a + b);
    assert_eq!(a + &b, a + b);
    assert_eq!(&a + b, a + b);
    assert_eq!(&a - &b, a - b);
    assert_eq!(a - &b, a - b);
    assert_eq!(&a - b, a - b);
    assert_eq!(&a * &b, a * b);
    assert_eq!(a * &b, a * b);
    assert_eq!(&a * b, a * b);
    assert_eq!(&a / &b, a / b);
    assert_eq!(a / &b, a / b);
    assert_eq!(&a / b, a / b);
    assert_eq!(&b % &a, b % a);
    assert_eq!(b % &a, b % a);
    assert_eq!(&b % a, b % a);
}

// =============================================================================
// Every pair in a window is correctly rounded
// =============================================================================

#[test]
fn test_add_sub_mul_div_are_correctly_rounded_over_a_window() {
    // Sums, differences and products of 8-bit significands are exact in f64, so a single
    // rounding of the f64 result is the correctly rounded answer. Quotients are not exact in
    // f64, but f64 has 53 bits against bf16's 8, so the f64 quotient rounds to the same bf16 as
    // the exact quotient unless the exact quotient lies within 2^-45 of a bf16 midpoint, which
    // no quotient of two 8-bit significands does.
    let values: Vec<BFloat16> = (0x3F80..0x3F80 + 64)
        .chain(0x4000..0x4000 + 64)
        .chain(0xBF80..0xBF80 + 16)
        .map(BFloat16::from_bits)
        .collect();
    for &a in &values {
        for &b in &values {
            let (x, y) = (a.to_f64(), b.to_f64());
            assert_eq!(a + b, BFloat16::round_from_f64(x + y), "{a} + {b}");
            assert_eq!(a - b, BFloat16::round_from_f64(x - y), "{a} - {b}");
            assert_eq!(a * b, BFloat16::round_from_f64(x * y), "{a} * {b}");
            assert_eq!(a / b, BFloat16::round_from_f64(x / y), "{a} / {b}");
        }
    }
}
