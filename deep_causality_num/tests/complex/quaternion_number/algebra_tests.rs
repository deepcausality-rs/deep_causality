/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Zero;
use deep_causality_num::{DivisionAlgebra, Quaternion};

// Helper for float comparison
const EPSILON: f64 = 1e-9;

// Test cases for MulGroup::inverse and DivisionAlgebra::inverse
// These trait methods delegate to Quaternion<T>::inverse()
#[test]
fn test_mul_group_inverse_general() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let inverse_q: Quaternion<f64> = q.inverse();
    // Conjugate is (1, -2, -3, -4), norm_sqr is 1+4+9+16 = 30
    // Expected: (1/30, -2/30, -3/30, -4/30) = (0.0333..., -0.0666..., -0.1, -0.1333...)
    let expected_inverse = Quaternion::new(1.0 / 30.0, -2.0 / 30.0, -3.0 / 30.0, -4.0 / 30.0);

    assert!((inverse_q.w - expected_inverse.w).abs() < EPSILON);
    assert!((inverse_q.x - expected_inverse.x).abs() < EPSILON);
    assert!((inverse_q.y - expected_inverse.y).abs() < EPSILON);
    assert!((inverse_q.z - expected_inverse.z).abs() < EPSILON);
}

#[test]
fn test_mul_group_inverse_purely_scalar() {
    let q = Quaternion::from_real(5.0);
    let inverse_q: Quaternion<f64> = q.inverse();
    // Conjugate is (5, 0, 0, 0), norm_sqr is 25
    // Expected: (5/25, 0, 0, 0) = (0.2, 0, 0, 0)
    let expected_inverse = Quaternion::from_real(0.2);

    assert!((inverse_q.w - expected_inverse.w).abs() < EPSILON);
    assert!((inverse_q.x - expected_inverse.x).abs() < EPSILON);
    assert!((inverse_q.y - expected_inverse.y).abs() < EPSILON);
    assert!((inverse_q.z - expected_inverse.z).abs() < EPSILON);
}

#[test]
fn test_mul_group_inverse_purely_vector() {
    let q = Quaternion::new(0.0, 2.0, 0.0, 0.0); // 2i
    let inverse_q: Quaternion<f64> = q.inverse();
    // Conjugate is (0, -2, 0, 0), norm_sqr is 4
    // Expected: (0, -2/4, 0, 0) = (0, -0.5, 0, 0)
    let expected_inverse = Quaternion::new(0.0, -0.5, 0.0, 0.0);

    assert!((inverse_q.w - expected_inverse.w).abs() < EPSILON);
    assert!((inverse_q.x - expected_inverse.x).abs() < EPSILON);
    assert!((inverse_q.y - expected_inverse.y).abs() < EPSILON);
    assert!((inverse_q.z - expected_inverse.z).abs() < EPSILON);
}

#[test]
fn test_mul_group_inverse_zero() {
    let q = Quaternion::<f64>::zero();
    let inverse_q: Quaternion<f64> = q.inverse();
    assert!(inverse_q.w.is_nan());
    assert!(inverse_q.x.is_nan());
    assert!(inverse_q.y.is_nan());
    assert!(inverse_q.z.is_nan());
}

// Test cases for DivisionAlgebra::conjugate
#[test]
fn test_division_algebra_conjugate_general() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let expected_conjugate = Quaternion::new(1.0, -2.0, -3.0, -4.0);
    let actual_conjugate: Quaternion<f64> = q.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_purely_scalar() {
    let q = Quaternion::from_real(5.0);
    let expected_conjugate = Quaternion::from_real(5.0);
    let actual_conjugate: Quaternion<f64> = q.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_purely_vector() {
    let q = Quaternion::new(0.0, 1.0, 2.0, 3.0);
    let expected_conjugate = Quaternion::new(0.0, -1.0, -2.0, -3.0);
    let actual_conjugate: Quaternion<f64> = q.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

#[test]
fn test_division_algebra_conjugate_zero() {
    let q = Quaternion::<f64>::zero();
    let expected_conjugate = Quaternion::<f64>::zero();
    let actual_conjugate: Quaternion<f64> = q.conjugate();
    assert_eq!(actual_conjugate, expected_conjugate);
}

// Test cases for DivisionAlgebra::norm_sqr
#[test]
fn test_division_algebra_norm_sqr_general() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let expected_norm_sqr = 1.0 + 4.0 + 9.0 + 16.0; // 30.0
    let actual_norm_sqr: f64 = q.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_purely_scalar() {
    let q = Quaternion::from_real(5.0);
    let expected_norm_sqr = 25.0; // 5*5
    let actual_norm_sqr: f64 = q.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_purely_vector() {
    let q = Quaternion::new(0.0, 2.0, 3.0, 0.0);
    let expected_norm_sqr = 4.0 + 9.0; // 13.0
    let actual_norm_sqr: f64 = q.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}

#[test]
fn test_division_algebra_norm_sqr_zero() {
    let q = Quaternion::<f64>::zero();
    let expected_norm_sqr = 0.0;
    let actual_norm_sqr: f64 = q.norm_sqr();
    assert!((actual_norm_sqr - expected_norm_sqr).abs() < EPSILON);
}
