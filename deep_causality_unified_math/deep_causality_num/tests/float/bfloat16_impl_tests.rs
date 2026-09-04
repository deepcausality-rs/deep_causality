/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `Float` trait implementation on `BFloat16`.
//!
//! Expected values are derived in the comment beside each assertion from a published constant
//! and the bf16 grid: in `[2^k, 2^(k+1))` the step is `2^(k-7)`, and the nearest grid point is
//! the answer. Where a family is checked, the reference is the exact `f64` result rounded once by
//! `round_from_f64`, which `tests/float_bfloat16/bfloat16_tests.rs` establishes independently.

use core::num::FpCategory;
use deep_causality_num::{BFloat16, Float};

fn bf(x: f32) -> BFloat16 {
    let v = BFloat16::from(x);
    assert_eq!(
        v.to_f32(),
        x,
        "test operand {x} is not exactly representable"
    );
    v
}

fn assert_float<T: Float>() {}

#[test]
fn test_float_bound() {
    assert_float::<BFloat16>();
}

// =============================================================================
// Special values and constants through the trait
// =============================================================================

#[test]
fn test_trait_constants_have_the_format_values() {
    assert_eq!(<BFloat16 as Float>::nan().to_bits(), 0x7FC0);
    assert_eq!(<BFloat16 as Float>::infinity().to_bits(), 0x7F80);
    assert_eq!(<BFloat16 as Float>::neg_infinity().to_bits(), 0xFF80);
    assert_eq!(<BFloat16 as Float>::neg_zero().to_bits(), 0x8000);
    assert_eq!(<BFloat16 as Float>::min_value().to_bits(), 0xFF7F);
    assert_eq!(<BFloat16 as Float>::min_positive_value().to_bits(), 0x0080);
    assert_eq!(<BFloat16 as Float>::epsilon().to_bits(), 0x3C00);
    assert_eq!(<BFloat16 as Float>::pi().to_bits(), 0x4049);
    assert_eq!(<BFloat16 as Float>::e().to_bits(), 0x402E);
    assert_eq!(<BFloat16 as Float>::max_value().to_bits(), 0x7F7F);
}

// =============================================================================
// Classification
// =============================================================================

#[test]
fn test_classification_predicates() {
    let one = BFloat16::ONE;
    assert!(Float::is_finite(one));
    assert!(Float::is_normal(one));
    assert!(!Float::is_nan(one));
    assert!(!Float::is_infinite(one));
    assert!(!Float::is_subnormal(one));

    let smallest_subnormal = BFloat16::from_bits(0x0001);
    let largest_subnormal = BFloat16::from_bits(0x007F);
    assert!(Float::is_subnormal(smallest_subnormal));
    assert!(Float::is_subnormal(largest_subnormal));
    assert!(!Float::is_normal(smallest_subnormal));
    assert!(Float::is_finite(smallest_subnormal));
    assert!(Float::is_normal(BFloat16::MIN_POSITIVE));
    assert!(!Float::is_subnormal(BFloat16::MIN_POSITIVE));

    assert!(!Float::is_normal(BFloat16::ZERO));
    assert!(!Float::is_subnormal(BFloat16::ZERO));
    assert!(Float::is_nan(BFloat16::NAN));
    assert!(!Float::is_normal(BFloat16::NAN));
    assert!(Float::is_infinite(BFloat16::INFINITY));
    assert!(Float::is_infinite(BFloat16::NEG_INFINITY));
    assert!(!Float::is_normal(BFloat16::INFINITY));
    assert!(!Float::is_finite(BFloat16::NEG_INFINITY));
}

#[test]
fn test_classify() {
    assert_eq!(Float::classify(BFloat16::ZERO), FpCategory::Zero);
    assert_eq!(Float::classify(BFloat16::NEG_ZERO), FpCategory::Zero);
    assert_eq!(
        Float::classify(BFloat16::from_bits(0x0001)),
        FpCategory::Subnormal
    );
    assert_eq!(
        Float::classify(BFloat16::from_bits(0x007F)),
        FpCategory::Subnormal
    );
    assert_eq!(Float::classify(BFloat16::MIN_POSITIVE), FpCategory::Normal);
    assert_eq!(Float::classify(BFloat16::MAX), FpCategory::Normal);
    assert_eq!(Float::classify(BFloat16::INFINITY), FpCategory::Infinite);
    assert_eq!(
        Float::classify(BFloat16::NEG_INFINITY),
        FpCategory::Infinite
    );
    assert_eq!(Float::classify(BFloat16::NAN), FpCategory::Nan);
}

