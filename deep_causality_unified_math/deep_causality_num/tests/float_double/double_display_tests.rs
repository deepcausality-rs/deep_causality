/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Display, LowerExp, and UpperExp formatting for DoubleFloat.

use deep_causality_num::Float106;

// =============================================================================
// Display Tests
// =============================================================================

#[test]
fn test_display_lo_zero() {
    let x = Float106::from_f64(42.0);
    let s = format!("{}", x);
    assert_eq!(s, "42");
}

#[test]
fn test_display_lo_nonzero() {
    let x = Float106::new(42.0, 1e-20);
    let s = format!("{}", x);
    // Should show "hi+lo" format
    assert!(s.contains("42"));
    assert!(s.contains("+"));
}

#[test]
fn test_display_negative() {
    let x = Float106::from_f64(-42.0);
    let s = format!("{}", x);
    assert!(s.contains("-42"));
}

#[test]
fn test_display_zero() {
    let x = Float106::from_f64(0.0);
    let s = format!("{}", x);
    assert_eq!(s, "0");
}

#[test]
fn test_display_very_large() {
    let x = Float106::from_f64(1e100);
    let s = format!("{}", x);
    assert!(s.contains("e") || s.contains("E") || s.len() > 10);
}

#[test]
fn test_display_very_small() {
    let x = Float106::from_f64(1e-100);
    let s = format!("{}", x);
    // Very small numbers display in exponential notation
    assert!(!s.is_empty());
}

// =============================================================================
// LowerExp Tests
// =============================================================================

#[test]
fn test_lower_exp_positive() {
    let x = Float106::from_f64(1234.5);
    let s = format!("{:e}", x);
    assert!(s.contains("e"));
    assert!(s.contains("1.2345"));
}

#[test]
fn test_lower_exp_negative() {
    let x = Float106::from_f64(-1234.5);
    let s = format!("{:e}", x);
    assert!(s.contains("e"));
    assert!(s.contains("-1.2345"));
}

#[test]
fn test_lower_exp_one() {
    let x = Float106::from_f64(1.0);
    let s = format!("{:e}", x);
    assert!(s.contains("1"));
    assert!(s.contains("e"));
}

#[test]
fn test_lower_exp_very_large() {
    let x = Float106::from_f64(1e100);
    let s = format!("{:e}", x);
    assert!(s.contains("e+100") || s.contains("e100"));
}

#[test]
fn test_lower_exp_very_small() {
    let x = Float106::from_f64(1e-100);
    let s = format!("{:e}", x);
    assert!(s.contains("e-100"));
}

// =============================================================================
// UpperExp Tests
// =============================================================================

#[test]
fn test_upper_exp_positive() {
    let x = Float106::from_f64(1234.5);
    let s = format!("{:E}", x);
    assert!(s.contains("E"));
    assert!(s.contains("1.2345"));
}

#[test]
fn test_upper_exp_negative() {
    let x = Float106::from_f64(-1234.5);
    let s = format!("{:E}", x);
    assert!(s.contains("E"));
    assert!(s.contains("-1.2345"));
}

#[test]
fn test_upper_exp_one() {
    let x = Float106::from_f64(1.0);
    let s = format!("{:E}", x);
    assert!(s.contains("1"));
    assert!(s.contains("E"));
}

#[test]
fn test_upper_exp_very_large() {
    let x = Float106::from_f64(1e100);
    let s = format!("{:E}", x);
    assert!(s.contains("E+100") || s.contains("E100"));
}

#[test]
fn test_upper_exp_very_small() {
    let x = Float106::from_f64(1e-100);
    let s = format!("{:E}", x);
    assert!(s.contains("E-100"));
}

// =============================================================================
// Debug Tests
// =============================================================================

#[test]
fn test_debug_format() {
    let x = Float106::from_f64(42.0);
    let s = format!("{:?}", x);
    assert!(s.contains("DoubleFloat"));
    assert!(s.contains("42"));
}

#[test]
fn test_debug_with_lo() {
    let x = Float106::new(42.0, 1e-20);
    let s = format!("{:?}", x);
    assert!(s.contains("DoubleFloat"));
    assert!(s.contains("hi"));
    assert!(s.contains("lo"));
}
