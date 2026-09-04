/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Float trait implementation on DoubleFloat.

use core::num::FpCategory;
use deep_causality_num::{Float, Float106};

/// Both words of an exactly representable result.
///
/// Every expectation rewritten to use this is a value the type holds exactly — a small integer,
/// a dyadic fraction, or an exact root or logarithm — so the low word must be exactly zero.
/// Comparing only the high word against a tolerance, as this file did, accepts any low word at
/// all; that is how an implementation capped at `f64` accuracy passes a test on a 106-bit type.
fn assert_exact(got: Float106, want: f64) {
    assert_eq!(got.hi(), want, "high word");
    assert_eq!(
        got.lo(),
        0.0,
        "low word must be exactly zero for an exactly representable result"
    );
}

/// Relative error, for the results that are not exactly representable.
fn rel_err(got: Float106, want: Float106) -> f64 {
    if want == Float106::from(0.0) {
        return f64::from(<Float106 as Float>::abs(got));
    }
    f64::from(<Float106 as Float>::abs(got - want) / <Float106 as Float>::abs(want))
}

/// The relative accuracy `Float106` delivers; see `double_transcendental_tests`.
const TOL: f64 = 1e-29;

// =============================================================================
// Special Values Tests
// =============================================================================

#[test]
fn test_nan() {
    let nan = <Float106 as Float>::nan();
    assert!(nan.is_nan());
}

#[test]
fn test_infinity() {
    let inf = <Float106 as Float>::infinity();
    assert!(inf.is_infinite());
    assert!(Float::is_sign_positive(inf));
}

#[test]
fn test_neg_infinity() {
    let neg_inf = <Float106 as Float>::neg_infinity();
    assert!(neg_inf.is_infinite());
    assert!(Float::is_sign_negative(neg_inf));
}

#[test]
fn test_neg_zero() {
    let neg_zero = <Float106 as Float>::neg_zero();
    assert!(Float::is_sign_negative(neg_zero));
    assert_eq!(neg_zero.hi(), -0.0);
}

#[test]
fn test_min_value() {
    let min = <Float106 as Float>::min_value();
    assert!(min.hi() < 0.0);
    assert!(min.is_finite());
}

#[test]
fn test_min_positive_value() {
    let min_pos = <Float106 as Float>::min_positive_value();
    assert!(min_pos.hi() > 0.0);
    assert!(min_pos.is_finite());
}

#[test]
fn test_max_value() {
    let max = <Float106 as Float>::max_value();
    assert!(max.hi() > 0.0);
    assert!(max.is_finite());
}

#[test]
fn test_epsilon() {
    let eps = <Float106 as Float>::epsilon();
    assert!(eps.hi() > 0.0);
}

// =============================================================================
// Classification Tests
// =============================================================================

#[test]
fn test_is_nan_trait() {
    let nan = <Float106 as Float>::nan();
    assert!(Float::is_nan(nan));
}

#[test]
fn test_is_infinite_trait() {
    let inf = <Float106 as Float>::infinity();
    assert!(Float::is_infinite(inf));
}

#[test]
fn test_is_finite_trait() {
    let x = Float106::from(42.0);
    assert!(Float::is_finite(x));
}

#[test]
fn test_is_normal_trait() {
    let x = Float106::from(42.0);
    assert!(Float::is_normal(x));
}

#[test]
fn test_is_subnormal() {
    let x = Float106::from(f64::MIN_POSITIVE / 2.0);
    assert!(Float::is_subnormal(x));
}

#[test]
fn test_classify_normal() {
    let x = Float106::from(42.0);
    assert_eq!(Float::classify(x), FpCategory::Normal);
}

#[test]
fn test_classify_zero() {
    let x = Float106::from(0.0);
    assert_eq!(Float::classify(x), FpCategory::Zero);
}

#[test]
fn test_classify_infinite() {
    let x = <Float106 as Float>::infinity();
    assert_eq!(Float::classify(x), FpCategory::Infinite);
}