#[test]
fn test_sign_predicates() {
    assert!(Float::is_sign_positive(BFloat16::ONE));
    assert!(Float::is_sign_positive(BFloat16::ZERO));
    assert!(Float::is_sign_positive(BFloat16::NAN));
    assert!(Float::is_sign_negative(BFloat16::NEG_ZERO));
    assert!(Float::is_sign_negative(bf(-1.0)));
    assert!(Float::is_sign_negative(-BFloat16::NAN));
    assert!(!Float::is_sign_negative(BFloat16::NAN));
}

// =============================================================================
// Rounding to integers
// =============================================================================

#[test]
fn test_floor_ceil_trunc_fract() {
    // 3.75 = 11.11b and 0.75 = 0.11b are exact.
    let x = bf(3.75);
    assert_eq!(Float::floor(x), bf(3.0));
    assert_eq!(Float::ceil(x), bf(4.0));
    assert_eq!(Float::trunc(x), bf(3.0));
    assert_eq!(Float::fract(x), bf(0.75));

    let y = bf(-3.75);
    assert_eq!(Float::floor(y), bf(-4.0));
    assert_eq!(Float::ceil(y), bf(-3.0));
    assert_eq!(Float::trunc(y), bf(-3.0));
    assert_eq!(Float::fract(y), bf(-0.75));

    // Toward zero from a negative fraction lands on negative zero.
    assert_eq!(Float::trunc(bf(-0.75)).to_bits(), 0x8000);
    assert_eq!(Float::ceil(bf(-0.5)).to_bits(), 0x8000);
    assert_eq!(Float::floor(bf(-0.5)), bf(-1.0));
    // Every value at or above 2^7 is an integer already.
    assert_eq!(Float::floor(bf(1000.0)), bf(1000.0));
    assert_eq!(Float::fract(bf(1000.0)).to_bits(), 0x0000);
}

#[test]
fn test_round_half_away_from_zero_both_sides() {
    // 2.25 is below the half, 2.5 is on it, 2.75 is above it.
    assert_eq!(Float::round(bf(2.25)), bf(2.0));
    assert_eq!(Float::round(bf(2.5)), bf(3.0));
    assert_eq!(Float::round(bf(2.75)), bf(3.0));
    assert_eq!(Float::round(bf(-2.5)), bf(-3.0));
    assert_eq!(Float::round(bf(-2.25)), bf(-2.0));
    assert_eq!(Float::round(bf(0.5)), bf(1.0));
    assert_eq!(Float::round(bf(-0.5)), bf(-1.0));
    assert_eq!(Float::round(bf(1.5)), bf(2.0));
}

#[test]
fn test_rounding_functions_keep_special_values() {
    for f in [Float::floor, Float::ceil, Float::round, Float::trunc] {
        assert!(f(BFloat16::NAN).is_nan());
        assert_eq!(f(BFloat16::INFINITY), BFloat16::INFINITY);
        assert_eq!(f(BFloat16::NEG_INFINITY), BFloat16::NEG_INFINITY);
        assert_eq!(f(BFloat16::NEG_ZERO).to_bits(), 0x8000);
    }
    assert!(Float::fract(BFloat16::NAN).is_nan());
    assert!(Float::fract(BFloat16::INFINITY).is_nan());
}

// =============================================================================
// Sign operations
// =============================================================================

#[test]
fn test_abs() {
    assert_eq!(Float::abs(bf(-2.0)), bf(2.0));
    assert_eq!(Float::abs(bf(2.0)), bf(2.0));
    assert_eq!(Float::abs(BFloat16::NEG_ZERO).to_bits(), 0x0000);
    assert_eq!(Float::abs(BFloat16::NEG_INFINITY), BFloat16::INFINITY);
    assert_eq!(Float::abs(BFloat16::MIN), BFloat16::MAX);
    let nan = Float::abs(-BFloat16::NAN);
    assert!(nan.is_nan());
    assert!(nan.is_sign_positive());
}

#[test]
fn test_signum() {
    assert_eq!(Float::signum(bf(3.5)), bf(1.0));
    assert_eq!(Float::signum(bf(-3.5)), bf(-1.0));
    assert_eq!(Float::signum(BFloat16::ZERO), bf(1.0));
    assert_eq!(Float::signum(BFloat16::NEG_ZERO), bf(-1.0));
    assert_eq!(Float::signum(BFloat16::INFINITY), bf(1.0));
    assert_eq!(Float::signum(BFloat16::NEG_INFINITY), bf(-1.0));
    assert!(Float::signum(BFloat16::NAN).is_nan());
}

#[test]
fn test_copysign() {
    assert_eq!(Float::copysign(bf(3.5), bf(-0.5)), bf(-3.5));
    assert_eq!(Float::copysign(bf(-3.5), bf(0.5)), bf(3.5));
    assert_eq!(Float::copysign(bf(3.5), bf(0.5)), bf(3.5));
    assert_eq!(Float::copysign(bf(-3.5), bf(-0.5)), bf(-3.5));
    assert_eq!(
        Float::copysign(bf(1.0), BFloat16::NEG_ZERO).to_bits(),
        0xBF80
    );
    let nan = Float::copysign(BFloat16::NAN, bf(-1.0));
    assert!(nan.is_nan());
    assert!(nan.is_sign_negative());
    assert_eq!(nan.to_bits(), 0xFFC0);
}

