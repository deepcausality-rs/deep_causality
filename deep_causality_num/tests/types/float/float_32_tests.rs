/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::Float;
use std::num::FpCategory;

// Constants
#[test]
fn test_nan() {
    assert!(f32::nan().is_nan());
}
#[test]
fn test_infinity() {
    assert!(f32::infinity().is_infinite());
}
#[test]
fn test_neg_infinity() {
    assert!(f32::neg_infinity().is_infinite());
}
#[test]
fn test_neg_zero() {
    assert_eq!(f32::neg_zero(), -0.0f32);
}
#[test]
fn test_min_value() {
    assert_eq!(f32::min_value(), f32::MIN);
}
#[test]
fn test_min_positive_value() {
    assert_eq!(f32::min_positive_value(), f32::MIN_POSITIVE);
}
#[test]
fn test_epsilon() {
    assert_eq!(f32::epsilon(), f32::EPSILON);
}
#[test]
fn test_max_value() {
    assert_eq!(f32::max_value(), f32::MAX);
}

// Methods
#[test]
fn is_nan_true() {
    assert!(f32::NAN.is_nan());
}
#[test]
fn is_nan_false() {
    assert!(!1.0f32.is_nan());
}
#[test]
fn is_infinite_true() {
    assert!(f32::INFINITY.is_infinite());
}
#[test]
fn is_infinite_false() {
    assert!(!1.0f32.is_infinite());
}
#[test]
fn is_finite_true() {
    assert!(1.0f32.is_finite());
}
#[test]
fn is_finite_false() {
    assert!(!f32::INFINITY.is_finite());
}
#[test]
fn is_normal_true() {
    assert!(1.0f32.is_normal());
}
#[test]
fn is_normal_false() {
    assert!(!0.0f32.is_normal());
}
#[test]
fn is_subnormal_true() {
    assert!((f32::MIN_POSITIVE / 2.0).is_subnormal());
}
#[test]
fn is_subnormal_false() {
    assert!(!1.0f32.is_subnormal());
}
#[test]
fn classify_normal() {
    assert_eq!(1.0f32.classify(), FpCategory::Normal);
}
#[test]
fn classify_infinite() {
    assert_eq!(f32::INFINITY.classify(), FpCategory::Infinite);
}
#[test]
fn floor_val() {
    assert_eq!(3.9f32.floor(), 3.0);
}
#[test]
fn ceil_val() {
    assert_eq!(3.1f32.ceil(), 4.0);
}
#[test]
fn round_val() {
    assert_eq!(3.5f32.round(), 4.0);
}
#[test]
fn trunc_val() {
    assert_eq!(3.9f32.trunc(), 3.0);
}
#[test]
fn fract_val() {
    assert_eq!(3.5f32.fract(), 0.5);
}
#[test]
fn abs_val() {
    assert_eq!((-3.0f32).abs(), 3.0);
}
#[test]
fn abs_nan() {
    assert!(f32::NAN.abs().is_nan());
}
#[test]
fn signum_pos() {
    assert_eq!(3.0f32.signum(), 1.0);
}
#[test]
fn signum_neg() {
    assert_eq!(-3.0f32.signum(), -1.0);
}
#[test]
fn signum_zero() {
    assert_eq!(0.0f32.signum(), 1.0);
}
#[test]
fn signum_nan() {
    assert!(f32::NAN.signum().is_nan());
}
#[test]
fn is_sign_positive_true() {
    assert!(1.0f32.is_sign_positive());
}
#[test]
fn is_sign_positive_false() {
    assert!(!(-1.0f32).is_sign_positive());
}
#[test]
fn is_sign_negative_true() {
    assert!((-1.0f32).is_sign_negative());
}
#[test]
fn is_sign_negative_false() {
    assert!(!1.0f32.is_sign_negative());
}
#[test]
fn mul_add_val() {
    assert_eq!(2.0f32.mul_add(3.0, 4.0), 10.0);
}
#[test]
fn recip_val() {
    assert_eq!(2.0f32.recip(), 0.5);
}
#[test]
fn powi_val() {
    assert_eq!(2.0f32.powi(3), 8.0);
}
#[test]
fn powf_val() {
    assert_eq!(2.0f32.powf(3.0), 8.0);
}
#[test]
fn sqrt_val() {
    assert_eq!(4.0f32.sqrt(), 2.0);
}
#[test]
fn sqrt_neg() {
    assert!((-1.0f32).sqrt().is_nan());
}
#[test]
fn exp_val() {
    assert_eq!(1.0f32.exp(), std::f32::consts::E);
}
#[test]
fn exp2_val() {
    assert_eq!(3.0f32.exp2(), 8.0);
}
#[test]
fn ln_val() {
    assert!((std::f32::consts::E.ln() - 1.0).abs() < 1e-6);
}
#[test]
fn log_val() {
    assert_eq!(10.0f32.log(10.0), 1.0);
}
#[test]
fn log2_val() {
    assert_eq!(8.0f32.log2(), 3.0);
}
#[test]
fn log10_val() {
    assert_eq!(100.0f32.log10(), 2.0);
}
#[test]
fn to_degrees_val() {
    assert_eq!(std::f32::consts::PI.to_degrees(), 180.0);
}
#[test]
fn to_radians_val() {
    assert_eq!(180.0f32.to_radians(), std::f32::consts::PI);
}
#[test]
fn max_val() {
    assert_eq!(1.0f32.max(2.0), 2.0);
}
#[test]
fn min_val() {
    assert_eq!(1.0f32.min(2.0), 1.0);
}
#[test]
fn clamp_val() {
    assert_eq!(1.5f32.clamp(1.0, 2.0), 1.5);
}
#[test]
fn clamp_low() {
    assert_eq!(0.5f32.clamp(1.0, 2.0), 1.0);
}
#[test]
fn clamp_high() {
    assert_eq!(2.5f32.clamp(1.0, 2.0), 2.0);
}
#[test]
fn cbrt_val() {
    assert_eq!(8.0f32.cbrt(), 2.0);
}
#[test]
fn hypot_val() {
    assert_eq!(3.0f32.hypot(4.0), 5.0);
}
#[test]
fn sin_val() {
    assert!(((std::f32::consts::PI / 2.0).sin() - 1.0).abs() < 1e-6);
}
#[test]
fn cos_val() {
    assert!((std::f32::consts::PI.cos() - -1.0).abs() < 1e-6);
}
#[test]
fn tan_val() {
    assert!(((std::f32::consts::PI / 4.0).tan() - 1.0).abs() < 1e-6);
}
#[test]
fn asin_val() {
    assert!((1.0f32.asin() - (std::f32::consts::PI / 2.0)).abs() < 1e-6);
}
#[test]
fn acos_val() {
    assert!((0.0f32.acos() - (std::f32::consts::PI / 2.0)).abs() < 1e-6);
}
#[test]
fn atan_val() {
    assert!((1.0f32.atan() - (std::f32::consts::PI / 4.0)).abs() < 1e-6);
}
#[test]
fn atan2_val() {
    assert!((1.0f32.atan2(1.0) - (std::f32::consts::PI / 4.0)).abs() < 1e-6);
}
#[test]
fn sin_cos_val() {
    let (s, c) = (std::f32::consts::PI / 2.0).sin_cos();
    assert!((s - 1.0).abs() < 1e-6);
    assert!((c - 0.0).abs() < 1e-6);
}
#[test]
fn exp_m1_val() {
    assert_eq!(0.0f32.exp_m1(), 0.0);
}
#[test]
fn ln_1p_val() {
    assert_eq!(0.0f32.ln_1p(), 0.0);
}
#[test]
fn sinh_val() {
    assert_eq!(0.0f32.sinh(), 0.0);
}
#[test]
fn cosh_val() {
    assert_eq!(0.0f32.cosh(), 1.0);
}
#[test]
fn tanh_val() {
    assert_eq!(0.0f32.tanh(), 0.0);
}
#[test]
fn asinh_val() {
    assert_eq!(0.0f32.asinh(), 0.0);
}
#[test]
fn acosh_val() {
    assert_eq!(1.0f32.acosh(), 0.0);
}
#[test]
fn atanh_val() {
    assert_eq!(0.0f32.atanh(), 0.0);
}
#[test]
fn copysign_pos() {
    assert_eq!(1.0f32.copysign(2.0), 1.0);
}
#[test]
fn copysign_neg() {
    assert_eq!(1.0f32.copysign(-2.0), -1.0);
}
#[test]
fn copysign_nan() {
    assert!(f32::NAN.copysign(1.0).is_nan());
    assert!(!f32::NAN.copysign(-1.0).is_sign_positive());
}
#[test]
fn integer_decode_normal() {
    let (mantissa, exponent, sign) = 2.0f32.integer_decode();
    assert_eq!(mantissa, 8388608);
    assert_eq!(exponent, -22);
    assert_eq!(sign, 1);
}
#[test]
fn integer_decode_zero() {
    let (mantissa, exponent, sign) = 0.0f32.integer_decode();
    assert_eq!(mantissa, 0);
    assert_eq!(exponent, -150);
    assert_eq!(sign, 1);
}
#[test]
fn integer_decode_subnormal() {
    let num = 2.0f32;
    let (mantissa, exponent, sign) = num.integer_decode();
    assert_eq!(mantissa, 8388608);
    assert_eq!(exponent, -22);
    assert_eq!(sign, 1);
}
