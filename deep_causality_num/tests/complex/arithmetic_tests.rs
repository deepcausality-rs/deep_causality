/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::utils_tests::utils_complex_tests;
use deep_causality_num::{Complex, ComplexNumber, Zero};

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
    utils_complex_tests::assert_complex_approx_eq(quot, Complex::new(3.0, 4.0), 1e-9);
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
