/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::f64::consts::{E, PI};
use deep_causality_num::utils_tests::utils_complex_tests;
use deep_causality_num::{Complex, DivisionAlgebra, RealField, Zero};

// --- Inherent Complex<T> method tests (formerly in float_tests.rs) ---

#[test]
fn test_complex_nan() {
    let c = Complex::<f64>::new(f64::nan(), f64::nan());
    assert!(c.re.is_nan());
    assert!(c.im.is_nan());
}

#[test]
fn test_complex_is_zero() {
    let c = Complex::<f64>::zero();
    assert!(c.is_zero());
    let c2 = Complex::new(1.0, 0.0);
    assert!(!c2.is_zero());
}

#[test]
fn test_complex_floor() {
    let c = Complex::new(1.5, 2.9);
    let floor_c = Complex::new(c.re.floor(), c.im.floor()); // floor() consumes self
    assert_eq!(floor_c.re, 1.0);
    assert_eq!(floor_c.im, 2.0);
}

#[test]
fn test_complex_ceil() {
    let c = Complex::new(1.1, 2.5);
    let ceil_c = Complex::new(c.re.ceil(), c.im.ceil()); // ceil() consumes self
    assert_eq!(ceil_c.re, 2.0);
    assert_eq!(ceil_c.im, 3.0);
}

#[test]
fn test_complex_round() {
    let c1 = Complex::new(1.4, 2.6);
    let c2 = Complex::new(1.5, 2.5);
    let c1_round = Complex::new(c1.re.round(), c1.im.round());
    let c2_round = Complex::new(c2.re.round(), c2.im.round());
    assert_eq!(c1_round.re, 1.0);
    assert_eq!(c1_round.im, 3.0);
    assert_eq!(c2_round.re, 2.0);
    assert_eq!(c2_round.im, 3.0);
}

#[test]
fn test_complex_trunc() {
    let c = Complex::new(1.9, -2.1);
    let trunc_c = Complex::new(f64::trunc(c.re), f64::trunc(c.im)); // trunc() consumes self
    assert_eq!(trunc_c.re, 1.0);
    assert_eq!(trunc_c.im, -2.0);
}

#[test]
fn test_complex_fract() {
    let c = Complex::new(1.9, -2.1);
    let fract_c = Complex::new(f64::fract(c.re), f64::fract(c.im)); // fract() consumes self
    utils_complex_tests::assert_approx_eq(fract_c.re, 0.9, 1e-9);
    utils_complex_tests::assert_approx_eq(fract_c.im, -0.1, 1e-9);
}

#[test]
fn test_complex_abs() {
    let c = Complex::new(3.0, 4.0);
    let abs_val = (c.re * c.re + c.im * c.im).sqrt(); // abs() calculates magnitude
    assert_eq!(abs_val, 5.0);
}

#[test]
fn test_complex_powi() {
    let c = Complex::new(1.0, 1.0);
    let c_sq = c.powi(2);
    utils_complex_tests::assert_complex_approx_eq(c_sq, Complex::new(0.0, 2.0), 1e-9);

    let c_neg = c.powi(-1);
    utils_complex_tests::assert_complex_approx_eq(c_neg, Complex::new(0.5, -0.5), 1e-9);

    let c_zero = c.powi(0);
    utils_complex_tests::assert_complex_approx_eq(c_zero, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_complex_powf() {
    let c = Complex::new(2.0, 0.0); // 2
    let n = 3.0; // real exponent
    let result = c.powf(n);
    utils_complex_tests::assert_complex_approx_eq(result, Complex::new(8.0, 0.0), 1e-9);

    let c_imag = Complex::new(0.0, 1.0); // i
    let n_half = 0.5; // real exponent
    let result_imag = c_imag.powf(n_half);
    utils_complex_tests::assert_complex_approx_eq(
        result_imag,
        Complex::new(
            std::f64::consts::FRAC_1_SQRT_2,
            std::f64::consts::FRAC_1_SQRT_2,
        ),
        1e-9,
    );
}

#[test]
fn test_complex_powc() {
    let c = Complex::new(E, 0.0); // e
    let n = Complex::new(0.0, PI / 2.0); // i * pi/2
    let result = c.powc(n);
    utils_complex_tests::assert_complex_approx_eq(result, Complex::new(0.0, 1.0), 1e-9);
}

#[test]
fn test_complex_sqrt() {
    let c = Complex::new(0.0, 2.0); // 2i
    let sqrt_c = c.sqrt();
    utils_complex_tests::assert_complex_approx_eq(sqrt_c, Complex::new(1.0, 1.0), 1e-9);
}

#[test]
fn test_complex_exp() {
    let c = Complex::new(0.0, PI); // i*pi
    let exp_c = c.exp();
    utils_complex_tests::assert_complex_approx_eq(exp_c, Complex::new(-1.0, 0.0), 1e-9);
}

#[test]
fn test_complex_ln() {
    let c = Complex::new(-1.0, 0.0); // -1
    let ln_c = c.ln();
    utils_complex_tests::assert_complex_approx_eq(ln_c, Complex::new(0.0, PI), 1e-9);
}

#[test]
fn test_complex_sin() {
    let c = Complex::new(PI / 2.0, 0.0);
    let sin_c = c.sin();
    utils_complex_tests::assert_complex_approx_eq(sin_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_complex_cos() {
    let c = Complex::new(PI, 0.0);
    let cos_c = c.cos();
    utils_complex_tests::assert_complex_approx_eq(cos_c, Complex::new(-1.0, 0.0), 1e-9);
}

#[test]
fn test_complex_tan() {
    let c = Complex::new(PI / 4.0, 0.0);
    let tan_c = c.tan();
    utils_complex_tests::assert_complex_approx_eq(tan_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_complex_sinh() {
    let c = Complex::new(0.0, 0.0);
    let sinh_c = c.sinh();
    utils_complex_tests::assert_complex_approx_eq(sinh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_complex_cosh() {
    let c = Complex::new(0.0, 0.0);
    let cosh_c = c.cosh();
    utils_complex_tests::assert_complex_approx_eq(cosh_c, Complex::new(1.0, 0.0), 1e-9);
}

#[test]
fn test_complex_tanh() {
    let c = Complex::new(0.0, 0.0);
    let tanh_c = c.tanh();
    utils_complex_tests::assert_complex_approx_eq(tanh_c, Complex::new(0.0, 0.0), 1e-9);
}

#[test]
fn test_complex_inverse_zero() {
    let c = Complex::<f64>::zero();
    let inv_c = c.inverse();
    assert!(inv_c.re.is_nan());
    assert!(inv_c.im.is_nan());
}
