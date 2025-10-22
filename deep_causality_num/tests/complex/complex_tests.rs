/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{
    AsPrimitive, Complex, ComplexNumber, Float, FromPrimitive, One, ToPrimitive, Zero,
};
use std::f64::consts::{E, PI};
use std::num::FpCategory;

// Helper for approximate equality for floats
fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
    assert!(
        (a - b).abs() < epsilon,
        "{} is not approximately equal to {}",
        a,
        b
    );
}

// Helper for approximate equality for complex numbers
fn assert_complex_approx_eq(a: Complex<f64>, b: Complex<f64>, epsilon: f64) {
    assert_approx_eq(a.re(), b.re(), epsilon);
    assert_approx_eq(a.im(), b.im(), epsilon);
}

#[test]
fn test_complex_new() {
    let c = Complex::new(1.0, 2.0);
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_from_real() {
    let c = Complex::from_real(3.0);
    assert_eq!(c.re(), 3.0);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_complex_norm_sqr() {
    let c = Complex::new(3.0, 4.0);
    assert_eq!(c.norm_sqr(), 25.0);
}

#[test]
fn test_complex_norm() {
    let c = Complex::new(3.0, 4.0);
    assert_eq!(c.norm(), 5.0);
}

#[test]
fn test_complex_arg() {
    let c = Complex::new(1.0, 1.0);
    assert_approx_eq(c.arg(), PI / 4.0, 1e-9);
}

#[test]
fn test_complex_conj() {
    let c = Complex::new(1.0, 2.0);
    let conj_c = c.conj();
    assert_eq!(conj_c.re(), 1.0);
    assert_eq!(conj_c.im(), -2.0);
}

#[test]
fn test_complex_zero() {
    let c = Complex::<f64>::zero();
    assert_eq!(c.re(), 0.0);
    assert_eq!(c.im(), 0.0);
    assert!(c.is_zero());
}

#[test]
fn test_complex_one() {
    let c = Complex::<f64>::one();
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 0.0);
    assert!(c.is_one());
}

#[test]
fn test_complex_add() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(3.0, 4.0);
    let sum = c1 + c2;
    assert_eq!(sum.re(), 4.0);
    assert_eq!(sum.im(), 6.0);
}

#[test]
fn test_complex_add_scalar() {
    let c = Complex::new(1.0, 2.0);
    let sum = c + 3.0;
    assert_eq!(sum.re(), 4.0);
    assert_eq!(sum.im(), 2.0);
}

#[test]
fn test_complex_sub() {
    let c1 = Complex::new(3.0, 4.0);
    let c2 = Complex::new(1.0, 2.0);
    let diff = c1 - c2;
    assert_eq!(diff.re(), 2.0);
    assert_eq!(diff.im(), 2.0);
}

#[test]
fn test_complex_sub_scalar() {
    let c = Complex::new(3.0, 2.0);
    let diff = c - 1.0;
    assert_eq!(diff.re(), 2.0);
    assert_eq!(diff.im(), 2.0);
}

#[test]
fn test_complex_mul() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(3.0, 4.0);
    let prod = c1 * c2;
    assert_eq!(prod.re(), -5.0); // (1*3 - 2*4) = 3 - 8 = -5
    assert_eq!(prod.im(), 10.0); // (1*4 + 2*3) = 4 + 6 = 10
}

#[test]
fn test_complex_mul_scalar() {
    let c = Complex::new(1.0, 2.0);
    let prod = c * 3.0;
    assert_eq!(prod.re(), 3.0);
    assert_eq!(prod.im(), 6.0);
}

#[test]
fn test_complex_div() {
    let c1 = Complex::new(-5.0, 10.0);
    let c2 = Complex::new(1.0, 2.0);
    let quot = c1 / c2;
    assert_complex_approx_eq(quot, Complex::new(3.0, 4.0), 1e-9);
}

#[test]
fn test_complex_div_scalar() {
    let c = Complex::new(3.0, 6.0);
    let quot = c / 3.0;
    assert_eq!(quot.re(), 1.0);
    assert_eq!(quot.im(), 2.0);
}

#[test]
fn test_complex_div_by_zero() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::<f64>::zero();
    let quot = c1 / c2;
    assert!(quot.re().is_nan());
    assert!(quot.im().is_nan());
}

#[test]
fn test_complex_neg() {
    let c = Complex::new(1.0, -2.0);
    let neg_c = -c;
    assert_eq!(neg_c.re(), -1.0);
    assert_eq!(neg_c.im(), 2.0);
}

