/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, Complex32, Complex64};

// Helper for float comparison
const EPSILON: f64 = 1e-9;

// Test cases for MulGroup::inverse and DivisionAlgebra::inverse
// These trait methods delegate to Complex<T>::inverse()
#[test]
fn test_mul_group_inverse_regular() {
    let z = Complex64::new(1.0, 2.0);
    let expected_inverse = Complex::new(0.2, -0.4); // 1/(1+2i) = (1-2i)/(1+4) = 0.2 - 0.4i
    let actual_inverse = z.inverse();
    assert!((actual_inverse.re - expected_inverse.re).abs() < EPSILON);
    assert!((actual_inverse.im - expected_inverse.im).abs() < EPSILON);
}

#[test]
fn test_mul_group_inverse_purely_real() {
    let z = Complex64::new(5.0, 0.0);
    let expected_inverse = Complex::new(0.2, 0.0); // 1/5 = 0.2
    let actual_inverse = z.inverse();
    assert!((actual_inverse.re - expected_inverse.re).abs() < EPSILON);
    assert!((actual_inverse.im - expected_inverse.im).abs() < EPSILON);
}

#[test]
fn test_mul_group_inverse_purely_imaginary() {
    let z = Complex64::new(0.0, 2.0);
    let expected_inverse = Complex::new(0.0, -0.5); // 1/(2i) = -i/2 = 0 - 0.5i
    let actual_inverse = z.inverse();
    assert!((actual_inverse.re - expected_inverse.re).abs() < EPSILON);
    assert!((actual_inverse.im - expected_inverse.im).abs() < EPSILON);
}

#[test]
fn test_mul_group_inverse_zero() {
    let z = Complex32::new(0.0, 0.0);
    let actual_inverse = z.inverse();
    assert!(actual_inverse.re.is_nan());
    assert!(actual_inverse.im.is_nan());
}

// Test cases for DivisionAlgebra::conjugate
#[test]
fn test_division_algebra_conjugate_regular() {
    let z = Complex::new(1.0, 2.0);
    let expected_conjugate = Complex::new(1.0, -2.0);
    let actual_conjugate = z.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_purely_real() {
    let z = Complex::new(5.0, 0.0);
    let expected_conjugate = Complex::new(5.0, 0.0);
    let actual_conjugate = z.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_purely_imaginary() {
    let z = Complex::new(0.0, 3.0);
    let expected_conjugate = Complex::new(0.0, -3.0);
    let actual_conjugate = z.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_zero() {
    let z = Complex::new(0.0, 0.0);
    let expected_conjugate = Complex::new(0.0, 0.0);
    let actual_conjugate = z.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

// Test cases for DivisionAlgebra::norm_sqr
#[test]
fn test_division_algebra_norm_sqr_regular() {
    let z = Complex64::new(3.0, 4.0);
    let expected_norm_sqr = 25.0; // 3*3 + 4*4 = 9 + 16 = 25
    let actual_norm_sqr = z.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_purely_real() {
    let z = Complex64::new(5.0, 0.0);
    let expected_norm_sqr = 25.0; // 5*5 + 0*0 = 25
    let actual_norm_sqr = z.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_purely_imaginary() {
    let z = Complex64::new(0.0, 3.0);
    let expected_norm_sqr = 9.0; // 0*0 + 3*3 = 9
    let actual_norm_sqr = z.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_zero() {
    let z = Complex64::new(0.0, 0.0);
    let expected_norm_sqr = 0.0; // 0*0 + 0*0 = 0
    let actual_norm_sqr = z.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}
