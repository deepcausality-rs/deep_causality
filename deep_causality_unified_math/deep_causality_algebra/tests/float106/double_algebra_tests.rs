/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for DivisionAlgebra and Real traits on DoubleFloat.

use deep_causality_algebra::{DivisionAlgebra, Real};
use deep_causality_num::Float106;

const EPSILON: f64 = 1e-14;

// =============================================================================
// Zero and One (via Real)
// =============================================================================

#[test]
fn test_realfield_nan() {
    let nan = <Float106 as Real>::nan();
    assert!(nan.is_nan());
}

// =============================================================================
// Clamp Tests (all branches)
// =============================================================================

#[test]
fn test_clamp_in_range() {
    let x = Float106::from_f64(5.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Real::clamp(x, min, max);
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

#[test]
fn test_clamp_below_min() {
    let x = Float106::from_f64(-5.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Real::clamp(x, min, max);
    assert!((result.hi() - 0.0).abs() < EPSILON);
}

#[test]
fn test_clamp_above_max() {
    let x = Float106::from_f64(15.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Real::clamp(x, min, max);
    assert!((result.hi() - 10.0).abs() < EPSILON);
}

#[test]
fn test_clamp_at_min() {
    let x = Float106::from_f64(0.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Real::clamp(x, min, max);
    assert!((result.hi() - 0.0).abs() < EPSILON);
}

#[test]
fn test_clamp_at_max() {
    let x = Float106::from_f64(10.0);
    let min = Float106::from_f64(0.0);
    let max = Float106::from_f64(10.0);
    let result = Real::clamp(x, min, max);
    assert!((result.hi() - 10.0).abs() < EPSILON);
}

// =============================================================================
// Real Mathematical Functions
// =============================================================================

#[test]
fn test_realfield_abs() {
    let x = Float106::from_f64(-42.0);
    let result = Real::abs(x);
    assert!((result.hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_realfield_sqrt() {
    let x = Float106::from_f64(4.0);
    let result = Real::sqrt(x);
    assert!((result.hi() - 2.0).abs() < EPSILON);
}

#[test]
fn test_realfield_floor() {
    let x = Float106::from_f64(3.7);
    let result = Real::floor(x);
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_realfield_ceil() {
    let x = Float106::from_f64(3.3);
    let result = Real::ceil(x);
    assert!((result.hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_realfield_round() {
    let x = Float106::from_f64(3.5);
    let result = Real::round(x);
    assert!((result.hi() - 4.0).abs() < EPSILON);
}

#[test]
fn test_realfield_exp() {
    let x = Float106::from_f64(1.0);
    let result = Real::exp(x);
    assert!((result.hi() - std::f64::consts::E).abs() < 1e-10);
}

#[test]
fn test_realfield_ln() {
    let x = Float106::from_f64(std::f64::consts::E);
    let result = Real::ln(x);
    assert!((result.hi() - 1.0).abs() < 1e-10);
}

#[test]
fn test_realfield_log() {
    let x = Float106::from_f64(100.0);
    let base = Float106::from_f64(10.0);
    let result = Real::log(x, base);
    assert!((result.hi() - 2.0).abs() < 1e-10);
}

#[test]
fn test_realfield_log2() {
    let x = Float106::from_f64(8.0);
    let result = Real::log2(x);
    assert!((result.hi() - 3.0).abs() < 1e-10);
    let one = Real::log2(Float106::from_f64(1.0));
    assert!(one.hi().abs() < 1e-10);
}

#[test]
fn test_realfield_log10() {
    let x = Float106::from_f64(1000.0);
    let result = Real::log10(x);
    assert!((result.hi() - 3.0).abs() < 1e-10);
    let one = Real::log10(Float106::from_f64(1.0));
    assert!(one.hi().abs() < 1e-10);
}

#[test]
fn test_realfield_powf() {
    let x = Float106::from_f64(2.0);
    let n = Float106::from_f64(3.0);
    let result = Real::powf(x, n);
    assert!((result.hi() - 8.0).abs() < 1e-10);
}

// =============================================================================
// Real Trigonometric Functions
// =============================================================================

#[test]
fn test_realfield_sin() {
    let x = Float106::from_f64(0.0);
    let result = Real::sin(x);
    assert!(result.hi().abs() < EPSILON);
}

#[test]
fn test_realfield_cos() {
    let x = Float106::from_f64(0.0);
    let result = Real::cos(x);
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_realfield_tan() {
    let x = Float106::from_f64(0.0);
    let result = Real::tan(x);
    assert!(result.hi().abs() < EPSILON);
}

#[test]
fn test_realfield_acos() {
    let x = Float106::from_f64(1.0);
    let result = Real::acos(x);
    assert!(result.hi().abs() < EPSILON);
}

// =============================================================================
// Real Hyperbolic Functions
// =============================================================================

#[test]
fn test_realfield_sinh() {
    let x = Float106::from_f64(0.0);
    let result = Real::sinh(x);
    assert!(result.hi().abs() < EPSILON);
}

#[test]
fn test_realfield_cosh() {
    let x = Float106::from_f64(0.0);
    let result = Real::cosh(x);
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_realfield_tanh() {
    let x = Float106::from_f64(0.0);
    let result = Real::tanh(x);
    assert!(result.hi().abs() < EPSILON);
}

// =============================================================================
// Real atan2
// =============================================================================

#[test]
fn test_realfield_atan2() {
    let y = Float106::from_f64(1.0);
    let x = Float106::from_f64(1.0);
    let result = Real::atan2(y, x);
    assert!((result.hi() - std::f64::consts::PI / 4.0).abs() < 1e-10);
}

// =============================================================================
// Real Constants
// =============================================================================

#[test]
fn test_realfield_pi() {
    let pi = <Float106 as Real>::pi();
    assert!((pi.hi() - std::f64::consts::PI).abs() < EPSILON);
}

#[test]
fn test_realfield_e() {
    let e = <Float106 as Real>::e();
    assert!((e.hi() - std::f64::consts::E).abs() < EPSILON);
}

#[test]
fn test_realfield_epsilon() {
    let eps = <Float106 as Real>::epsilon();
    assert!(eps.hi() > 0.0);
    assert!(eps.hi() < 1e-30);
}

// =============================================================================
// DivisionAlgebra Tests
// =============================================================================

#[test]
fn test_divisionalgebra_conjugate() {
    let x = Float106::from_f64(5.0);
    // For reals, conjugate is identity
    let result = DivisionAlgebra::conjugate(&x);
    assert_eq!(result, x);
}

#[test]
fn test_divisionalgebra_norm_sqr() {
    let x = Float106::from_f64(3.0);
    let result = DivisionAlgebra::norm_sqr(&x);
    assert!((result.hi() - 9.0).abs() < EPSILON);
}

#[test]
fn test_divisionalgebra_inverse() {
    let x = Float106::from_f64(4.0);
    let result = DivisionAlgebra::inverse(&x);
    assert!((result.hi() - 0.25).abs() < EPSILON);
}

#[test]
fn test_divisionalgebra_inverse_identity() {
    let x = Float106::from_f64(4.0);
    let inv = DivisionAlgebra::inverse(&x);
    let product = x * inv;
    assert!((product.hi() - 1.0).abs() < EPSILON);
}

// =============================================================================
// Real classification traits
// =============================================================================

#[test]
fn test_realfield_is_nan() {
    assert!(<Float106 as Real>::is_nan(<Float106 as Real>::nan()));
    assert!(!<Float106 as Real>::is_nan(Float106::from_f64(1.0)));
}

#[test]
fn test_realfield_is_infinite() {
    let inf: Float106 = <Float106 as deep_causality_num::Float>::infinity();
    assert!(<Float106 as Real>::is_infinite(inf));
    assert!(!<Float106 as Real>::is_infinite(Float106::from_f64(1.0)));
}

#[test]
fn test_realfield_is_finite() {
    assert!(<Float106 as Real>::is_finite(Float106::from_f64(1.0)));
    let inf: Float106 = <Float106 as deep_causality_num::Float>::infinity();
    assert!(!<Float106 as Real>::is_finite(inf));
}
