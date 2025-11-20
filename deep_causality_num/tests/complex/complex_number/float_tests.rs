/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::utils_tests::utils_complex_tests;
use deep_causality_num::{Complex, ComplexNumber, Float, Zero};
use std::f64::consts::{E, PI};
use std::num::FpCategory;

// Float trait tests
#[test]
fn test_float_nan() {
    let c = Complex::<f64>::nan();
    assert!(c.is_nan());
}

#[test]
fn test_float_infinity() {
    let c = Complex::<f64>::infinity();
    assert!(c.is_infinite());
    assert_eq!(c.re(), f64::INFINITY);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_float_neg_infinity() {
    let c = Complex::<f64>::neg_infinity();
    assert!(c.is_infinite());
    assert_eq!(c.re(), f64::NEG_INFINITY);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_float_neg_zero() {
    let c = Complex::<f64>::neg_zero();
    assert_eq!(c.re(), -0.0);
    assert_eq!(c.im(), -0.0);
}

#[test]
fn test_float_min_value() {
    let c = Complex::<f64>::min_value();
    assert_eq!(c.re(), f64::MIN);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_float_min_positive_value() {
    let c = Complex::<f64>::min_positive_value();
    assert_eq!(c.re(), f64::MIN_POSITIVE);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_float_epsilon() {
    let c = Complex::<f64>::epsilon();
    assert_eq!(c.re(), f64::EPSILON);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_float_max_value() {
    let c = Complex::<f64>::max_value();
    assert_eq!(c.re(), f64::MAX);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_float_is_nan() {
    let c1 = Complex::new(1.0, f64::NAN);
    let c2 = Complex::new(f64::NAN, 2.0);
    let c3 = Complex::new(1.0, 2.0);
    assert!(c1.is_nan());
    assert!(c2.is_nan());
    assert!(!c3.is_nan());
}

#[test]
fn test_float_is_infinite() {
    let c1 = Complex::new(1.0, f64::INFINITY);
    let c2 = Complex::new(f64::NEG_INFINITY, 2.0);
    let c3 = Complex::new(1.0, 2.0);
    assert!(c1.is_infinite());
    assert!(c2.is_infinite());
    assert!(!c3.is_infinite());
}

#[test]
fn test_float_is_finite() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(1.0, f64::INFINITY);
    assert!(c1.is_finite());
    assert!(!c2.is_finite());
}

#[test]
fn test_float_classify() {
    let c_nan = Complex::new(1.0, f64::NAN);
    let c_inf = Complex::new(f64::INFINITY, 2.0);
    let c_zero = Complex::new(0.0, 0.0);
    let c_normal = Complex::new(1.0, 2.0);

    assert_eq!(c_nan.classify(), FpCategory::Nan);
    assert_eq!(c_inf.classify(), FpCategory::Infinite);
    assert_eq!(c_zero.classify(), FpCategory::Zero);
    assert_eq!(c_normal.classify(), FpCategory::Normal);
}

#[test]
fn test_float_floor() {
    let c = Complex::new(1.5, 2.9);
    let floor_c = c.floor();
    assert_eq!(floor_c.re(), 1.0);
    assert_eq!(floor_c.im(), 2.0);
}

#[test]
fn test_float_ceil() {
    let c = Complex::new(1.1, 2.5);
    let ceil_c = c.ceil();
    assert_eq!(ceil_c.re(), 2.0);
    assert_eq!(ceil_c.im(), 3.0);
}

#[test]
fn test_float_round() {
    let c1 = Complex::new(1.4, 2.6);
    let c2 = Complex::new(1.5, 2.5);
    assert_eq!(c1.round().re(), 1.0);
    assert_eq!(c1.round().im(), 3.0);
    assert_eq!(c2.round().re(), 2.0);
    assert_eq!(c2.round().im(), 3.0);
}

#[test]
fn test_float_trunc() {
    let c = Complex::new(1.9, -2.1);
    let trunc_c = c.trunc();
    assert_eq!(trunc_c.re(), 1.0);
    assert_eq!(trunc_c.im(), -2.0);
}

#[test]
fn test_float_fract() {
    let c = Complex::new(1.9, -2.1);
    let fract_c = c.fract();
    utils_complex_tests::assert_approx_eq(fract_c.re(), 0.9, 1e-9);
    utils_complex_tests::assert_approx_eq(fract_c.im(), -0.1, 1e-9);
}

#[test]
fn test_float_abs() {
    let c = Complex::new(3.0, 4.0);
    let abs_c = c.abs();
    assert_eq!(abs_c.re(), 5.0);
    assert_eq!(abs_c.im(), 0.0);
}

#[test]
fn test_float_signum() {
    let c = Complex::new(3.0, 4.0);
    let sign_c = c.signum();
    utils_complex_tests::assert_complex_approx_eq(sign_c, Complex::new(0.6, 0.8), 1e-9);
    let zero_c = Complex::<f64>::zero();
    assert!(zero_c.signum().is_nan());
}

#[test]
fn test_float_is_sign_positive() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(-1.0, 2.0);
    let c3 = Complex::new(1.0, -2.0);
    let c4 = Complex::new(0.0, 2.0);
    let c5 = Complex::new(0.0, -2.0);
    assert!(c1.is_sign_positive());
    assert!(!c2.is_sign_positive());
    assert!(c3.is_sign_positive());
    assert!(c4.is_sign_positive());
    assert!(!c5.is_sign_positive());
}

#[test]
fn test_float_is_sign_negative() {
    let c1 = Complex::new(-1.0, -2.0);
    let c2 = Complex::new(1.0, -2.0);
    let c3 = Complex::new(-1.0, 2.0);
    let c4 = Complex::new(0.0, 2.0);
    let c5 = Complex::new(0.0, -2.0);
    assert!(c1.is_sign_negative());
    assert!(!c2.is_sign_negative());
    assert!(c3.is_sign_negative());
    assert!(!c4.is_sign_negative());
    assert!(c5.is_sign_negative());
}

#[test]
fn test_float_mul_add() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(3.0, 4.0);
    let c3 = Complex::new(5.0, 6.0);
    let result = c1.mul_add(c2, c3);
    // (1+2i)*(3+4i) + (5+6i) = (-5+10i) + (5+6i) = 0+16i
    utils_complex_tests::assert_complex_approx_eq(result, Complex::new(0.0, 16.0), 1e-9);
}

#[test]
fn test_float_recip() {
    let c = Complex::new(1.0, 2.0);
    let recip_c = c.recip();
    // 1/(1+2i) = (1-2i)/(1+4) = 1/5 - 2/5 i
    utils_complex_tests::assert_complex_approx_eq(recip_c, Complex::new(0.2, -0.4), 1e-9);
}

#[test]
fn test_float_powi() {
    let c = Complex::new(1.0, 1.0);
    let c_sq = c.powi(2); // (1+i)^2 = 1 + 2i - 1 = 2i
    utils_complex_tests::assert_complex_approx_eq(c_sq, Complex::new(0.0, 2.0), 1e-9);

    let c_neg = c.powi(-1); // 1/(1+i) = (1-i)/2 = 0.5 - 0.5i
    utils_complex_tests::assert_complex_approx_eq(c_neg, Complex::new(0.5, -0.5), 1e-9);

    let c_zero = c.powi(0);
    utils_complex_tests::assert_complex_approx_eq(c_zero, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_powf() {
    let c = Complex::new(E, 0.0); // e
    let n = Complex::new(0.0, PI / 2.0); // i * pi/2
    let result = c.powf(n); // e^(i*pi/2) = cos(pi/2) + i*sin(pi/2) = i
    utils_complex_tests::assert_complex_approx_eq(result, Complex::new(0.0, 1.0), 1e-9);
}

#[test]
fn test_float_sqrt() {
    let c = Complex::new(0.0, 2.0); // 2i
    let sqrt_c = c.sqrt(); // sqrt(2i) = 1+i
    utils_complex_tests::assert_complex_approx_eq(sqrt_c, Complex::new(1.0, 1.0), 1e-9);
}

#[test]
fn test_float_exp() {
    let c = Complex::new(0.0, PI); // i*pi
    let exp_c = c.exp(); // e^(i*pi) = cos(pi) + i*sin(pi) = -1
    utils_complex_tests::assert_complex_approx_eq(exp_c, Complex::new(-1.0, 0.0), 1e-9);
}

#[test]
fn test_float_exp2() {
    let c = Complex::new(2.0, 0.0);
    let exp2_c = c.exp2(); // 2^2 = 4
    utils_complex_tests::assert_complex_approx_eq(exp2_c, Complex::new(4.0, 0.0), 1e-9);
}

#[test]
fn test_float_ln() {
    let c = Complex::new(-1.0, 0.0); // -1
    let ln_c = c.ln(); // ln(-1) = i*pi
    utils_complex_tests::assert_complex_approx_eq(ln_c, Complex::new(0.0, PI), 1e-9);
}

#[test]
fn test_float_log() {
    let c = Complex::new(E, 0.0);
    let base = Complex::new(E, 0.0);
    let log_c = c.log(base);
    utils_complex_tests::assert_complex_approx_eq(log_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_log2() {
    let c = Complex::new(4.0, 0.0);
    let log2_c = c.log2();
    utils_complex_tests::assert_complex_approx_eq(log2_c, Complex::new(2.0, 0.0), 1e-9);
}

#[test]
fn test_float_log10() {
    let c = Complex::new(100.0, 0.0);
    let log10_c = c.log10();
    utils_complex_tests::assert_complex_approx_eq(log10_c, Complex::new(2.0, 0.0), 1e-9);
}

#[test]
fn test_float_cbrt() {
    let c = Complex::new(8.0, 0.0);
    let cbrt_c = c.cbrt();
    utils_complex_tests::assert_complex_approx_eq(cbrt_c, Complex::new(2.0, 0.0), 1e-9);
}

#[test]
fn test_float_hypot() {
    let c1 = Complex::new(3.0, 0.0);
    let c2 = Complex::new(4.0, 0.0);
    let hypot_c = c1.hypot(c2);
    utils_complex_tests::assert_complex_approx_eq(hypot_c, Complex::new(5.0, 0.0), 1e-9);
}

#[test]
fn test_float_sin() {
    let c = Complex::new(PI / 2.0, 0.0);
    let sin_c = c.sin();
    utils_complex_tests::assert_complex_approx_eq(sin_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_cos() {
    let c = Complex::new(PI, 0.0);
    let cos_c = c.cos();
    utils_complex_tests::assert_complex_approx_eq(cos_c, Complex::new(-1.0, 0.0), 1e-9);
}

#[test]
fn test_float_tan() {
    let c = Complex::new(PI / 4.0, 0.0);
    let tan_c = c.tan();
    utils_complex_tests::assert_complex_approx_eq(tan_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_asin() {
    let c = Complex::new(1.0, 0.0);
    let asin_c = c.asin();
    utils_complex_tests::assert_complex_approx_eq(asin_c, Complex::new(PI / 2.0, 0.0), 1e-9);
}

#[test]
fn test_float_acos() {
    let c = Complex::new(0.0, 0.0);
    let acos_c = c.acos();
    utils_complex_tests::assert_complex_approx_eq(acos_c, Complex::new(PI / 2.0, 0.0), 1e-9);
}

#[test]
fn test_float_atan() {
    let c = Complex::new(1.0, 0.0);
    let atan_c = c.atan();
    utils_complex_tests::assert_complex_approx_eq(atan_c, Complex::new(PI / 4.0, 0.0), 1e-9);
}

#[test]
fn test_float_atan2() {
    let c1 = Complex::new(1.0, 0.0); // y
    let c2 = Complex::new(1.0, 0.0); // x
    let atan2_c = c1.atan2(c2);
    assert_eq!(atan2_c.im(), 0.0);
}

#[test]
fn test_float_exp_m1() {
    let c = Complex::new(0.0, 0.0);
    let exp_m1_c = c.exp_m1();
    utils_complex_tests::assert_complex_approx_eq(exp_m1_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_ln_1p() {
    let c = Complex::new(0.0, 0.0);
    let ln_1p_c = c.ln_1p();
    utils_complex_tests::assert_complex_approx_eq(ln_1p_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_sinh() {
    let c = Complex::new(0.0, 0.0);
    let sinh_c = c.sinh();
    utils_complex_tests::assert_complex_approx_eq(sinh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_cosh() {
    let c = Complex::new(0.0, 0.0);
    let cosh_c = c.cosh();
    utils_complex_tests::assert_complex_approx_eq(cosh_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_tanh() {
    let c = Complex::new(0.0, 0.0);
    let tanh_c = c.tanh();
    utils_complex_tests::assert_complex_approx_eq(tanh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_asinh() {
    let c = Complex::new(0.0, 0.0);
    let asinh_c = c.asinh();
    utils_complex_tests::assert_complex_approx_eq(asinh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_acosh() {
    let c = Complex::new(1.0, 0.0);
    let acosh_c = c.acosh();
    utils_complex_tests::assert_complex_approx_eq(acosh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_atanh() {
    let c = Complex::new(0.0, 0.0);
    let atanh_c = c.atanh();
    utils_complex_tests::assert_complex_approx_eq(atanh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_to_degrees() {
    let c = Complex::new(PI, PI / 2.0);
    let deg_c = c.to_degrees();
    utils_complex_tests::assert_complex_approx_eq(deg_c, Complex::new(180.0, 90.0), 1e-9);
}

#[test]
fn test_float_to_radians() {
    let c = Complex::new(180.0, 90.0);
    let rad_c = c.to_radians();
    utils_complex_tests::assert_complex_approx_eq(rad_c, Complex::new(PI, PI / 2.0), 1e-9);
}

#[test]
fn test_float_max() {
    let c1 = Complex::new(3.0, 4.0); // norm 5
    let c2 = Complex::new(1.0, 1.0); // norm sqrt(2)
    let max_c = c1.max(c2);
    utils_complex_tests::assert_complex_approx_eq(max_c, c1, 1e-9);
}

#[test]
fn test_float_min() {
    let c1 = Complex::new(3.0, 4.0); // norm 5
    let c2 = Complex::new(1.0, 1.0); // norm sqrt(2)
    let min_c = c1.min(c2);
    utils_complex_tests::assert_complex_approx_eq(min_c, c2, 1e-9);
}

#[test]
fn test_float_clamp() {
    let c = Complex::new(2.0, 2.0); // norm sqrt(8) approx 2.82
    let min_c = Complex::new(1.0, 0.0); // norm 1
    let max_c = Complex::new(3.0, 0.0); // norm 3
    let clamped_c = c.clamp(min_c, max_c);
    utils_complex_tests::assert_complex_approx_eq(clamped_c, c, 1e-9);

    let c_low = Complex::new(0.5, 0.0);
    let clamped_low = c_low.clamp(min_c, max_c);
    utils_complex_tests::assert_complex_approx_eq(clamped_low, min_c, 1e-9);

    let c_high = Complex::new(4.0, 0.0);
    let clamped_high = c_high.clamp(min_c, max_c);
    utils_complex_tests::assert_complex_approx_eq(clamped_high, max_c, 1e-9);
}

#[test]
fn test_float_integer_decode() {
    let c = Complex::new(1.5, 2.0);
    let (mantissa, exponent, sign) = c.integer_decode();
    assert_eq!(mantissa, 6755399441055744); // Mantissa for 1.5f64
    assert_eq!(exponent, -52); // Exponent for 1.5f64
    assert_eq!(sign, 1); // Sign for 1.5f64
}

#[test]
fn test_float_sin_cos() {
    let c = Complex::new(PI / 4.0, 0.0);
    let (s, co) = c.sin_cos();
    utils_complex_tests::assert_complex_approx_eq(s, Complex::new((PI / 4.0).sin(), 0.0), 1e-9);
    utils_complex_tests::assert_complex_approx_eq(co, Complex::new((PI / 4.0).cos(), 0.0), 1e-9);
}

#[test]
fn test_float_copysign() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(-3.0, -4.0);
    let result = c1.copysign(c2);
    assert_eq!(result.re(), -1.0);
    assert_eq!(result.im(), -2.0);

    let c3 = Complex::new(-1.0, -2.0);
    let c4 = Complex::new(3.0, 4.0);
    let result2 = c3.copysign(c4);
    assert_eq!(result2.re(), 1.0);
    assert_eq!(result2.im(), 2.0);

    let c5 = Complex::new(1.0, -2.0);
    let c6 = Complex::new(-3.0, 4.0);
    let result3 = c5.copysign(c6);
    assert_eq!(result3.re(), -1.0);
    assert_eq!(result3.im(), 2.0);

    let c7 = Complex::new(0.0, 0.0);
    let c8 = Complex::new(-1.0, -1.0);
    let result4 = c7.copysign(c8);
    assert_eq!(result4.re(), -0.0);
    assert_eq!(result4.im(), -0.0);

    let c9 = Complex::new(f64::nan(), 2.0);
    let c10 = Complex::new(-1.0, -1.0);
    let result5 = c9.copysign(c10);
    assert!(result5.re().is_nan());
    assert_eq!(result5.im(), -2.0);
}

#[test]
fn test_float_sqrt_zero_input() {
    let c = Complex::new(0.0, 0.0);
    let sqrt_c = c.sqrt();
    utils_complex_tests::assert_complex_approx_eq(sqrt_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_sqrt_negative_real() {
    let c = Complex::new(-1.0, 0.0); // sqrt(-1) = i
    let sqrt_c = c.sqrt();
    utils_complex_tests::assert_complex_approx_eq(sqrt_c, Complex::new(0.0, 1.0), 1e-9);
}

#[test]
fn test_float_sqrt_negative_real_with_imaginary() {
    let c = Complex::new(-3.0, 4.0); // sqrt(-3 + 4i) = 1 + 2i
    let sqrt_c = c.sqrt();
    utils_complex_tests::assert_complex_approx_eq(sqrt_c, Complex::new(1.0, 2.0), 1e-9);
}

#[test]
fn test_float_sqrt_positive_real_zero_im_sqrt() {
    // This case is when re_sqrt is zero, which happens when norm + re is zero.
    // This implies re and norm are both zero, which means the complex number is zero.
    // This is already covered by test_float_sqrt_zero_input.
    // However, to explicitly test the branch `if re_sqrt.is_zero()`, we can use a very small number.
    let c = Complex::new(f64::EPSILON, 0.0);
    let sqrt_c = c.sqrt();
    utils_complex_tests::assert_complex_approx_eq(
        sqrt_c,
        Complex::new(f64::EPSILON.sqrt(), 0.0),
        1e-9,
    );
}

#[test]
fn test_float_sqrt_negative_real_zero_im_sqrt_branch() {
    // This case is when im_sqrt is zero, which happens when norm - re is zero.
    // This implies re is negative and im is zero.
    let c = Complex::new(-f64::EPSILON, 0.0);
    let sqrt_c = c.sqrt();
    utils_complex_tests::assert_complex_approx_eq(
        sqrt_c,
        Complex::new(0.0, f64::EPSILON.sqrt()),
        1e-9,
    );
}