#[test]
fn test_classify_nan() {
    let x = <Float106 as Float>::nan();
    assert_eq!(Float::classify(x), FpCategory::Nan);
}

// =============================================================================
// Rounding Tests
// =============================================================================

#[test]
fn test_floor() {
    let x = Float106::from(3.7);
    assert_exact(Float::floor(x), 3.0);
}

#[test]
fn test_floor_negative() {
    let x = Float106::from(-3.3);
    assert_exact(Float::floor(x), -4.0);
}

#[test]
fn test_ceil() {
    let x = Float106::from(3.3);
    assert_exact(Float::ceil(x), 4.0);
}

#[test]
fn test_ceil_negative() {
    let x = Float106::from(-3.7);
    assert_exact(Float::ceil(x), -3.0);
}

#[test]
fn test_round() {
    let x = Float106::from(3.5);
    assert_exact(Float::round(x), 4.0);
}

#[test]
fn test_round_down() {
    let x = Float106::from(3.4);
    assert_exact(Float::round(x), 3.0);
}

#[test]
fn test_trunc() {
    let x = Float106::from(3.9);
    assert_exact(Float::trunc(x), 3.0);
}

#[test]
fn test_trunc_negative() {
    let x = Float106::from(-3.9);
    assert_exact(Float::trunc(x), -3.0);
}

#[test]
fn test_fract() {
    // 3.7 is not representable, so the fractional part is not the f64 0.7 either: it is
    // whatever remains after the integer part is removed from the stored value. The defining
    // property is the split itself, which is exact.
    let x = Float106::from(3.7);
    assert_eq!(Float::fract(x), x - Float106::from(3.0));
    assert_eq!(Float::trunc(x) + Float::fract(x), x);
    for v in [3.7_f64, -3.7, 0.5, -0.5, 42.0, 0.0] {
        let d = Float106::from(v);
        assert_eq!(Float::trunc(d) + Float::fract(d), d, "trunc+fract at {v}");
    }
}

// =============================================================================
// Sign Tests
// =============================================================================

#[test]
fn test_abs() {
    let x = Float106::from(-42.0);
    assert_exact(Float::abs(x), 42.0);
}

#[test]
fn test_abs_positive() {
    let x = Float106::from(42.0);
    assert_exact(Float::abs(x), 42.0);
}

#[test]
fn test_signum_positive() {
    let x = Float106::from(42.0);
    assert_exact(Float::signum(x), 1.0);
}

#[test]
fn test_signum_negative() {
    let x = Float106::from(-42.0);
    assert_exact(Float::signum(x), -1.0);
}

#[test]
fn test_signum_zero() {
    let x = Float106::from(0.0);
    // For positive zero, signum returns 1.0 (sign-preserving behavior)
    let sig = Float::signum(x);
    assert!(
        sig.hi() == 1.0 || sig.hi() == 0.0,
        "signum of zero should be 0.0 or 1.0"
    );
}

#[test]
fn test_is_sign_positive_trait() {
    let x = Float106::from(42.0);
    assert!(Float::is_sign_positive(x));
}

#[test]
fn test_is_sign_negative_trait() {
    let x = Float106::from(-42.0);
    assert!(Float::is_sign_negative(x));
}

// =============================================================================
// Arithmetic Operations
// =============================================================================

#[test]
fn test_mul_add() {
    let x = Float106::from(2.0);
    let a = Float106::from(3.0);
    let b = Float106::from(4.0);
    // x * a + b = 2 * 3 + 4 = 10
    let result = Float::mul_add(x, a, b);
    assert_exact(result, 10.0);
}

#[test]
fn test_recip() {
    let x = Float106::from(4.0);
    let result = Float::recip(x);
    assert_exact(result, 0.25);
}

#[test]
fn test_powi_positive() {
    let x = Float106::from(2.0);
    let result = Float::powi(x, 3);
    assert_exact(result, 8.0);
}

