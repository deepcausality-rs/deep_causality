/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for DivisionAlgebra and RealField traits on DoubleFloat.

use deep_causality_num::{DivisionAlgebra, DoubleFloat, Float, RealField};

const EPSILON: f64 = 1e-14;

// =============================================================================
// Zero and One (via RealField)
// =============================================================================

#[test]
fn test_realfield_nan() {
    let nan = <DoubleFloat as RealField>::nan();
    assert!(nan.is_nan());
}

// =============================================================================
// Clamp Tests (all branches)
// =============================================================================

#[test]
fn test_clamp_in_range() {
    let x = DoubleFloat::from_f64(5.0);
    let min = DoubleFloat::from_f64(0.0);
    let max = DoubleFloat::from_f64(10.0);
    let result = RealField::clamp(x, min, max);
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

#[test]
fn test_clamp_below_min() {
    let x = DoubleFloat::from_f64(-5.0);
    let min = DoubleFloat::from_f64(0.0);
    let max = DoubleFloat::from_f64(10.0);
    let result = RealField::clamp(x, min, max);
    assert!((result.hi() - 0.0).abs() < EPSILON);
}

#[test]
fn test_clamp_above_max() {
    let x = DoubleFloat::from_f64(15.0);
    let min = DoubleFloat::from_f64(0.0);
    let max = DoubleFloat::from_f64(10.0);
    let result = RealField::clamp(x, min, max);
    assert!((result.hi() - 10.0).abs() < EPSILON);
}

#[test]
fn test_clamp_at_min() {
    let x = DoubleFloat::from_f64(0.0);
    let min = DoubleFloat::from_f64(0.0);
    let max = DoubleFloat::from_f64(10.0);
    let result = RealField::clamp(x, min, max);
    assert!((result.hi() - 0.0).abs() < EPSILON);
}

#[test]
fn test_clamp_at_max() {
    let x = DoubleFloat::from_f64(10.0);
    let min = DoubleFloat::from_f64(0.0);
    let max = DoubleFloat::from_f64(10.0);
    let result = RealField::clamp(x, min, max);
    assert!((result.hi() - 10.0).abs() < EPSILON);
}

// =============================================================================
// RealField Mathematical Functions
// =============================================================================

#[test]
fn test_realfield_abs() {
    let x = DoubleFloat::from_f64(-42.0);
    let result = RealField::abs(x);
    assert!((result.hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_realfield_sqrt() {
    let x = DoubleFloat::from_f64(4.0);
    let result = RealField::sqrt(x);
    assert!((result.hi() - 2.0).abs() < EPSILON);
}

#[test]
fn test_realfield_floor() {
    let x = DoubleFloat::from_f64(3.7);
    let result = RealField::floor(x);
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_realfield_ceil() {
    let x = DoubleFloat::from_f64(3.3);
    let result = RealField::ceil(x);
    assert!((result.hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_realfield_round() {
    let x = DoubleFloat::from_f64(3.5);
    let result = RealField::round(x);
    assert!((result.hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_realfield_exp() {
    let x = DoubleFloat::from_f64(1.0);
    let result = RealField::exp(x);
    assert!((result.hi() - std::f64::consts::E).abs() < 1e-10);
}

#[test]
fn test_realfield_ln() {
    let x = DoubleFloat::from_f64(std::f64::consts::E);
    let result = RealField::ln(x);
    assert!((result.hi() - 1.0).abs() < 1e-10);
}

#[test]
fn test_realfield_log() {
    let x = DoubleFloat::from_f64(100.0);
    let base = DoubleFloat::from_f64(10.0);
    let result = RealField::log(x, base);
    assert!((result.hi() - 2.0).abs() < 1e-10);
}

#[test]
fn test_realfield_powf() {
    let x = DoubleFloat::from_f64(2.0);
    let n = DoubleFloat::from_f64(3.0);
    let result = RealField::powf(x, n);
    assert!((result.hi() - 8.0).abs() < 1e-10);
}

// =============================================================================
// RealField Trigonometric Functions
// =============================================================================

#[test]
fn test_realfield_sin() {
    let x = DoubleFloat::from_f64(0.0);
    let result = RealField::sin(x);
    assert!(result.hi().abs() < EPSILON);
}

#[test]
fn test_realfield_cos() {
    let x = DoubleFloat::from_f64(0.0);
    let result = RealField::cos(x);
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_realfield_tan() {
    let x = DoubleFloat::from_f64(0.0);
    let result = RealField::tan(x);
    assert!(result.hi().abs() < EPSILON);
}

#[test]
fn test_realfield_acos() {
    let x = DoubleFloat::from_f64(1.0);
    let result = RealField::acos(x);
    assert!(result.hi().abs() < EPSILON);
}

// =============================================================================
// RealField Hyperbolic Functions
// =============================================================================

#[test]
fn test_realfield_sinh() {
    let x = DoubleFloat::from_f64(0.0);
    let result = RealField::sinh(x);
    assert!(result.hi().abs() < EPSILON);
}

#[test]
fn test_realfield_cosh() {
    let x = DoubleFloat::from_f64(0.0);
    let result = RealField::cosh(x);
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_realfield_tanh() {
    let x = DoubleFloat::from_f64(0.0);
    let result = RealField::tanh(x);
    assert!(result.hi().abs() < EPSILON);
}

// =============================================================================
// RealField atan2
// =============================================================================

#[test]
fn test_realfield_atan2() {
    let y = DoubleFloat::from_f64(1.0);
    let x = DoubleFloat::from_f64(1.0);
    let result = RealField::atan2(y, x);
    assert!((result.hi() - std::f64::consts::PI / 4.0).abs() < 1e-10);
}

// =============================================================================
// RealField Constants
// =============================================================================

#[test]
fn test_realfield_pi() {
    let pi = <DoubleFloat as RealField>::pi();
    assert!((pi.hi() - std::f64::consts::PI).abs() < EPSILON);
}

#[test]
fn test_realfield_e() {
    let e = <DoubleFloat as RealField>::e();
    assert!((e.hi() - std::f64::consts::E).abs() < EPSILON);
}

#[test]
fn test_realfield_epsilon() {
    let eps = <DoubleFloat as RealField>::epsilon();
    assert!(eps.hi() > 0.0);
    assert!(eps.hi() < 1e-30);
}

// =============================================================================
// DivisionAlgebra Tests
// =============================================================================

#[test]
fn test_divisionalgebra_conjugate() {
    let x = DoubleFloat::from_f64(5.0);
    // For reals, conjugate is identity
    let result = DivisionAlgebra::conjugate(&x);
    assert_eq!(result, x);
}

#[test]
fn test_divisionalgebra_norm_sqr() {
    let x = DoubleFloat::from_f64(3.0);
    let result = DivisionAlgebra::norm_sqr(&x);
    assert!((result.hi() - 9.0).abs() < EPSILON);
}

#[test]
fn test_divisionalgebra_inverse() {
    let x = DoubleFloat::from_f64(4.0);
    let result = DivisionAlgebra::inverse(&x);
    assert!((result.hi() - 0.25).abs() < EPSILON);
}

#[test]
fn test_divisionalgebra_inverse_identity() {
    let x = DoubleFloat::from_f64(4.0);
    let inv = DivisionAlgebra::inverse(&x);
    let product = x * inv;
    assert!((product.hi() - 1.0).abs() < EPSILON);
}
