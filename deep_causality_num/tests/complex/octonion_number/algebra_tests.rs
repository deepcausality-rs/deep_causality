/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Zero;
use deep_causality_num::{DivisionAlgebra, Octonion, Octonion64};

// Helper for float comparison
const EPSILON: f64 = 1e-9;

// Test cases for DivisionAlgebra::conjugate
#[test]
fn test_division_algebra_conjugate_general() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    let expected_conjugate = Octonion::new(1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0, -8.0);
    let actual_conjugate: Octonion<f64> = o.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_purely_scalar() {
    let o = Octonion::from_real(5.0);
    let expected_conjugate = Octonion::from_real(5.0);
    let actual_conjugate: Octonion<f64> = o.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_purely_imaginary() {
    let o = Octonion::new(0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
    let expected_conjugate = Octonion::new(0.0, -1.0, -2.0, -3.0, -4.0, -5.0, -6.0, -7.0);
    let actual_conjugate: Octonion<f64> = o.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_zero() {
    let o = Octonion::<f64>::zero();
    let expected_conjugate = Octonion::<f64>::zero();
    let actual_conjugate: Octonion<f64> = o.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

// Test cases for DivisionAlgebra::norm_sqr
#[test]
fn test_division_algebra_norm_sqr_general() {
    let o = Octonion::new(1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
    let expected_norm_sqr = 8.0; // 8 * (1*1)
    let actual_norm_sqr: f64 = o.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_purely_scalar() {
    let o = Octonion::from_real(3.0);
    let expected_norm_sqr = 9.0; // 3*3
    let actual_norm_sqr: f64 = o.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_purely_imaginary() {
    let o = Octonion::new(0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let expected_norm_sqr = 4.0; // 2*2
    let actual_norm_sqr: f64 = o.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_zero() {
    let o = Octonion::<f64>::zero();
    let expected_norm_sqr = 0.0;
    let actual_norm_sqr: f64 = o.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

// Test cases for DivisionAlgebra::inverse
#[test]
fn test_division_algebra_inverse_general() {
    use deep_causality_num::DivisionAlgebra;

    let o = Octonion64::new(1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 1 + e1
    let inverse_o = o.inverse();
    // conjugate is (1 - e1), norm_sqr is 2.0
    // Expected: 0.5 - 0.5e1
    let expected_inverse = Octonion::new(0.5, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);

    assert!((inverse_o.s - expected_inverse.s).abs() < EPSILON);
    assert!((inverse_o.e1 - expected_inverse.e1).abs() < EPSILON);
    assert!((inverse_o.e2 - expected_inverse.e2).abs() < EPSILON);
    assert!((inverse_o.e3 - expected_inverse.e3).abs() < EPSILON);
    assert!((inverse_o.e4 - expected_inverse.e4).abs() < EPSILON);
    assert!((inverse_o.e5 - expected_inverse.e5).abs() < EPSILON);
    assert!((inverse_o.e6 - expected_inverse.e6).abs() < EPSILON);
    assert!((inverse_o.e7 - expected_inverse.e7).abs() < EPSILON);
}

#[test]
fn test_division_algebra_inverse_purely_scalar() {
    let o = Octonion::from_real(4.0);
    let inverse_o: Octonion<f64> = o.inverse();
    // conjugate is 4.0, norm_sqr is 16.0
    // Expected: 0.25
    let expected_inverse = Octonion::from_real(0.25);

    assert!((inverse_o.s - expected_inverse.s).abs() < EPSILON);
    assert!((inverse_o.e1 - expected_inverse.e1).abs() < EPSILON);
    assert!((inverse_o.e2 - expected_inverse.e2).abs() < EPSILON);
    assert!((inverse_o.e3 - expected_inverse.e3).abs() < EPSILON);
    assert!((inverse_o.e4 - expected_inverse.e4).abs() < EPSILON);
    assert!((inverse_o.e5 - expected_inverse.e5).abs() < EPSILON);
    assert!((inverse_o.e6 - expected_inverse.e6).abs() < EPSILON);
    assert!((inverse_o.e7 - expected_inverse.e7).abs() < EPSILON);
}

#[test]
fn test_division_algebra_inverse_purely_imaginary() {
    let o = Octonion::new(0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0); // 2e2
    let inverse_o: Octonion<f64> = o.inverse();
    // conjugate is -2e2, norm_sqr is 4.0
    // Expected: -0.5e2
    let expected_inverse = Octonion::new(0.0, 0.0, -0.5, 0.0, 0.0, 0.0, 0.0, 0.0);

    assert!((inverse_o.s - expected_inverse.s).abs() < EPSILON);
    assert!((inverse_o.e1 - expected_inverse.e1).abs() < EPSILON);
    assert!((inverse_o.e2 - expected_inverse.e2).abs() < EPSILON);
    assert!((inverse_o.e3 - expected_inverse.e3).abs() < EPSILON);
    assert!((inverse_o.e4 - expected_inverse.e4).abs() < EPSILON);
    assert!((inverse_o.e5 - expected_inverse.e5).abs() < EPSILON);
    assert!((inverse_o.e6 - expected_inverse.e6).abs() < EPSILON);
    assert!((inverse_o.e7 - expected_inverse.e7).abs() < EPSILON);
}

#[test]
fn test_division_algebra_inverse_zero() {
    let o = Octonion::<f64>::zero();
    let inverse_o: Octonion<f64> = o.inverse();
    assert!(inverse_o.s.is_nan());
    assert!(inverse_o.e1.is_nan());
    assert!(inverse_o.e2.is_nan());
    assert!(inverse_o.e3.is_nan());
    assert!(inverse_o.e4.is_nan());
    assert!(inverse_o.e5.is_nan());
    assert!(inverse_o.e6.is_nan());
    assert!(inverse_o.e7.is_nan());
}