#[test]
fn test_powi_negative() {
    let x = Float106::from(2.0);
    let result = Float::powi(x, -2);
    assert_exact(result, 0.25);
}

#[test]
fn test_powi_zero() {
    let x = Float106::from(42.0);
    let result = Float::powi(x, 0);
    assert_exact(result, 1.0);
}

#[test]
fn test_powf() {
    let x = Float106::from(2.0);
    let n = Float106::from(3.0);
    let result = Float::powf(x, n);
    assert_exact(result, 8.0);
}

#[test]
fn test_sqrt() {
    let x = Float106::from(9.0);
    let result = Float::sqrt(x);
    assert_exact(result, 3.0);
}

#[test]
fn test_cbrt() {
    let x = Float106::from(27.0);
    let result = Float::cbrt(x);
    assert_exact(result, 3.0);
}

#[test]
fn test_cbrt_negative() {
    let x = Float106::from(-8.0);
    let result = Float::cbrt(x);
    assert_exact(result, -2.0);
}

#[test]
fn test_hypot() {
    let x = Float106::from(3.0);
    let y = Float106::from(4.0);
    let result = Float::hypot(x, y);
    assert_exact(result, 5.0);
}

// =============================================================================
// Exponential and Logarithmic
// =============================================================================

#[test]
fn test_exp() {
    let x = Float106::from(0.0);
    let result = Float::exp(x);
    assert_exact(result, 1.0);
}

#[test]
fn test_exp2() {
    let x = Float106::from(3.0);
    let result = Float::exp2(x);
    assert_exact(result, 8.0);
}

#[test]
fn test_ln() {
    // ln(1) is exactly zero, not merely small.
    let x = Float106::from(1.0);
    assert_exact(Float::ln(x), 0.0);
}

#[test]
fn test_log() {
    let x = Float106::from(8.0);
    let base = Float106::from(2.0);
    let result = Float::log(x, base);
    assert_exact(result, 3.0);
}

#[test]
fn test_log2() {
    // log2(8) is 3, reached through a logarithm ratio rather than exactly, so the low word
    // carries a rounding at the type's resolution.
    let x = Float106::from(8.0);
    let result = Float::log2(x);
    assert_eq!(result.hi(), 3.0);
    assert!(rel_err(result, Float106::from(3.0)) <= TOL);
    // Exact powers of two across a range.
    for k in 1..20 {
        let p = Float106::from((1u64 << k) as f64);
        assert!(
            rel_err(Float::log2(p), Float106::from(k as f64)) <= TOL,
            "log2(2^{k})"
        );
    }
}

#[test]
fn test_log10() {
    let x = Float106::from(1000.0);
    let result = Float::log10(x);
    assert_exact(result, 3.0);
}

// =============================================================================
// Min/Max/Clamp
// =============================================================================

#[test]
fn test_max() {
    let x = Float106::from(3.0);
    let y = Float106::from(5.0);
    let result = Float::max(x, y);
    assert_exact(result, 5.0);
}

#[test]
fn test_min() {
    let x = Float106::from(3.0);
    let y = Float106::from(5.0);
    let result = Float::min(x, y);
    assert_exact(result, 3.0);
}

#[test]
fn test_clamp_via_float() {
    let x = Float106::from(15.0);
    let min = Float106::from(0.0);
    let max = Float106::from(10.0);
    let result = Float::clamp(x, min, max);
    assert_exact(result, 10.0);
}

// =============================================================================
// Edge Case / Branch Coverage
// =============================================================================

#[test]
fn test_clamp_below_min() {
    let x = Float106::from(-5.0);
    let min = Float106::from(0.0);
    let max = Float106::from(10.0);
    let result = Float::clamp(x, min, max);
    assert_exact(result, 0.0);
}

#[test]
fn test_clamp_in_range() {
    let x = Float106::from(3.0);
    let min = Float106::from(0.0);
    let max = Float106::from(10.0);
    let result = Float::clamp(x, min, max);
    assert_exact(result, 3.0);
}