#[test]
fn test_complex_add_assign() {
    let mut c = Complex::new(1.0, 2.0);
    c += Complex::new(3.0, 4.0);
    assert_eq!(c.re(), 4.0);
    assert_eq!(c.im(), 6.0);
}

#[test]
fn test_complex_add_assign_scalar() {
    let mut c = Complex::new(1.0, 2.0);
    c += 3.0;
    assert_eq!(c.re(), 4.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_sub_assign() {
    let mut c = Complex::new(3.0, 4.0);
    c -= Complex::new(1.0, 2.0);
    assert_eq!(c.re(), 2.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_sub_assign_scalar() {
    let mut c = Complex::new(3.0, 2.0);
    c -= 1.0;
    assert_eq!(c.re(), 2.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_mul_assign() {
    let mut c = Complex::new(1.0, 2.0);
    c *= Complex::new(3.0, 4.0);
    assert_eq!(c.re(), -5.0);
    assert_eq!(c.im(), 10.0);
}

#[test]
fn test_complex_mul_assign_scalar() {
    let mut c = Complex::new(1.0, 2.0);
    c *= 3.0;
    assert_eq!(c.re(), 3.0);
    assert_eq!(c.im(), 6.0);
}

#[test]
fn test_complex_div_assign() {
    let mut c = Complex::new(-5.0, 10.0);
    c /= Complex::new(1.0, 2.0);
    assert_complex_approx_eq(c, Complex::new(3.0, 4.0), 1e-9);
}

#[test]
fn test_complex_div_assign_scalar() {
    let mut c = Complex::new(3.0, 6.0);
    c /= 3.0;
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_rem() {
    let c1 = Complex::new(5.0, 5.0);
    let c2 = Complex::new(2.0, 3.0);
    let rem = c1 % c2;
    assert_eq!(rem.re(), 1.0);
    assert_eq!(rem.im(), 2.0);
}

#[test]
fn test_complex_rem_scalar() {
    let c = Complex::new(5.0, 5.0);
    let rem = c % 2.0;
    assert_eq!(rem.re(), 1.0);
    assert_eq!(rem.im(), 1.0);
}

#[test]
fn test_complex_rem_assign() {
    let mut c = Complex::new(5.0, 5.0);
    c %= Complex::new(2.0, 3.0);
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_rem_assign_scalar() {
    let mut c = Complex::new(5.0, 5.0);
    c %= 2.0;
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 1.0);
}

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
    assert_approx_eq(fract_c.re(), 0.9, 1e-9);
    assert_approx_eq(fract_c.im(), -0.1, 1e-9);
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
    assert_complex_approx_eq(sign_c, Complex::new(0.6, 0.8), 1e-9);
    let zero_c = Complex::<f64>::zero();
    assert!(zero_c.signum().is_nan());
}

#[test]
fn test_float_is_sign_positive() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(-1.0, 2.0);
    let c3 = Complex::new(1.0, -2.0);
    assert!(c1.is_sign_positive());
    assert!(!c2.is_sign_positive());
    assert!(!c3.is_sign_positive());
}

#[test]
fn test_float_is_sign_negative() {
    let c1 = Complex::new(-1.0, -2.0);
    let c2 = Complex::new(1.0, -2.0);
    let c3 = Complex::new(-1.0, 2.0);
    assert!(c1.is_sign_negative());
    assert!(!c2.is_sign_negative());
    assert!(!c3.is_sign_negative());
}

#[test]
fn test_float_mul_add() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(3.0, 4.0);
    let c3 = Complex::new(5.0, 6.0);
    let result = c1.mul_add(c2, c3);
    // (1+2i)*(3+4i) + (5+6i) = (-5+10i) + (5+6i) = 0+16i
    assert_complex_approx_eq(result, Complex::new(0.0, 16.0), 1e-9);
}

#[test]
fn test_float_recip() {
    let c = Complex::new(1.0, 2.0);
    let recip_c = c.recip();
    // 1/(1+2i) = (1-2i)/(1+4) = 1/5 - 2/5 i
    assert_complex_approx_eq(recip_c, Complex::new(0.2, -0.4), 1e-9);
}

#[test]
fn test_float_powi() {
    let c = Complex::new(1.0, 1.0);
    let c_sq = c.powi(2); // (1+i)^2 = 1 + 2i - 1 = 2i
    assert_complex_approx_eq(c_sq, Complex::new(0.0, 2.0), 1e-9);

    let c_neg = c.powi(-1); // 1/(1+i) = (1-i)/2 = 0.5 - 0.5i
    assert_complex_approx_eq(c_neg, Complex::new(0.5, -0.5), 1e-9);

    let c_zero = c.powi(0);
    assert_complex_approx_eq(c_zero, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_powf() {
    let c = Complex::new(E, 0.0); // e
    let n = Complex::new(0.0, PI / 2.0); // i * pi/2
    let result = c.powf(n); // e^(i*pi/2) = cos(pi/2) + i*sin(pi/2) = i
    assert_complex_approx_eq(result, Complex::new(0.0, 1.0), 1e-9);
}

#[test]
fn test_float_sqrt() {
    let c = Complex::new(0.0, 2.0); // 2i
    let sqrt_c = c.sqrt(); // sqrt(2i) = 1+i
    assert_complex_approx_eq(sqrt_c, Complex::new(1.0, 1.0), 1e-9);
}

#[test]
fn test_float_exp() {
    let c = Complex::new(0.0, PI); // i*pi
    let exp_c = c.exp(); // e^(i*pi) = cos(pi) + i*sin(pi) = -1
    assert_complex_approx_eq(exp_c, Complex::new(-1.0, 0.0), 1e-9);
}

#[test]
fn test_float_exp2() {
    let c = Complex::new(2.0, 0.0);
    let exp2_c = c.exp2(); // 2^2 = 4
    assert_complex_approx_eq(exp2_c, Complex::new(4.0, 0.0), 1e-9);
}

#[test]
fn test_float_ln() {
    let c = Complex::new(-1.0, 0.0); // -1
    let ln_c = c.ln(); // ln(-1) = i*pi
    assert_complex_approx_eq(ln_c, Complex::new(0.0, PI), 1e-9);
}

#[test]
fn test_float_log() {
    let c = Complex::new(E, 0.0);
    let base = Complex::new(E, 0.0);
    let log_c = c.log(base);
    assert_complex_approx_eq(log_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_log2() {
    let c = Complex::new(4.0, 0.0);
    let log2_c = c.log2();
    assert_complex_approx_eq(log2_c, Complex::new(2.0, 0.0), 1e-9);
}

#[test]
fn test_float_log10() {
    let c = Complex::new(100.0, 0.0);
    let log10_c = c.log10();
    assert_complex_approx_eq(log10_c, Complex::new(2.0, 0.0), 1e-9);
}

#[test]
fn test_float_cbrt() {
    let c = Complex::new(8.0, 0.0);
    let cbrt_c = c.cbrt();
    assert_complex_approx_eq(cbrt_c, Complex::new(2.0, 0.0), 1e-9);
}

#[test]
fn test_float_hypot() {
    let c1 = Complex::new(3.0, 0.0);
    let c2 = Complex::new(4.0, 0.0);
    let hypot_c = c1.hypot(c2);
    assert_complex_approx_eq(hypot_c, Complex::new(5.0, 0.0), 1e-9);
}

#[test]
fn test_float_sin() {
    let c = Complex::new(PI / 2.0, 0.0);
    let sin_c = c.sin();
    assert_complex_approx_eq(sin_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_cos() {
    let c = Complex::new(PI, 0.0);
    let cos_c = c.cos();
    assert_complex_approx_eq(cos_c, Complex::new(-1.0, 0.0), 1e-9);
}

#[test]
fn test_float_tan() {
    let c = Complex::new(PI / 4.0, 0.0);
    let tan_c = c.tan();
    assert_complex_approx_eq(tan_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_asin() {
    let c = Complex::new(1.0, 0.0);
    let asin_c = c.asin();
    assert_complex_approx_eq(asin_c, Complex::new(PI / 2.0, 0.0), 1e-9);
}

#[test]
fn test_float_acos() {
    let c = Complex::new(0.0, 0.0);
    let acos_c = c.acos();
    assert_complex_approx_eq(acos_c, Complex::new(PI / 2.0, 0.0), 1e-9);
}

#[test]
fn test_float_atan() {
    let c = Complex::new(1.0, 0.0);
    let atan_c = c.atan();
    assert_complex_approx_eq(atan_c, Complex::new(PI / 4.0, 0.0), 1e-9);
}

#[test]
fn test_float_atan2() {
    let c1 = Complex::new(1.0, 0.0); // y
    let c2 = Complex::new(1.0, 0.0); // x
    let atan2_c = c1.atan2(c2);
    assert_approx_eq(atan2_c.re(), PI / 4.0, 1e-9);
    assert_eq!(atan2_c.im(), 0.0);
}

#[test]
fn test_float_exp_m1() {
    let c = Complex::new(0.0, 0.0);
    let exp_m1_c = c.exp_m1();
    assert_complex_approx_eq(exp_m1_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_ln_1p() {
    let c = Complex::new(0.0, 0.0);
    let ln_1p_c = c.ln_1p();
    assert_complex_approx_eq(ln_1p_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_sinh() {
    let c = Complex::new(0.0, 0.0);
    let sinh_c = c.sinh();
    assert_complex_approx_eq(sinh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_cosh() {
    let c = Complex::new(0.0, 0.0);
    let cosh_c = c.cosh();
    assert_complex_approx_eq(cosh_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_float_tanh() {
    let c = Complex::new(0.0, 0.0);
    let tanh_c = c.tanh();
    assert_complex_approx_eq(tanh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_asinh() {
    let c = Complex::new(0.0, 0.0);
    let asinh_c = c.asinh();
    assert_complex_approx_eq(asinh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_acosh() {
    let c = Complex::new(1.0, 0.0);
    let acosh_c = c.acosh();
    assert_complex_approx_eq(acosh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_atanh() {
    let c = Complex::new(0.0, 0.0);
    let atanh_c = c.atanh();
    assert_complex_approx_eq(atanh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_float_to_degrees() {
    let c = Complex::new(PI, PI / 2.0);
    let deg_c = c.to_degrees();
    assert_complex_approx_eq(deg_c, Complex::new(180.0, 90.0), 1e-9);
}

#[test]
fn test_float_to_radians() {
    let c = Complex::new(180.0, 90.0);
    let rad_c = c.to_radians();
    assert_complex_approx_eq(rad_c, Complex::new(PI, PI / 2.0), 1e-9);
}

#[test]
fn test_float_max() {
    let c1 = Complex::new(3.0, 4.0); // norm 5
    let c2 = Complex::new(1.0, 1.0); // norm sqrt(2)
    let max_c = c1.max(c2);
    assert_complex_approx_eq(max_c, c1, 1e-9);
}

#[test]
fn test_float_min() {
    let c1 = Complex::new(3.0, 4.0); // norm 5
    let c2 = Complex::new(1.0, 1.0); // norm sqrt(2)
    let min_c = c1.min(c2);
    assert_complex_approx_eq(min_c, c2, 1e-9);
}

#[test]
fn test_float_clamp() {
    let c = Complex::new(2.0, 2.0); // norm sqrt(8) approx 2.82
    let min_c = Complex::new(1.0, 0.0); // norm 1
    let max_c = Complex::new(3.0, 0.0); // norm 3
    let clamped_c = c.clamp(min_c, max_c);
    assert_complex_approx_eq(clamped_c, c, 1e-9);

    let c_low = Complex::new(0.5, 0.0);
    let clamped_low = c_low.clamp(min_c, max_c);
    assert_complex_approx_eq(clamped_low, min_c, 1e-9);

    let c_high = Complex::new(4.0, 0.0);
    let clamped_high = c_high.clamp(min_c, max_c);
    assert_complex_approx_eq(clamped_high, max_c, 1e-9);
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
    assert_complex_approx_eq(s, Complex::new((PI / 4.0).sin(), 0.0), 1e-9);
    assert_complex_approx_eq(co, Complex::new((PI / 4.0).cos(), 0.0), 1e-9);
}

#[test]
fn test_float_copysign() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(-3.0, 4.0);
    let result = c1.copysign(c2);
    assert_eq!(result.re(), -1.0);
    assert_eq!(result.im(), 2.0);
}

// Conversion trait tests
#[test]
fn test_to_primitive_f64() {
    let c = Complex::new(1.5, 2.5);
    assert_eq!(c.to_f64(), Some(1.5));
}

#[test]
fn test_from_primitive_f64() {
    let c = Complex::<f64>::from_f64(1.5).unwrap();
    assert_eq!(c.re(), 1.5);
    assert_eq!(c.im(), 0.0);
}

#[test]
fn test_as_primitive_f64() {
    let c = Complex::new(1.5, 2.5);
    let val: f64 = c.as_();
    assert_eq!(val, 1.5);
}

#[test]
fn test_num_cast_f64() {
    let c = <Complex<f64> as deep_causality_num::NumCast>::from(1.5).unwrap();
    assert_eq!(c.re(), 1.5);
    assert_eq!(c.im(), 0.0);
}
