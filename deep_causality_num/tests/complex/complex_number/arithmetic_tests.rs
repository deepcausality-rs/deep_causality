/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use core::ops::Div;
use deep_causality_num::utils_tests::utils_complex_tests;
use deep_causality_num::{Complex, Zero};

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

    let quot = c1.div(c2);
    assert!(quot.re().is_nan());
    assert!(quot.im().is_nan());
}

#[test]
fn test_complex_sum() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(3.0, 4.0);
    let c3 = Complex::new(5.0, 6.0);
    let v = vec![c1, c2, c3];
    let sum: Complex<f64> = v.into_iter().sum();
    assert_eq!(sum.re(), 9.0);
    assert_eq!(sum.im(), 12.0);
}

#[test]
fn test_complex_product() {
    let c1 = Complex::new(1.0, 2.0);
    let c2 = Complex::new(3.0, 4.0);
    let c3 = Complex::new(5.0, 6.0);
    let v = vec![c1, c2, c3];
    let product: Complex<f64> = v.into_iter().product();
    // (1+2i)(3+4i) = 3 + 4i + 6i - 8 = -5 + 10i
    // (-5+10i)(5+6i) = -25 - 30i + 50i - 60 = -85 + 20i
    assert_eq!(product.re(), -85.0);
    assert_eq!(product.im(), 20.0);
}

#[test]
fn test_complex_sum_empty() {
    let v: Vec<Complex<f64>> = Vec::new();
    let sum: Complex<f64> = v.into_iter().sum();
    assert_eq!(sum.re(), 0.0);
    assert_eq!(sum.im(), 0.0);
}

#[test]
fn test_complex_product_empty() {
    let v: Vec<Complex<f64>> = Vec::new();
    let product: Complex<f64> = v.into_iter().product();
    assert_eq!(product.re(), 1.0);
    assert_eq!(product.im(), 0.0);
}