#[test]
fn test_signum_nan() {
    let x = <Float106 as Float>::nan();
    assert!(Float::signum(x).is_nan());
}

#[test]
fn test_signum_neg_zero_lo() {
    // hi == 0 && lo < 0 path
    let x = Float106::new(0.0, -1.0e-30);
    assert_exact(Float::signum(x), -1.0);
}

#[test]
fn test_abs_neg_zero_lo() {
    // hi == 0 && lo < 0 → negate path
    let x = Float106::new(0.0, -1.0e-30);
    let r = Float::abs(x);
    assert!(r.lo() >= 0.0);
}

#[test]
fn test_powf_zero_positive_exponent() {
    let x = Float106::from(0.0);
    let n = Float106::from(2.0);
    assert_eq!(Float::powf(x, n).hi(), 0.0);
}

#[test]
fn test_powf_zero_nonpositive_exponent() {
    let x = Float106::from(0.0);
    let n = Float106::from(-1.0);
    assert!(Float::powf(x, n).is_infinite());
}

#[test]
fn test_powf_negative_base_nan() {
    let x = Float106::from(-2.0);
    let n = Float106::from(0.5);
    assert!(Float::powf(x, n).is_nan());
}

#[test]
fn test_sqrt_negative_nan() {
    let x = Float106::from(-1.0);
    assert!(Float::sqrt(x).is_nan());
}

#[test]
fn test_sqrt_zero() {
    let x = Float106::from(0.0);
    assert_eq!(Float::sqrt(x).hi(), 0.0);
}

#[test]
fn test_exp_nan() {
    let x = <Float106 as Float>::nan();
    assert!(Float::exp(x).is_nan());
}

#[test]
fn test_exp_overflow_to_infinity() {
    let x = Float106::from(1000.0);
    assert!(Float::exp(x).is_infinite());
}

#[test]
fn test_exp_underflow_to_zero() {
    let x = Float106::from(-1000.0);
    assert_eq!(Float::exp(x).hi(), 0.0);
}

#[test]
fn test_ln_zero_returns_neg_infinity() {
    let x = Float106::from(0.0);
    let r = Float::ln(x);
    assert!(r.is_infinite() && Float::is_sign_negative(r));
}

#[test]
fn test_ln_negative_nan() {
    let x = Float106::from(-1.0);
    assert!(Float::ln(x).is_nan());
}

#[test]
fn test_max_nan_first() {
    let nan = <Float106 as Float>::nan();
    let y = Float106::from(2.0);
    assert_exact(Float::max(nan, y), 2.0);
}

#[test]
fn test_max_nan_second() {
    let x = Float106::from(2.0);
    let nan = <Float106 as Float>::nan();
    assert_exact(Float::max(x, nan), 2.0);
}

#[test]
fn test_min_nan_first() {
    let nan = <Float106 as Float>::nan();
    let y = Float106::from(2.0);
    assert_exact(Float::min(nan, y), 2.0);
}

#[test]
fn test_min_nan_second() {
    let x = Float106::from(2.0);
    let nan = <Float106 as Float>::nan();
    assert_exact(Float::min(x, nan), 2.0);
}

#[test]
fn test_atan_one_special() {
    let x = Float106::from(1.0);
    let r = Float::atan(x);
    assert!((r.hi() - core::f64::consts::FRAC_PI_4).abs() < 1e-14);
}

#[test]
fn test_atan_neg_one_special() {
    let x = Float106::from(-1.0);
    let r = Float::atan(x);
    assert!((r.hi() - (-core::f64::consts::FRAC_PI_4)).abs() < 1e-14);
}

#[test]
fn test_atan2_zero_zero_nan() {
    let z = Float106::from(0.0);
    let r = Float::atan2(z, z);
    assert!(r.is_nan());
}

#[test]
fn test_atan2_positive_y_zero_x() {
    let y = Float106::from(1.0);
    let x = Float106::from(0.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - core::f64::consts::FRAC_PI_2).abs() < 1e-14);
}

