/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::Float;
use std::num::FpCategory;

// Here we have to explicit cast into the trait type by using <f32 as Float> and Float::
// to prevent the Rust compiler to optimize away the trait dispatch for most methods.

// Constants
#[test]
fn test_nan() {
    assert!(<f32 as Float>::nan().is_nan());
}
#[test]
fn test_infinity() {
    assert!(<f32 as Float>::infinity().is_infinite());
}
#[test]
fn test_neg_infinity() {
    assert!(<f32 as Float>::neg_infinity().is_infinite());
}
#[test]
fn test_neg_zero() {
    assert_eq!(<f32 as Float>::neg_zero(), -0.0f32);
}
#[test]
fn test_min_value() {
    assert_eq!(<f32 as Float>::min_value(), f32::MIN);
}
#[test]
fn test_min_positive_value() {
    assert_eq!(<f32 as Float>::min_positive_value(), f32::MIN_POSITIVE);
}
#[test]
fn test_epsilon() {
    assert_eq!(<f32 as Float>::epsilon(), f32::EPSILON);
}
#[test]
fn test_max_value() {
    assert_eq!(<f32 as Float>::max_value(), f32::MAX);
}

// Methods
#[test]
fn is_nan_true() {
    assert!(Float::is_nan(f32::NAN));
}
#[test]
fn is_nan_false() {
    assert!(!Float::is_nan(1.0f32));
}
#[test]
fn is_infinite_true() {
    assert!(Float::is_infinite(f32::INFINITY));
}
#[test]
fn is_infinite_false() {
    assert!(!Float::is_infinite(1.0f32));
}
#[test]
fn is_finite_true() {
    assert!(Float::is_finite(1.0f32));
}
#[test]
fn is_finite_false() {
    assert!(!Float::is_finite(f32::INFINITY));
}
#[test]
fn is_normal_true() {
    assert!(Float::is_normal(1.0f32));
}
#[test]
fn is_normal_false() {
    assert!(!Float::is_normal(0.0f32));
}
#[test]
fn is_subnormal_true() {
    assert!(Float::is_subnormal(f32::MIN_POSITIVE / 2.0));
}
#[test]
fn is_subnormal_false() {
    assert!(!Float::is_subnormal(1.0f32));
}
#[test]
fn classify_normal() {
    assert_eq!(Float::classify(1.0f32), FpCategory::Normal);
}
#[test]
fn classify_infinite() {
    assert_eq!(Float::classify(f32::INFINITY), FpCategory::Infinite);
}
#[test]
fn floor_val() {
    assert_eq!(Float::floor(3.9f32), 3.0);
}
#[test]
fn ceil_val() {
    assert_eq!(Float::ceil(3.1f32), 4.0);
}
#[test]
fn round_val() {
    assert_eq!(Float::round(3.5f32), 4.0);
}
#[test]
fn trunc_val() {
    assert_eq!(Float::trunc(3.9f32), 3.0);
}
#[test]
fn fract_val() {
    assert_eq!(Float::fract(3.5f32), 0.5);
}
#[test]
fn abs_val() {
    assert_eq!(Float::abs(-3.0f32), 3.0);
}
#[test]
fn abs_nan() {
    assert!(Float::abs(f32::NAN).is_nan());
}
#[test]
fn signum_pos() {
    assert_eq!(Float::signum(3.0f32), 1.0);
}
#[test]
fn signum_neg() {
    assert_eq!(Float::signum(-3.0f32), -1.0);
}
#[test]
fn signum_zero() {
    assert_eq!(Float::signum(0.0f32), 1.0);
}
#[test]
fn signum_nan() {
    assert!(Float::signum(f32::NAN).is_nan());
}
#[test]
fn is_sign_positive_true() {
    assert!(Float::is_sign_positive(1.0f32));
}
#[test]
fn is_sign_positive_false() {
    assert!(!Float::is_sign_positive(-1.0f32));
}
#[test]
fn is_sign_negative_true() {
    assert!(Float::is_sign_negative(-1.0f32));
}
#[test]
fn is_sign_negative_false() {
    assert!(!Float::is_sign_negative(1.0f32));
}
#[test]
fn mul_add_val() {
    assert_eq!(Float::mul_add(2.0f32, 3.0, 4.0), 10.0);
}
#[test]
fn recip_val() {
    assert_eq!(Float::recip(2.0f32), 0.5);
}
#[test]
fn powi_val() {
    assert_eq!(Float::powi(2.0f32, 3), 8.0);
}
#[test]
fn powf_val() {
    assert_eq!(Float::powf(2.0f32, 3.0), 8.0);
}
#[test]
fn sqrt_val() {
    assert_eq!(Float::sqrt(4.0f32), 2.0);
}
#[test]
fn sqrt_neg() {
    assert!(Float::sqrt(-1.0f32).is_nan());
}
#[test]
fn exp_val() {
    assert_eq!(Float::exp(1.0f32), std::f32::consts::E);
}
#[test]
fn exp2_val() {
    assert_eq!(Float::exp2(3.0f32), 8.0);
}
#[test]
fn ln_val() {
    assert!((Float::ln(std::f32::consts::E) - 1.0).abs() < 1e-6);
}
#[test]
fn log_val() {
    assert_eq!(Float::log(10.0f32, 10.0), 1.0);
}
#[test]
fn log2_val() {
    assert_eq!(Float::log2(8.0f32), 3.0);
}
#[test]
fn log10_val() {
    assert_eq!(Float::log10(100.0f32), 2.0);
}
#[test]
fn to_degrees_val() {
    assert_eq!(Float::to_degrees(std::f32::consts::PI), 180.0);
}
#[test]
fn to_radians_val() {
    assert_eq!(Float::to_radians(180.0f32), std::f32::consts::PI);
}
#[test]
fn max_val() {
    assert_eq!(Float::max(1.0f32, 2.0), 2.0);
}
#[test]
fn min_val() {
    assert_eq!(Float::min(1.0f32, 2.0), 1.0);
}
#[test]
fn clamp_val() {
    assert_eq!(Float::clamp(1.5f32, 1.0, 2.0), 1.5);
}
#[test]
fn clamp_low() {
    assert_eq!(Float::clamp(0.5f32, 1.0, 2.0), 1.0);
}
#[test]
fn clamp_high() {
    assert_eq!(Float::clamp(2.5f32, 1.0, 2.0), 2.0);
}
#[test]
fn cbrt_val() {
    assert_eq!(Float::cbrt(8.0f32), 2.0);
}
#[test]
fn hypot_val() {
    assert_eq!(Float::hypot(3.0f32, 4.0), 5.0);
}
#[test]
fn sin_val() {
    assert!((Float::sin(std::f32::consts::PI / 2.0) - 1.0).abs() < 1e-6);
}
#[test]
fn cos_val() {
    assert!((Float::cos(std::f32::consts::PI) - -1.0).abs() < 1e-6);
}
#[test]
fn tan_val() {
    assert!((Float::tan(std::f32::consts::PI / 4.0) - 1.0).abs() < 1e-6);
}
#[test]
fn asin_val() {
    assert!((Float::asin(1.0f32) - (std::f32::consts::PI / 2.0)).abs() < 1e-6);
}
#[test]
fn acos_val() {
    assert!((Float::acos(0.0f32) - (std::f32::consts::PI / 2.0)).abs() < 1e-6);
}
#[test]
fn atan_val() {
    assert!((Float::atan(1.0f32) - (std::f32::consts::PI / 4.0)).abs() < 1e-6);
}
#[test]
fn atan2_val() {
    assert!((Float::atan2(1.0f32, 1.0) - (std::f32::consts::PI / 4.0)).abs() < 1e-6);
}
#[test]
fn sin_cos_val() {
    let (s, c) = Float::sin_cos(std::f32::consts::PI / 2.0);
    assert!((s - 1.0).abs() < 1e-6);
    assert!((c - 0.0).abs() < 1e-6);
}
#[test]
fn exp_m1_val() {
    assert_eq!(Float::exp_m1(0.0f32), 0.0);
}
#[test]
fn ln_1p_val() {
    assert_eq!(Float::ln_1p(0.0f32), 0.0);
}
#[test]
fn sinh_val() {
    assert_eq!(Float::sinh(0.0f32), 0.0);
}
#[test]
fn cosh_val() {
    assert_eq!(Float::cosh(0.0f32), 1.0);
}
#[test]
fn tanh_val() {
    assert_eq!(Float::tanh(0.0f32), 0.0);
}
#[test]
fn asinh_val() {
    assert_eq!(Float::asinh(0.0f32), 0.0);
}
#[test]
fn acosh_val() {
    assert_eq!(Float::acosh(1.0f32), 0.0);
}
#[test]
fn atanh_val() {
    assert_eq!(Float::atanh(0.0f32), 0.0);
}
#[test]
fn copysign_pos() {
    assert_eq!(Float::copysign(1.0f32, 2.0), 1.0);
}
#[test]
fn copysign_neg() {
    assert_eq!(Float::copysign(1.0f32, -2.0), -1.0);
}
#[test]
fn copysign_nan() {
    assert!(Float::copysign(f32::NAN, 1.0).is_nan());
    assert!(!Float::copysign(f32::NAN, -1.0).is_sign_positive());
}
#[test]
fn integer_decode_normal() {
    let (mantissa, exponent, sign) = f32::integer_decode(2.0f32);
    assert_eq!(mantissa, 8388608);
    assert_eq!(exponent, -22);
    assert_eq!(sign, 1);
}
#[test]
fn integer_decode_zero() {
    let (mantissa, exponent, sign) = f32::integer_decode(0.0f32);
    assert_eq!(mantissa, 0);
    assert_eq!(exponent, -150);
    assert_eq!(sign, 1);
}
#[test]
fn integer_decode_subnormal() {
    let subnormal = f32::MIN_POSITIVE / 2.0;
    let (mantissa, exponent, sign) = f32::integer_decode(subnormal);
    assert_eq!(mantissa, 8388608);
    assert_eq!(exponent, -150);
    assert_eq!(sign, 1);
}