// =============================================================================
// Fused multiply-add
// =============================================================================

#[test]
fn test_mul_add_exact() {
    assert_eq!(Float::mul_add(bf(2.0), bf(3.0), bf(1.0)), bf(7.0));
    assert_eq!(Float::mul_add(bf(-1.5), bf(4.0), bf(6.0)), bf(0.0));
}

#[test]
fn test_mul_add_is_correctly_rounded() {
    // half-rs #141: `mul_add` computed in f32 and rounded again gives off-by-one results.
    // 7 * 37 = 259 is exactly the bf16 tie between 258 (odd) and 260 (even). Adding 2^-100 moves
    // the exact result off the tie, and a single rounding must follow that direction; any
    // intermediate rounding loses the 2^-100 and falls back to the tie's even neighbour.
    let tiny = BFloat16::round_from_f64(2f64.powi(-100));
    assert_eq!(tiny.to_f64(), 2f64.powi(-100));
    assert_eq!(
        Float::mul_add(bf(7.0), bf(37.0), -tiny).to_bits(),
        0x4381,
        "259 - 2^-100 rounds to 258"
    );
    assert_eq!(
        Float::mul_add(bf(7.0), bf(37.0), tiny).to_bits(),
        0x4382,
        "259 + 2^-100 rounds to 260"
    );
    assert_eq!(
        Float::mul_add(bf(7.0), bf(37.0), BFloat16::ZERO).to_bits(),
        0x4382,
        "the exact tie 259 rounds to even"
    );
}

#[test]
fn test_mul_add_leaves_an_already_odd_sum_alone() {
    // The kernel nudges the f64 sum toward the error only when the sum's significand is even; a
    // sum that is already odd carries the sticky information and must be left as it is. Reaching
    // that branch needs an addend far enough below the product to make the f64 addition inexact,
    // yet large enough to reach the sum's lowest bit.
    //
    // (17/16)^2 = 289/256 = 1 + 33 * 2^-8, and 33 is odd, so the product lands exactly on the
    // bf16 midpoint between 1.125 (stored significand 0010000, even) and 1.1328125 (0010001,
    // odd). The product of two 5-bit significands is exact in f64, so the midpoint is exact.
    let a = BFloat16::from_bits(0x3F88);
    assert_eq!(a.to_f64(), 17.0 / 16.0);
    assert_eq!(a.to_f64() * a.to_f64(), 289.0 / 256.0);

    // 3 * 2^-54 is three quarters of an f64 ulp of the midpoint. The f64 sum therefore rounds up
    // to midpoint + 1 ulp, whose lowest bit is set, and leaves a negative error term behind.
    // Were the odd sum nudged anyway, it would go back down to the midpoint exactly and the tie
    // rule would pick the even neighbour 1.125 — the wrong side of the exact result.
    let c = BFloat16::round_from_f64(3.0 * 2f64.powi(-54));
    assert_eq!(c.to_f64(), 3.0 * 2f64.powi(-54));

    assert_eq!(
        Float::mul_add(a, a, c).to_bits(),
        0x3F91,
        "289/256 + 3 * 2^-54 is above the midpoint and rounds up to 1.1328125"
    );
    // The two other sides of the same threshold: the exact tie, and the same step below it.
    assert_eq!(
        Float::mul_add(a, a, BFloat16::ZERO).to_bits(),
        0x3F90,
        "the exact tie 289/256 rounds to even"
    );
    assert_eq!(
        Float::mul_add(a, a, -c).to_bits(),
        0x3F90,
        "289/256 - 3 * 2^-54 is below the midpoint and rounds down to 1.125"
    );
}

#[test]
fn test_mul_add_agrees_with_a_single_rounding_over_a_window() {
    // For operands in [1, 4) the products are multiples of 2^-14 below 16 and the sums below 32,
    // so a * b + c is exact in f64 and one rounding of it is the correctly rounded answer.
    let values: Vec<BFloat16> = (0x3F80..0x3F80 + 128)
        .step_by(3)
        .chain((0x4000..0x4000 + 128).step_by(5))
        .map(BFloat16::from_bits)
        .collect();
    for &a in &values {
        for &b in &values {
            for &c in &values {
                let exact = a.to_f64() * b.to_f64() + c.to_f64();
                assert_eq!(
                    Float::mul_add(a, b, c),
                    BFloat16::round_from_f64(exact),
                    "{a} * {b} + {c}"
                );
                assert_eq!(
                    Float::mul_add(a, b, -c),
                    BFloat16::round_from_f64(a.to_f64() * b.to_f64() - c.to_f64()),
                    "{a} * {b} - {c}"
                );
            }
        }
    }
}