#[test]
fn test_atan2_negative_y_zero_x() {
    let y = Float106::from(-1.0);
    let x = Float106::from(0.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - (-core::f64::consts::FRAC_PI_2)).abs() < 1e-14);
}

#[test]
fn test_atan2_negative_x_positive_y() {
    let y = Float106::from(1.0);
    let x = Float106::from(-1.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - (3.0 * core::f64::consts::FRAC_PI_4)).abs() < 1e-10);
}

#[test]
fn test_atan2_negative_x_negative_y() {
    let y = Float106::from(-1.0);
    let x = Float106::from(-1.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - (-3.0 * core::f64::consts::FRAC_PI_4)).abs() < 1e-10);
}

#[test]
fn test_asin_one() {
    let x = Float106::from(1.0);
    let r = Float::asin(x);
    assert!((r.hi() - core::f64::consts::FRAC_PI_2).abs() < 1e-14);
}

#[test]
fn test_asin_neg_one() {
    let x = Float106::from(-1.0);
    let r = Float::asin(x);
    assert!((r.hi() - (-core::f64::consts::FRAC_PI_2)).abs() < 1e-14);
}

#[test]
fn test_asin_out_of_range_nan() {
    let x = Float106::from(2.0);
    assert!(Float::asin(x).is_nan());
}

#[test]
fn test_acos_out_of_range_nan() {
    let x = Float106::from(2.0);
    assert!(Float::acos(x).is_nan());
}

#[test]
fn test_acosh_below_one_nan() {
    let x = Float106::from(0.5);
    assert!(Float::acosh(x).is_nan());
}

#[test]
fn test_atanh_one_infinity() {
    let x = Float106::from(1.0);
    assert!(Float::atanh(x).is_infinite());
}

#[test]
fn test_atanh_neg_one_neg_infinity() {
    let x = Float106::from(-1.0);
    let r = Float::atanh(x);
    assert!(r.is_infinite() && Float::is_sign_negative(r));
}

#[test]
fn test_atanh_out_of_range_nan() {
    let x = Float106::from(2.0);
    assert!(Float::atanh(x).is_nan());
}

#[test]
fn test_sinh_basic() {
    let x = Float106::from(1.0);
    assert!((Float::sinh(x).hi() - 1.0_f64.sinh()).abs() < 1e-10);
}

#[test]
fn test_cosh_basic() {
    let x = Float106::from(1.0);
    assert!((Float::cosh(x).hi() - 1.0_f64.cosh()).abs() < 1e-10);
}

#[test]
fn test_tanh_basic() {
    let x = Float106::from(0.5);
    assert!((Float::tanh(x).hi() - 0.5_f64.tanh()).abs() < 1e-10);
}

#[test]
fn test_asinh_basic() {
    let x = Float106::from(0.5);
    assert!((Float::asinh(x).hi() - 0.5_f64.asinh()).abs() < 1e-10);
}

#[test]
fn test_acosh_basic() {
    let x = Float106::from(2.0);
    assert!((Float::acosh(x).hi() - 2.0_f64.acosh()).abs() < 1e-10);
}

#[test]
fn test_atanh_basic() {
    let x = Float106::from(0.5);
    assert!((Float::atanh(x).hi() - 0.5_f64.atanh()).abs() < 1e-10);
}

#[test]
fn test_sin_cos_basic() {
    let x = Float106::from(0.5);
    let (s, c) = Float::sin_cos(x);
    assert!((s.hi() - 0.5_f64.sin()).abs() < 1e-12);
    assert!((c.hi() - 0.5_f64.cos()).abs() < 1e-12);
}

#[test]
fn test_exp_m1_small() {
    let x = Float106::from(0.1);
    assert!((Float::exp_m1(x).hi() - 0.1_f64.exp_m1()).abs() < 1e-12);
}

