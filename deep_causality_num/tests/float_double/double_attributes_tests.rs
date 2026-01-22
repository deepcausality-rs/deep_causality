/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for DoubleFloat attribute methods.

use deep_causality_num::{Float106, Float};

// =============================================================================
// is_nan Tests
// =============================================================================

#[test]
fn test_is_nan_true() {
    let x = <Float106 as Float>::nan();
    assert!(x.is_nan());
}

#[test]
fn test_is_nan_false_normal() {
    let x = Float106::from_f64(42.0);
    assert!(!x.is_nan());
}

#[test]
fn test_is_nan_false_infinity() {
    let x = <Float106 as Float>::infinity();
    assert!(!x.is_nan());
}

#[test]
fn test_is_nan_false_zero() {
    let x = Float106::from_f64(0.0);
    assert!(!x.is_nan());
}

// =============================================================================
// is_infinite Tests
// =============================================================================

#[test]
fn test_is_infinite_pos_inf() {
    let x = <Float106 as Float>::infinity();
    assert!(x.is_infinite());
}

#[test]
fn test_is_infinite_neg_inf() {
    let x = <Float106 as Float>::neg_infinity();
    assert!(x.is_infinite());
}

#[test]
fn test_is_infinite_false_normal() {
    let x = Float106::from_f64(42.0);
    assert!(!x.is_infinite());
}

#[test]
fn test_is_infinite_false_nan() {
    let x = <Float106 as Float>::nan();
    assert!(!x.is_infinite());
}

#[test]
fn test_is_infinite_false_zero() {
    let x = Float106::from_f64(0.0);
    assert!(!x.is_infinite());
}

// =============================================================================
// is_finite Tests
// =============================================================================

#[test]
fn test_is_finite_true_normal() {
    let x = Float106::from_f64(42.0);
    assert!(x.is_finite());
}

#[test]
fn test_is_finite_true_zero() {
    let x = Float106::from_f64(0.0);
    assert!(x.is_finite());
}

#[test]
fn test_is_finite_true_negative() {
    let x = Float106::from_f64(-42.0);
    assert!(x.is_finite());
}

#[test]
fn test_is_finite_false_pos_inf() {
    let x = <Float106 as Float>::infinity();
    assert!(!x.is_finite());
}

#[test]
fn test_is_finite_false_neg_inf() {
    let x = <Float106 as Float>::neg_infinity();
    assert!(!x.is_finite());
}

#[test]
fn test_is_finite_false_nan() {
    let x = <Float106 as Float>::nan();
    assert!(!x.is_finite());
}

// =============================================================================
// is_sign_positive Tests
// =============================================================================

#[test]
fn test_is_sign_positive_true() {
    let x = Float106::from_f64(42.0);
    assert!(x.is_sign_positive());
}

#[test]
fn test_is_sign_positive_false() {
    let x = Float106::from_f64(-42.0);
    assert!(!x.is_sign_positive());
}

#[test]
fn test_is_sign_positive_zero() {
    let x = Float106::from_f64(0.0);
    assert!(x.is_sign_positive());
}

#[test]
fn test_is_sign_positive_pos_inf() {
    let x = <Float106 as Float>::infinity();
    assert!(x.is_sign_positive());
}

// =============================================================================
// is_sign_negative Tests
// =============================================================================

#[test]
fn test_is_sign_negative_true() {
    let x = Float106::from_f64(-42.0);
    assert!(x.is_sign_negative());
}

#[test]
fn test_is_sign_negative_false() {
    let x = Float106::from_f64(42.0);
    assert!(!x.is_sign_negative());
}

#[test]
fn test_is_sign_negative_neg_zero() {
    let x = Float106::from_f64(-0.0);
    assert!(x.is_sign_negative());
}

#[test]
fn test_is_sign_negative_neg_inf() {
    let x = <Float106 as Float>::neg_infinity();
    assert!(x.is_sign_negative());
}