#[test]
fn test_mul_add_special_values() {
    assert!(Float::mul_add(BFloat16::INFINITY, BFloat16::ZERO, bf(1.0)).is_nan());
    assert!(Float::mul_add(bf(1.0), bf(1.0), BFloat16::NAN).is_nan());
    assert!(Float::mul_add(BFloat16::NAN, bf(1.0), bf(1.0)).is_nan());
    assert_eq!(
        Float::mul_add(BFloat16::INFINITY, bf(1.0), bf(1.0)),
        BFloat16::INFINITY
    );
    assert!(Float::mul_add(BFloat16::NEG_INFINITY, bf(1.0), BFloat16::INFINITY).is_nan());
    assert_eq!(
        Float::mul_add(BFloat16::MAX, bf(2.0), BFloat16::MAX),
        BFloat16::INFINITY
    );
    // An infinite sum has no error term to steer by; both signs must pass straight through.
    assert_eq!(
        Float::mul_add(BFloat16::NEG_INFINITY, bf(1.0), bf(1.0)),
        BFloat16::NEG_INFINITY
    );
    assert_eq!(
        Float::mul_add(BFloat16::MIN, bf(2.0), BFloat16::MIN),
        BFloat16::NEG_INFINITY
    );
    assert_eq!(
        Float::mul_add(bf(-1.0), BFloat16::ZERO, BFloat16::NEG_ZERO).to_bits(),
        0x8000
    );
}

// =============================================================================
// Powers and roots
// =============================================================================

#[test]
fn test_recip() {
    // 1/4 = 2^-2: biased exponent 125 = 0x7D: 0x3E80.
    assert_eq!(Float::recip(bf(4.0)).to_bits(), 0x3E80);
    assert_eq!(Float::recip(bf(3.0)).to_bits(), 0x3EAB);
    assert_eq!(Float::recip(BFloat16::ZERO), BFloat16::INFINITY);
    assert_eq!(Float::recip(BFloat16::NEG_ZERO), BFloat16::NEG_INFINITY);
    assert_eq!(Float::recip(BFloat16::INFINITY).to_bits(), 0x0000);
    assert!(Float::recip(BFloat16::NAN).is_nan());
}

#[test]
fn test_powi() {
    // 2^10 = 1024, biased exponent 137 = 0x89: 0x4480.
    assert_eq!(Float::powi(bf(2.0), 10).to_bits(), 0x4480);
    assert_eq!(Float::powi(bf(2.0), -1), bf(0.5));
    assert_eq!(Float::powi(bf(5.0), 0), bf(1.0));
    assert_eq!(Float::powi(BFloat16::ZERO, 0), bf(1.0));
    // (-2)^3 = -8: sign 1, biased exponent 130 = 0x82: 0xC100.
    assert_eq!(Float::powi(bf(-2.0), 3).to_bits(), 0xC100);
    // 1.5^2 = 2.25 = 10.01b: significand 0x10 on exponent 0x80: 0x4010.
    assert_eq!(Float::powi(bf(1.5), 2).to_bits(), 0x4010);
    assert_eq!(Float::powi(bf(10.0), 3), bf(1000.0));
    assert_eq!(Float::powi(BFloat16::ZERO, 2).to_bits(), 0x0000);
    assert_eq!(Float::powi(BFloat16::ZERO, -1), BFloat16::INFINITY);
}

#[test]
fn test_powf() {
    // 2^0.5 = 1.41421...; in [1, 2) the step is 2^-7 and (sqrt2 - 1) / 2^-7 = 53.02, so
    // 1 + 53/128 = 1.4140625, significand 0x35: 0x3FB5.
    assert_eq!(Float::powf(bf(2.0), bf(0.5)).to_bits(), 0x3FB5);
    assert_eq!(Float::powf(bf(4.0), bf(0.5)), bf(2.0));
    assert_eq!(Float::powf(bf(2.0), bf(3.0)), bf(8.0));
    assert_eq!(Float::powf(bf(5.0), BFloat16::ZERO), bf(1.0));
    assert_eq!(Float::powf(BFloat16::ZERO, bf(2.0)).to_bits(), 0x0000);
    // A negative base with a non-integer exponent has no real value.
    assert!(Float::powf(bf(-8.0), bf(171.0 / 512.0)).is_nan());
    assert_eq!(Float::powf(bf(2.0), BFloat16::INFINITY), BFloat16::INFINITY);
}