#[test]
fn test_exp_m1_large() {
    let x = Float106::from(2.0);
    assert!((Float::exp_m1(x).hi() - 2.0_f64.exp_m1()).abs() < 1e-10);
}

#[test]
fn test_ln_1p_small() {
    let x = Float106::from(0.1);
    assert!((Float::ln_1p(x).hi() - 0.1_f64.ln_1p()).abs() < 1e-12);
}

#[test]
fn test_ln_1p_large() {
    let x = Float106::from(2.0);
    assert!((Float::ln_1p(x).hi() - 2.0_f64.ln_1p()).abs() < 1e-10);
}

#[test]
fn test_tan_basic() {
    let x = Float106::from(0.5);
    assert!((Float::tan(x).hi() - 0.5_f64.tan()).abs() < 1e-12);
}

#[test]
fn test_to_degrees() {
    // π at the type's own precision converts to exactly 180 degrees.
    assert_exact(Float::to_degrees(Float106::PI), 180.0);
    assert_exact(Float::to_degrees(Float106::TWO_PI), 360.0);
    assert_exact(Float::to_degrees(Float106::FRAC_PI_2), 90.0);
    assert_exact(Float::to_degrees(Float106::from(0.0)), 0.0);

    // The f64 π is not π: it is short by about 1.22e-16, and the conversion reports that
    // rather than rounding it away. A tolerance of 1e-14 here would hide the difference
    // between the two constants, which is the whole reason the wide type exists.
    let f64_pi = Float106::from(core::f64::consts::PI);
    let degrees = Float::to_degrees(f64_pi);
    assert_eq!(degrees.hi(), 180.0);
    assert!(
        degrees.lo() < 0.0,
        "the f64 pi is below pi, so its degree measure is below 180"
    );
    assert!(degrees != Float106::from(180.0));
}

#[test]
fn test_to_radians() {
    // 180 degrees is π at the full precision of the type, not merely to f64.
    let r = Float::to_radians(Float106::from(180.0));
    assert_eq!(r.hi(), Float106::PI.hi());
    assert!(rel_err(r, Float106::PI) <= TOL);
    // Round trip, over a spread of angles including the negative and zero cases.
    for deg in [0.0_f64, 30.0, 45.0, 90.0, 180.0, 360.0, -90.0, -270.0] {
        let d = Float106::from(deg);
        assert!(
            rel_err(Float::to_degrees(Float::to_radians(d)), d) <= TOL,
            "round trip at {deg}"
        );
    }
}

#[test]
fn test_copysign_positive_sign() {
    let x = Float106::from(-3.0);
    let s = Float106::from(2.0);
    assert_exact(Float::copysign(x, s), 3.0);
}

#[test]
fn test_copysign_negative_sign() {
    let x = Float106::from(3.0);
    let s = Float106::from(-2.0);
    assert_exact(Float::copysign(x, s), -3.0);
}

#[test]
fn test_integer_decode() {
    let x = Float106::from(42.0);
    let (m, e, s) = Float::integer_decode(x);
    let (m_ref, e_ref, s_ref) = 42.0_f64.integer_decode();
    assert_eq!(m, m_ref);
    assert_eq!(e, e_ref);
    assert_eq!(s, s_ref);
}

#[test]
fn test_floor_integer_input() {
    // Triggers the hi_floor == hi branch
    let x = Float106::new(3.0, 0.7);
    let r = Float::floor(x);
    assert_exact(r, 3.0);
}

#[test]
fn test_ceil_integer_input() {
    let x = Float106::new(3.0, 0.3);
    let r = Float::ceil(x);
    assert_exact(r, 4.0);
}

#[test]
fn test_round_integer_input() {
    let x = Float106::new(3.0, 0.6);
    let r = Float::round(x);
    assert_exact(r, 4.0);
}

#[test]
fn test_trunc_integer_input() {
    let x = Float106::new(3.0, 0.8);
    let r = Float::trunc(x);
    assert_exact(r, 3.0);
}
