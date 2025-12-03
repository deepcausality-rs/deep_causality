/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::utils_tests::utils_complex_tests;
use deep_causality_num::{Complex, RealField};

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
    utils_complex_tests::assert_complex_approx_eq(c, Complex::new(3.0, 4.0), 1e-9);
}

#[test]
fn test_complex_div_assign_scalar() {
    let mut c = Complex::new(3.0, 6.0);
    c /= 3.0;
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_div_assign_nan_re_divisor() {
    let mut c = Complex::new(1.0, 2.0);
    let rhs = Complex::new(f64::nan(), 1.0);
    c /= rhs;
    assert!(c.re().is_nan());
    assert!(c.im().is_nan());
}

#[test]
fn test_complex_div_assign_nan_im_divisor() {
    let mut c = Complex::new(1.0, 2.0);
    let rhs = Complex::new(1.0, f64::nan());
    c /= rhs;
    assert!(c.re().is_nan());
    assert!(c.im().is_nan());
}

#[test]
fn test_complex_div_assign_nan_both_divisor() {
    let mut c = Complex::new(1.0, 2.0);
    let rhs = Complex::new(f64::nan(), f64::nan());
    c /= rhs;
    assert!(c.re().is_nan());
    assert!(c.im().is_nan());
}

#[test]
fn test_complex_div_assign_scalar_nan() {
    let mut c = Complex::new(1.0, 2.0);
    let scalar = f64::nan();
    c /= scalar;
    assert!(c.re().is_nan());
    assert!(c.im().is_nan());
}
