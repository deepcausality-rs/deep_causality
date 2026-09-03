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
    let x = Float106::from(42.0);
    let s = format!("{}", x);
    assert_eq!(s, "42");
}

#[test]
fn test_display_renders_the_value_not_its_halves() {
    // A `lo` below f64 resolution does not show: Display renders the value to f64 precision and
    // Debug is where the two components are.
    let x = Float106::new(42.0, 1e-20);
    assert_eq!(format!("{}", x), "42");
    assert!(!format!("{}", x).contains('+'));
    assert!(format!("{:?}", x).contains("lo"));
}

#[test]
fn test_display_honours_precision_width_and_sign() {
    // The flags a caller writes at the display boundary reach the number.
    let x = Float106::from(0.37695);
    assert_eq!(format!("{:.3}", x), "0.377");
    assert_eq!(format!("{:.1}", x), "0.4");
    assert_eq!(format!("{:>8.2}", x), "    0.38");
    assert_eq!(format!("{:<8.2}|", x), "0.38    |");
    assert_eq!(format!("{:+.2}", x), "+0.38");
    let big = Float106::from(1234.5);
    assert_eq!(format!("{:.3e}", big), "1.234e3");
    assert_eq!(format!("{:.0}", big), "1234");
}

#[test]
fn test_display_negative() {
    let x = Float106::from(-42.0);
    let s = format!("{}", x);
    assert!(s.contains("-42"));
}

#[test]
fn test_display_zero() {
    let x = Float106::from(0.0);
    let s = format!("{}", x);
    assert_eq!(s, "0");
}

#[test]
fn test_display_very_large() {
    let x = Float106::from(1e100);
    let s = format!("{}", x);
    assert!(s.contains("e") || s.contains("E") || s.len() > 10);
}

#[test]
fn test_display_very_small() {
    let x = Float106::from(1e-100);
    let s = format!("{}", x);
    // Very small numbers display in exponential notation
    assert!(!s.is_empty());
}

// =============================================================================
// LowerExp Tests
// =============================================================================

#[test]
fn test_lower_exp_positive() {
    let x = Float106::from(1234.5);
    let s = format!("{:e}", x);
    assert!(s.contains("e"));
    assert!(s.contains("1.2345"));
}

#[test]
fn test_lower_exp_negative() {
    let x = Float106::from(-1234.5);
    let s = format!("{:e}", x);
    assert!(s.contains("e"));
    assert!(s.contains("-1.2345"));
}

#[test]
fn test_lower_exp_one() {
    let x = Float106::from(1.0);
    let s = format!("{:e}", x);
    assert!(s.contains("1"));
    assert!(s.contains("e"));
}

#[test]
fn test_lower_exp_very_large() {
    let x = Float106::from(1e100);
    let s = format!("{:e}", x);
    assert!(s.contains("e+100") || s.contains("e100"));
}

#[test]
fn test_lower_exp_very_small() {
    let x = Float106::from(1e-100);
    let s = format!("{:e}", x);
    assert!(s.contains("e-100"));
}

// =============================================================================
// UpperExp Tests
// =============================================================================

#[test]
fn test_upper_exp_positive() {
    let x = Float106::from(1234.5);
    let s = format!("{:E}", x);
    assert!(s.contains("E"));
    assert!(s.contains("1.2345"));
}

#[test]
fn test_upper_exp_negative() {
    let x = Float106::from(-1234.5);
    let s = format!("{:E}", x);
    assert!(s.contains("E"));
    assert!(s.contains("-1.2345"));
}

#[test]
fn test_upper_exp_one() {
    let x = Float106::from(1.0);
    let s = format!("{:E}", x);
    assert!(s.contains("1"));
    assert!(s.contains("E"));
}

#[test]
fn test_upper_exp_very_large() {
    let x = Float106::from(1e100);
    let s = format!("{:E}", x);
    assert!(s.contains("E+100") || s.contains("E100"));
}

#[test]
fn test_upper_exp_very_small() {
    let x = Float106::from(1e-100);
    let s = format!("{:E}", x);
    assert!(s.contains("E-100"));
}

// =============================================================================
// Debug Tests
// =============================================================================

#[test]
fn test_debug_format() {
    let x = Float106::from(42.0);
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