#[test]
fn test_sqrt() {
    assert_eq!(Float::sqrt(bf(4.0)), bf(2.0));
    assert_eq!(Float::sqrt(bf(2.0)).to_bits(), 0x3FB5);
    assert!(Float::sqrt(bf(-1.0)).is_nan());
    assert_eq!(Float::sqrt(BFloat16::ZERO).to_bits(), 0x0000);
    // IEEE 754: the square root of negative zero is negative zero.
    assert_eq!(Float::sqrt(BFloat16::NEG_ZERO).to_bits(), 0x8000);
    assert_eq!(Float::sqrt(BFloat16::INFINITY), BFloat16::INFINITY);
    // sqrt(2^-126) = 2^-63, biased exponent 64 = 0x40: 0x2000.
    assert_eq!(Float::sqrt(BFloat16::MIN_POSITIVE).to_bits(), 0x2000);
    // sqrt(2^-133) = 2^-66.5 = 1.41421 * 2^-67: significand 0x35 on biased exponent 60 = 0x3C:
    // 0x1E35.
    assert_eq!(Float::sqrt(BFloat16::from_bits(0x0001)).to_bits(), 0x1E35);
}

#[test]
fn test_cbrt_is_real_for_negative_input() {
    assert_eq!(Float::cbrt(bf(27.0)), bf(3.0));
    assert_eq!(Float::cbrt(bf(-8.0)), bf(-2.0));
    assert_eq!(Float::cbrt(bf(8.0)), bf(2.0));
    assert_eq!(Float::cbrt(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Float::cbrt(BFloat16::NEG_ZERO).to_bits(), 0x8000);
    assert_eq!(Float::cbrt(BFloat16::NEG_INFINITY), BFloat16::NEG_INFINITY);
    assert!(Float::cbrt(BFloat16::NAN).is_nan());
}

#[test]
fn test_hypot() {
    // 5 = 1.01b * 2^2: significand 0x20 on biased exponent 129 = 0x81: 0x40A0.
    assert_eq!(Float::hypot(bf(3.0), bf(4.0)).to_bits(), 0x40A0);
    assert_eq!(Float::hypot(bf(-3.0), bf(4.0)), bf(5.0));
    assert_eq!(Float::hypot(BFloat16::ZERO, bf(-2.0)), bf(2.0));
    assert_eq!(
        Float::hypot(BFloat16::INFINITY, BFloat16::NAN),
        BFloat16::INFINITY
    );
}

// =============================================================================
// Exponentials and logarithms
// =============================================================================

#[test]
fn test_exp_and_exp2() {
    assert_eq!(Float::exp(BFloat16::ZERO), bf(1.0));
    // e = 2.71828...; in [2, 4) the step is 2^-6 and (e - 2) / 2^-6 = 45.97: 2 + 46/64 = 0x402E.
    assert_eq!(Float::exp(bf(1.0)).to_bits(), 0x402E);
    // e^0.5 = 1.64872...; (0.64872) / 2^-7 = 83.04: 1 + 83/128 = 1.6484375 = 0x3FD3.
    assert_eq!(Float::exp(bf(0.5)).to_bits(), 0x3FD3);
    assert_eq!(Float::exp(BFloat16::NEG_INFINITY).to_bits(), 0x0000);
    assert_eq!(Float::exp(BFloat16::INFINITY), BFloat16::INFINITY);
    assert!(Float::exp(BFloat16::NAN).is_nan());
    assert_eq!(Float::exp2(bf(3.0)), bf(8.0));
    assert_eq!(Float::exp2(bf(-2.0)), bf(0.25));
    assert_eq!(Float::exp2(BFloat16::ZERO), bf(1.0));
}

#[test]
fn test_ln_and_logs() {
    assert_eq!(Float::ln(bf(1.0)).to_bits(), 0x0000);
    // ln(2.71875) = 1.000172...; in [1, 2) the step is 2^-7 = 0.0078, so the result is 1.0.
    assert_eq!(Float::ln(BFloat16::E), bf(1.0));
    assert_eq!(Float::ln(BFloat16::ZERO), BFloat16::NEG_INFINITY);
    assert!(Float::ln(bf(-1.0)).is_nan());
    assert_eq!(Float::ln(BFloat16::INFINITY), BFloat16::INFINITY);
    assert_eq!(Float::log(bf(8.0), bf(2.0)), bf(3.0));
    assert_eq!(Float::log(bf(100.0), bf(10.0)), bf(2.0));
    assert_eq!(Float::log2(bf(8.0)), bf(3.0));
    assert_eq!(Float::log2(bf(0.25)), bf(-2.0));
    assert_eq!(Float::log2(bf(1.0)).to_bits(), 0x0000);
    assert_eq!(Float::log10(bf(1000.0)), bf(3.0));
    assert_eq!(Float::log10(bf(100.0)), bf(2.0));
    assert_eq!(Float::log10(bf(1.0)).to_bits(), 0x0000);
}

#[test]
fn test_exp_m1_and_ln_1p() {
    assert_eq!(Float::exp_m1(BFloat16::ZERO).to_bits(), 0x0000);
    // e^0.5 - 1 = 0.64872...; in [0.5, 1) the step is 2^-8 and (0.64872 - 0.5) / 2^-8 = 38.07:
    // 0.5 + 38/256 = 0.6484375, significand 0x26 on exponent 0x7E: 0x3F26.
    assert_eq!(Float::exp_m1(bf(0.5)).to_bits(), 0x3F26);
    assert_eq!(Float::exp_m1(BFloat16::NEG_INFINITY), bf(-1.0));
    assert_eq!(Float::ln_1p(BFloat16::ZERO).to_bits(), 0x0000);
    // ln(1.5) = 0.405465...; in [0.25, 0.5) the step is 2^-9 and (0.405465 - 0.25) / 2^-9 =
    // 79.6: 0.25 + 80/512 = 0.40625, significand 0x50 on exponent 0x7D: 0x3ED0.
    assert_eq!(Float::ln_1p(bf(0.5)).to_bits(), 0x3ED0);
    assert_eq!(Float::ln_1p(bf(-1.0)), BFloat16::NEG_INFINITY);
    assert!(Float::ln_1p(bf(-2.0)).is_nan());
}

#[test]
fn test_degrees_and_radians() {
    // 3.140625 * 180 / pi = 179.945; in [128, 256) the step is 1, so 180 = 1.0110100b * 2^7:
    // significand 0x34 on biased exponent 134 = 0x86: 0x4334.
    assert_eq!(Float::to_degrees(BFloat16::PI).to_bits(), 0x4334);
    // 180 * pi / 180 = pi, which rounds to the PI constant 0x4049.
    assert_eq!(Float::to_radians(bf(180.0)).to_bits(), 0x4049);
    assert_eq!(Float::to_degrees(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Float::to_radians(BFloat16::NEG_ZERO).to_bits(), 0x8000);
}

// =============================================================================
// Trigonometry
// =============================================================================

#[test]
fn test_trigonometric_functions() {
    assert_eq!(Float::sin(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Float::cos(BFloat16::ZERO), bf(1.0));
    assert_eq!(Float::tan(BFloat16::ZERO).to_bits(), 0x0000);
    // 0x3FC9 = 1.5703125 is bf16(pi/2); sin of it is 0.99999996, which rounds to 1.
    let half_pi = BFloat16::from_bits(0x3FC9);
    assert_eq!(Float::sin(half_pi), bf(1.0));
    // cos(3.140625) = -0.99999952; in [0.5, 1) the step is 2^-8 = 0.0039, so it rounds to -1.
    assert_eq!(Float::cos(BFloat16::PI), bf(-1.0));
    // 0x3F49 = 0.78515625 is bf16(pi/4); tan of it is 0.99951, which rounds to 1.
    assert_eq!(Float::tan(BFloat16::from_bits(0x3F49)), bf(1.0));
    assert!(Float::sin(BFloat16::INFINITY).is_nan());
    assert!(Float::cos(BFloat16::NAN).is_nan());
    let (s, c) = Float::sin_cos(BFloat16::ZERO);
    assert_eq!(s.to_bits(), 0x0000);
    assert_eq!(c, bf(1.0));
    for x in [bf(0.5), bf(-1.5), bf(3.0), BFloat16::PI] {
        assert_eq!(Float::sin_cos(x), (Float::sin(x), Float::cos(x)), "{x}");
    }
}

#[test]
fn test_inverse_trigonometric_at_the_domain_boundary() {
    // asin(1) = pi/2 = 1.5707963; (0.5707963) / 2^-7 = 73.06: 1 + 73/128 = 1.5703125 = 0x3FC9.
    assert_eq!(Float::asin(bf(1.0)).to_bits(), 0x3FC9);
    assert_eq!(Float::asin(bf(-1.0)).to_bits(), 0xBFC9);
    assert_eq!(Float::asin(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Float::acos(bf(1.0)).to_bits(), 0x0000);
    // acos(-1) = pi, rounding to the PI constant.
    assert_eq!(Float::acos(bf(-1.0)).to_bits(), 0x4049);
    // acos(0.5) = pi/3 = 1.0471976; (0.0471976) / 2^-7 = 6.04: 1 + 6/128 = 0x3F86.
    assert_eq!(Float::acos(bf(0.5)).to_bits(), 0x3F86);
    // Outside [-1, 1] there is no real value.
    assert!(Float::asin(bf(1.5)).is_nan());
    assert!(Float::acos(bf(-1.5)).is_nan());
    // atan(1) = pi/4 = 0.7853982; in [0.5, 1) the step is 2^-8 and (0.7853982 - 0.5) / 2^-8 =
    // 73.06: 0.5 + 73/256 = 0.78515625 = 0x3F49.
    assert_eq!(Float::atan(bf(1.0)).to_bits(), 0x3F49);
    assert_eq!(Float::atan(BFloat16::INFINITY).to_bits(), 0x3FC9);
    assert_eq!(Float::atan2(bf(1.0), bf(1.0)).to_bits(), 0x3F49);
    assert_eq!(Float::atan2(BFloat16::ZERO, bf(-1.0)).to_bits(), 0x4049);
    assert_eq!(Float::atan2(bf(-1.0), BFloat16::ZERO).to_bits(), 0xBFC9);
    assert_eq!(
        Float::atan2(BFloat16::ZERO, BFloat16::ZERO).to_bits(),
        0x0000
    );
}

// =============================================================================
// Hyperbolic functions
// =============================================================================

#[test]
fn test_hyperbolic() {
    assert_eq!(Float::sinh(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Float::cosh(BFloat16::ZERO), bf(1.0));
    assert_eq!(Float::tanh(BFloat16::ZERO).to_bits(), 0x0000);
    // sinh 1 = 1.1752012; (0.1752012) / 2^-7 = 22.43: 1 + 22/128 = 1.171875 = 0x3F96.
    assert_eq!(Float::sinh(bf(1.0)).to_bits(), 0x3F96);
    // cosh 1 = 1.5430806; (0.5430806) / 2^-7 = 69.51: 1 + 70/128 = 1.546875 = 0x3FC6.
    assert_eq!(Float::cosh(bf(1.0)).to_bits(), 0x3FC6);
    // tanh 1 = 0.7615942; (0.7615942 - 0.5) / 2^-8 = 66.97: 0.5 + 67/256 = 0.76171875 = 0x3F43.
    assert_eq!(Float::tanh(bf(1.0)).to_bits(), 0x3F43);
    assert_eq!(Float::tanh(BFloat16::INFINITY), bf(1.0));
    assert_eq!(Float::tanh(BFloat16::NEG_INFINITY), bf(-1.0));
    assert_eq!(Float::sinh(bf(-1.0)).to_bits(), 0xBF96);
}

#[test]
fn test_inverse_hyperbolic() {
    assert_eq!(Float::asinh(BFloat16::ZERO).to_bits(), 0x0000);
    // asinh 1 = 0.8813736; (0.8813736 - 0.5) / 2^-8 = 97.63: 0.5 + 98/256 = 0.8828125 = 0x3F62.
    assert_eq!(Float::asinh(bf(1.0)).to_bits(), 0x3F62);
    assert_eq!(Float::acosh(bf(1.0)).to_bits(), 0x0000);
    // acosh 2 = 1.3169579; (0.3169579) / 2^-7 = 40.57: 1 + 41/128 = 1.3203125 = 0x3FA9.
    assert_eq!(Float::acosh(bf(2.0)).to_bits(), 0x3FA9);
    // Below 1 there is no real value.
    assert!(Float::acosh(bf(0.5)).is_nan());
    assert_eq!(Float::atanh(BFloat16::ZERO).to_bits(), 0x0000);
    // atanh 0.5 = 0.5493061; (0.0493061) / 2^-8 = 12.62: 0.5 + 13/256 = 0.55078125 = 0x3F0D.
    assert_eq!(Float::atanh(bf(0.5)).to_bits(), 0x3F0D);
    assert_eq!(Float::atanh(bf(1.0)), BFloat16::INFINITY);
    assert_eq!(Float::atanh(bf(-1.0)), BFloat16::NEG_INFINITY);
    assert!(Float::atanh(bf(2.0)).is_nan());
}

// =============================================================================
// max, min, clamp
// =============================================================================

#[test]
fn test_max_and_min() {
    assert_eq!(Float::max(bf(1.0), bf(2.0)), bf(2.0));
    assert_eq!(Float::max(bf(2.0), bf(1.0)), bf(2.0));
    assert_eq!(Float::min(bf(1.0), bf(2.0)), bf(1.0));
    assert_eq!(Float::min(bf(2.0), bf(1.0)), bf(1.0));
    assert_eq!(Float::max(bf(-1.0), BFloat16::NEG_INFINITY), bf(-1.0));
    assert_eq!(
        Float::min(bf(-1.0), BFloat16::NEG_INFINITY),
        BFloat16::NEG_INFINITY
    );
    assert_eq!(
        Float::max(BFloat16::MAX, BFloat16::INFINITY),
        BFloat16::INFINITY
    );
}

#[test]
fn test_max_and_min_return_the_other_operand_for_nan() {
    // half-rs #126 (fixed in 2.7.0): `max` and `min` returned NaN whenever `self` was NaN. IEEE
    // 754-2008 maxNum and minNum return the number when exactly one operand is NaN.
    assert_eq!(Float::max(BFloat16::NAN, bf(1.0)), bf(1.0));
    assert_eq!(Float::max(bf(1.0), BFloat16::NAN), bf(1.0));
    assert_eq!(Float::min(BFloat16::NAN, bf(1.0)), bf(1.0));
    assert_eq!(Float::min(bf(1.0), BFloat16::NAN), bf(1.0));
    assert!(Float::max(BFloat16::NAN, BFloat16::NAN).is_nan());
    assert!(Float::min(BFloat16::NAN, BFloat16::NAN).is_nan());
}

#[test]
fn test_clamp() {
    let (lo, hi) = (BFloat16::ZERO, bf(10.0));
    assert_eq!(Float::clamp(bf(5.0), lo, hi), bf(5.0));
    assert_eq!(Float::clamp(bf(-5.0), lo, hi), lo);
    assert_eq!(Float::clamp(bf(15.0), lo, hi), hi);
    assert_eq!(Float::clamp(lo, lo, hi), lo);
    assert_eq!(Float::clamp(hi, lo, hi), hi);
    // One step inside each bound stays where it is.
    assert_eq!(Float::clamp(bf(0.0078125), lo, hi).to_bits(), 0x3C00);
    assert_eq!(Float::clamp(bf(9.9375), lo, hi), bf(9.9375));
    assert_eq!(Float::clamp(BFloat16::NEG_INFINITY, lo, hi), lo);
    assert_eq!(Float::clamp(BFloat16::INFINITY, lo, hi), hi);
    assert!(Float::clamp(BFloat16::NAN, lo, hi).is_nan());
}

// =============================================================================
// integer_decode
// =============================================================================

#[test]
fn test_integer_decode() {
    // 2.0 = 128 * 2^-6 with the implicit bit set: (128, -6, 1).
    assert_eq!(Float::integer_decode(bf(2.0)), (128, -6, 1));
    // 1.0 = 128 * 2^-7.
    assert_eq!(Float::integer_decode(bf(1.0)), (128, -7, 1));
    // -0.5 = -(128 * 2^-8).
    assert_eq!(Float::integer_decode(bf(-0.5)), (128, -8, -1));
    // The smallest subnormal follows the f32 convention: the significand is shifted left by one
    // and the exponent is the fixed minimum, 2 * 2^-134 = 2^-133.
    assert_eq!(
        Float::integer_decode(BFloat16::from_bits(0x0001)),
        (2, -134, 1)
    );
    // 3.75 = 1.111b * 2^1 = 240 * 2^-6.
    assert_eq!(Float::integer_decode(bf(3.75)), (240, -6, 1));
}

#[test]
fn test_integer_decode_reconstructs_every_finite_value() {
    for bits in 0..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        if !x.is_finite() {
            continue;
        }
        let (mantissa, exponent, sign) = Float::integer_decode(x);
        let rebuilt = sign as f64 * mantissa as f64 * 2f64.powi(exponent as i32);
        assert_eq!(rebuilt, x.to_f64(), "bits {bits:#06x}");
    }
}

// =============================================================================
// Overflow and underflow reach
// =============================================================================

#[test]
fn test_intermediates_beyond_the_type_do_not_break_representable_results() {
    // MAX * 2 is not representable but MAX * 2 - MAX = MAX is; a fused multiply-add rounds once,
    // at the end, and must return MAX rather than the infinity a two-step evaluation produces.
    assert_eq!(
        Float::mul_add(BFloat16::MAX, bf(2.0), BFloat16::MIN),
        BFloat16::MAX
    );
    // hypot(MAX, 0) = MAX without squaring MAX first.
    assert_eq!(Float::hypot(BFloat16::MAX, BFloat16::ZERO), BFloat16::MAX);
    // sqrt(MAX) = sqrt(255/64) * 2^63 = 1.99609 * 2^63; (0.99609) / 2^-7 = 127.4995, so the
    // significand is 127 on biased exponent 190 = 0xBE: 0x5F7F.
    assert_eq!(Float::sqrt(BFloat16::MAX).to_bits(), 0x5F7F);
    // Overflow and underflow at the type's own extremes.
    assert_eq!(Float::powi(bf(2.0), 200), BFloat16::INFINITY);
    assert_eq!(Float::powi(bf(2.0), -200).to_bits(), 0x0000);
    assert_eq!(Float::powi(bf(2.0), -133).to_bits(), 0x0001);
    assert_eq!(Float::powi(bf(2.0), 127).to_bits(), 0x7F00);
    assert_eq!(Float::exp(bf(100.0)), BFloat16::INFINITY);
    assert_eq!(Float::exp(bf(-100.0)).to_bits(), 0x0000);
    assert_eq!(
        Float::hypot(BFloat16::MAX, BFloat16::MAX),
        BFloat16::INFINITY
    );
    assert_eq!(Float::exp2(bf(128.0)), BFloat16::INFINITY);
}
