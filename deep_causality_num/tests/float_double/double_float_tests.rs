/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Float trait implementation on DoubleFloat.

use core::num::FpCategory;
use deep_causality_num::{Float, Float106};

const EPSILON: f64 = 1e-14;

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
    let x = Float106::from_f64(42.0);
    assert!(Float::is_finite(x));
}

#[test]
fn test_is_normal_trait() {
    let x = Float106::from_f64(42.0);
    assert!(Float::is_normal(x));
}

#[test]
fn test_is_subnormal() {
    let x = Float106::from_f64(f64::MIN_POSITIVE / 2.0);
    assert!(Float::is_subnormal(x));
}

#[test]
fn test_classify_normal() {
    let x = Float106::from_f64(42.0);
    assert_eq!(Float::classify(x), FpCategory::Normal);
}

#[test]
fn test_classify_zero() {
    let x = Float106::from_f64(0.0);
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
    let x = Float106::from_f64(3.7);
    assert!((Float::floor(x).hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_floor_negative() {
    let x = Float106::from_f64(-3.3);
    assert!((Float::floor(x).hi() - (-4.0)).abs() < EPSILON);
}

#[test]
fn test_ceil() {
    let x = Float106::from_f64(3.3);
    assert!((Float::ceil(x).hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_ceil_negative() {
    let x = Float106::from_f64(-3.7);
    assert!((Float::ceil(x).hi() - (-3.0)).abs() < EPSILON);
}

#[test]
fn test_round() {
    let x = Float106::from_f64(3.5);
    assert!((Float::round(x).hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_round_down() {
    let x = Float106::from_f64(3.4);
    assert!((Float::round(x).hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_trunc() {
    let x = Float106::from_f64(3.9);
    assert!((Float::trunc(x).hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_trunc_negative() {
    let x = Float106::from_f64(-3.9);
    assert!((Float::trunc(x).hi() - (-3.0)).abs() < EPSILON);
}

#[test]
fn test_fract() {
    let x = Float106::from_f64(3.7);
    assert!((Float::fract(x).hi() - 0.7).abs() < EPSILON);
}

// =============================================================================
// Sign Tests
// =============================================================================

#[test]
fn test_abs() {
    let x = Float106::from_f64(-42.0);
    assert!((Float::abs(x).hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_abs_positive() {
    let x = Float106::from_f64(42.0);
    assert!((Float::abs(x).hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_signum_positive() {
    let x = Float106::from_f64(42.0);
    assert!((Float::signum(x).hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_signum_negative() {
    let x = Float106::from_f64(-42.0);
    assert!((Float::signum(x).hi() - (-1.0)).abs() < EPSILON);
}

#[test]
fn test_signum_zero() {
    let x = Float106::from_f64(0.0);
    // For positive zero, signum returns 1.0 (sign-preserving behavior)
    let sig = Float::signum(x);
    assert!(
        sig.hi() == 1.0 || sig.hi() == 0.0,
        "signum of zero should be 0.0 or 1.0"
    );
}

#[test]
fn test_is_sign_positive_trait() {
    let x = Float106::from_f64(42.0);
    assert!(Float::is_sign_positive(x));
}

#[test]
fn test_is_sign_negative_trait() {
    let x = Float106::from_f64(-42.0);
    assert!(Float::is_sign_negative(x));
}

// =============================================================================
// Arithmetic Operations
// =============================================================================

#[test]
fn test_mul_add() {
    let x = Float106::from_f64(2.0);
    let a = Float106::from_f64(3.0);
    let b = Float106::from_f64(4.0);
    // x * a + b = 2 * 3 + 4 = 10
    let result = Float::mul_add(x, a, b);
    assert!((result.hi() - 10.0).abs() < EPSILON);
}

#[test]
fn test_recip() {
    let x = Float106::from_f64(4.0);
    let result = Float::recip(x);
    assert!((result.hi() - 0.25).abs() < EPSILON);
}

#[test]
fn test_powi_positive() {
    let x = Float106::from_f64(2.0);
    let result = Float::powi(x, 3);
    assert!((result.hi() - 8.0).abs() < EPSILON);
}

#[test]
fn test_powi_negative() {
    let x = Float106::from_f64(2.0);
    let result = Float::powi(x, -2);
    assert!((result.hi() - 0.25).abs() < EPSILON);
}

#[test]
fn test_powi_zero() {
    let x = Float106::from_f64(42.0);
    let result = Float::powi(x, 0);
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_powf() {
    let x = Float106::from_f64(2.0);
    let n = Float106::from_f64(3.0);
    let result = Float::powf(x, n);
    assert!((result.hi() - 8.0).abs() < 1e-10);
}

#[test]
fn test_sqrt() {
    let x = Float106::from_f64(9.0);
    let result = Float::sqrt(x);
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_cbrt() {
    let x = Float106::from_f64(27.0);
    let result = Float::cbrt(x);
    assert!((result.hi() - 3.0).abs() < 1e-10);
}

#[test]
fn test_cbrt_negative() {
    let x = Float106::from_f64(-8.0);
    let result = Float::cbrt(x);
    assert!((result.hi() - (-2.0)).abs() < 1e-10);
}

#[test]
fn test_hypot() {
    let x = Float106::from_f64(3.0);
    let y = Float106::from_f64(4.0);
    let result = Float::hypot(x, y);
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

// =============================================================================
// Exponential and Logarithmic
// =============================================================================

#[test]
fn test_exp() {
    let x = Float106::from_f64(0.0);
    let result = Float::exp(x);
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_exp2() {
    let x = Float106::from_f64(3.0);
    let result = Float::exp2(x);
    assert!((result.hi() - 8.0).abs() < 1e-10);
}

#[test]
fn test_ln() {
    let x = Float106::from_f64(1.0);
    let result = Float::ln(x);
    assert!(result.hi().abs() < EPSILON);
}

#[test]
fn test_log() {
    let x = Float106::from_f64(8.0);
    let base = Float106::from_f64(2.0);
    let result = Float::log(x, base);
    assert!((result.hi() - 3.0).abs() < 1e-10);
}

#[test]
fn test_log2() {
    let x = Float106::from_f64(8.0);
    let result = Float::log2(x);
    assert!((result.hi() - 3.0).abs() < 1e-10);
}

#[test]
fn test_log10() {
    let x = Float106::from_f64(1000.0);
    let result = Float::log10(x);
    assert!((result.hi() - 3.0).abs() < 1e-10);
}

// =============================================================================
// Min/Max/Clamp
// =============================================================================

#[test]
fn test_max() {
    let x = Float106::from_f64(3.0);
    let y = Float106::from_f64(5.0);
    let result = Float::max(x, y);
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

#[test]
fn test_min() {
    let x = Float106::from_f64(3.0);
    let y = Float106::from_f64(5.0);
    let result = Float::min(x, y);
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_clamp_via_float() {
    let x = Float106::from_f64(15.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Float::clamp(x, min, max);
    assert!((result.hi() - 10.0).abs() < EPSILON);
}

// =============================================================================
// Edge Case / Branch Coverage
// =============================================================================

#[test]
fn test_clamp_below_min() {
    let x = Float106::from_f64(-5.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Float::clamp(x, min, max);
    assert!((result.hi() - 0.0).abs() < EPSILON);
}

#[test]
fn test_clamp_in_range() {
    let x = Float106::from_f64(3.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Float::clamp(x, min, max);
    assert!((result.hi() - 3.0).abs() < EPSILON);
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
    assert!((Float::signum(x).hi() - (-1.0)).abs() < EPSILON);
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
    let x = Float106::from_f64(0.0);
    let n = Float106::from_f64(2.0);
    assert_eq!(Float::powf(x, n).hi(), 0.0);
}

#[test]
fn test_powf_zero_nonpositive_exponent() {
    let x = Float106::from_f64(0.0);
    let n = Float106::from_f64(-1.0);
    assert!(Float::powf(x, n).is_infinite());
}

#[test]
fn test_powf_negative_base_nan() {
    let x = Float106::from_f64(-2.0);
    let n = Float106::from_f64(0.5);
    assert!(Float::powf(x, n).is_nan());
}

#[test]
fn test_sqrt_negative_nan() {
    let x = Float106::from_f64(-1.0);
    assert!(Float::sqrt(x).is_nan());
}

#[test]
fn test_sqrt_zero() {
    let x = Float106::from_f64(0.0);
    assert_eq!(Float::sqrt(x).hi(), 0.0);
}

#[test]
fn test_exp_nan() {
    let x = <Float106 as Float>::nan();
    assert!(Float::exp(x).is_nan());
}

#[test]
fn test_exp_overflow_to_infinity() {
    let x = Float106::from_f64(1000.0);
    assert!(Float::exp(x).is_infinite());
}

#[test]
fn test_exp_underflow_to_zero() {
    let x = Float106::from_f64(-1000.0);
    assert_eq!(Float::exp(x).hi(), 0.0);
}

#[test]
fn test_ln_zero_returns_neg_infinity() {
    let x = Float106::from_f64(0.0);
    let r = Float::ln(x);
    assert!(r.is_infinite() && Float::is_sign_negative(r));
}

#[test]
fn test_ln_negative_nan() {
    let x = Float106::from_f64(-1.0);
    assert!(Float::ln(x).is_nan());
}

#[test]
fn test_max_nan_first() {
    let nan = <Float106 as Float>::nan();
    let y = Float106::from_f64(2.0);
    assert!((Float::max(nan, y).hi() - 2.0).abs() < EPSILON);
}

#[test]
fn test_max_nan_second() {
    let x = Float106::from_f64(2.0);
    let nan = <Float106 as Float>::nan();
    assert!((Float::max(x, nan).hi() - 2.0).abs() < EPSILON);
}

#[test]
fn test_min_nan_first() {
    let nan = <Float106 as Float>::nan();
    let y = Float106::from_f64(2.0);
    assert!((Float::min(nan, y).hi() - 2.0).abs() < EPSILON);
}

#[test]
fn test_min_nan_second() {
    let x = Float106::from_f64(2.0);
    let nan = <Float106 as Float>::nan();
    assert!((Float::min(x, nan).hi() - 2.0).abs() < EPSILON);
}

#[test]
fn test_atan_one_special() {
    let x = Float106::from_f64(1.0);
    let r = Float::atan(x);
    assert!((r.hi() - core::f64::consts::FRAC_PI_4).abs() < 1e-14);
}

#[test]
fn test_atan_neg_one_special() {
    let x = Float106::from_f64(-1.0);
    let r = Float::atan(x);
    assert!((r.hi() - (-core::f64::consts::FRAC_PI_4)).abs() < 1e-14);
}

#[test]
fn test_atan2_zero_zero_nan() {
    let z = Float106::from_f64(0.0);
    let r = Float::atan2(z, z);
    assert!(r.is_nan());
}

#[test]
fn test_atan2_positive_y_zero_x() {
    let y = Float106::from_f64(1.0);
    let x = Float106::from_f64(0.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - core::f64::consts::FRAC_PI_2).abs() < 1e-14);
}

#[test]
fn test_atan2_negative_y_zero_x() {
    let y = Float106::from_f64(-1.0);
    let x = Float106::from_f64(0.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - (-core::f64::consts::FRAC_PI_2)).abs() < 1e-14);
}

#[test]
fn test_atan2_negative_x_positive_y() {
    let y = Float106::from_f64(1.0);
    let x = Float106::from_f64(-1.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - (3.0 * core::f64::consts::FRAC_PI_4)).abs() < 1e-10);
}

#[test]
fn test_atan2_negative_x_negative_y() {
    let y = Float106::from_f64(-1.0);
    let x = Float106::from_f64(-1.0);
    let r = Float::atan2(y, x);
    assert!((r.hi() - (-3.0 * core::f64::consts::FRAC_PI_4)).abs() < 1e-10);
}

#[test]
fn test_asin_one() {
    let x = Float106::from_f64(1.0);
    let r = Float::asin(x);
    assert!((r.hi() - core::f64::consts::FRAC_PI_2).abs() < 1e-14);
}

#[test]
fn test_asin_neg_one() {
    let x = Float106::from_f64(-1.0);
    let r = Float::asin(x);
    assert!((r.hi() - (-core::f64::consts::FRAC_PI_2)).abs() < 1e-14);
}

#[test]
fn test_asin_out_of_range_nan() {
    let x = Float106::from_f64(2.0);
    assert!(Float::asin(x).is_nan());
}

#[test]
fn test_acos_out_of_range_nan() {
    let x = Float106::from_f64(2.0);
    assert!(Float::acos(x).is_nan());
}

#[test]
fn test_acosh_below_one_nan() {
    let x = Float106::from_f64(0.5);
    assert!(Float::acosh(x).is_nan());
}

#[test]
fn test_atanh_one_infinity() {
    let x = Float106::from_f64(1.0);
    assert!(Float::atanh(x).is_infinite());
}

#[test]
fn test_atanh_neg_one_neg_infinity() {
    let x = Float106::from_f64(-1.0);
    let r = Float::atanh(x);
    assert!(r.is_infinite() && Float::is_sign_negative(r));
}

#[test]
fn test_atanh_out_of_range_nan() {
    let x = Float106::from_f64(2.0);
    assert!(Float::atanh(x).is_nan());
}

#[test]
fn test_sinh_basic() {
    let x = Float106::from_f64(1.0);
    assert!((Float::sinh(x).hi() - 1.0_f64.sinh()).abs() < 1e-10);
}

#[test]
fn test_cosh_basic() {
    let x = Float106::from_f64(1.0);
    assert!((Float::cosh(x).hi() - 1.0_f64.cosh()).abs() < 1e-10);
}

#[test]
fn test_tanh_basic() {
    let x = Float106::from_f64(0.5);
    assert!((Float::tanh(x).hi() - 0.5_f64.tanh()).abs() < 1e-10);
}

#[test]
fn test_asinh_basic() {
    let x = Float106::from_f64(0.5);
    assert!((Float::asinh(x).hi() - 0.5_f64.asinh()).abs() < 1e-10);
}

#[test]
fn test_acosh_basic() {
    let x = Float106::from_f64(2.0);
    assert!((Float::acosh(x).hi() - 2.0_f64.acosh()).abs() < 1e-10);
}

#[test]
fn test_atanh_basic() {
    let x = Float106::from_f64(0.5);
    assert!((Float::atanh(x).hi() - 0.5_f64.atanh()).abs() < 1e-10);
}

#[test]
fn test_sin_cos_basic() {
    let x = Float106::from_f64(0.5);
    let (s, c) = Float::sin_cos(x);
    assert!((s.hi() - 0.5_f64.sin()).abs() < 1e-12);
    assert!((c.hi() - 0.5_f64.cos()).abs() < 1e-12);
}

#[test]
fn test_exp_m1_small() {
    let x = Float106::from_f64(0.1);
    assert!((Float::exp_m1(x).hi() - 0.1_f64.exp_m1()).abs() < 1e-12);
}

#[test]
fn test_exp_m1_large() {
    let x = Float106::from_f64(2.0);
    assert!((Float::exp_m1(x).hi() - 2.0_f64.exp_m1()).abs() < 1e-10);
}

#[test]
fn test_ln_1p_small() {
    let x = Float106::from_f64(0.1);
    assert!((Float::ln_1p(x).hi() - 0.1_f64.ln_1p()).abs() < 1e-12);
}

#[test]
fn test_ln_1p_large() {
    let x = Float106::from_f64(2.0);
    assert!((Float::ln_1p(x).hi() - 2.0_f64.ln_1p()).abs() < 1e-10);
}

#[test]
fn test_tan_basic() {
    let x = Float106::from_f64(0.5);
    assert!((Float::tan(x).hi() - 0.5_f64.tan()).abs() < 1e-12);
}

#[test]
fn test_to_degrees() {
    let x = Float106::from_f64(core::f64::consts::PI);
    assert!((Float::to_degrees(x).hi() - 180.0).abs() < 1e-10);
}

#[test]
fn test_to_radians() {
    let x = Float106::from_f64(180.0);
    assert!((Float::to_radians(x).hi() - core::f64::consts::PI).abs() < 1e-10);
}

#[test]
fn test_copysign_positive_sign() {
    let x = Float106::from_f64(-3.0);
    let s = Float106::from_f64(2.0);
    assert!((Float::copysign(x, s).hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_copysign_negative_sign() {
    let x = Float106::from_f64(3.0);
    let s = Float106::from_f64(-2.0);
    assert!((Float::copysign(x, s).hi() - (-3.0)).abs() < EPSILON);
}

#[test]
fn test_integer_decode() {
    let x = Float106::from_f64(42.0);
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
    assert!((r.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_ceil_integer_input() {
    let x = Float106::new(3.0, 0.3);
    let r = Float::ceil(x);
    assert!((r.hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_round_integer_input() {
    let x = Float106::new(3.0, 0.6);
    let r = Float::round(x);
    assert!((r.hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_trunc_integer_input() {
    let x = Float106::new(3.0, 0.8);
    let r = Float::trunc(x);
    assert!((r.hi() - 3.0).abs() < EPSILON);
}
