/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for comparison operations on DoubleFloat.

use deep_causality_num::DoubleFloat;
use std::cmp::Ordering;

// =============================================================================
// PartialEq Tests
// =============================================================================

#[test]
fn test_eq_same() {
    let a = DoubleFloat::from_f64(42.0);
    let b = DoubleFloat::from_f64(42.0);
    assert_eq!(a, b);
}

#[test]
fn test_eq_different_hi() {
    let a = DoubleFloat::from_f64(42.0);
    let b = DoubleFloat::from_f64(43.0);
    assert_ne!(a, b);
}

#[test]
fn test_eq_different_lo() {
    let a = DoubleFloat::new(42.0, 1e-20);
    let b = DoubleFloat::new(42.0, 2e-20);
    assert_ne!(a, b);
}

#[test]
fn test_eq_zero() {
    let a = DoubleFloat::from_f64(0.0);
    let b = DoubleFloat::from_f64(0.0);
    assert_eq!(a, b);
}

// =============================================================================
// PartialEq<f64> Tests
// =============================================================================

#[test]
fn test_eq_f64_same() {
    let a = DoubleFloat::from_f64(42.0);
    assert!(a == 42.0_f64);
}

#[test]
fn test_eq_f64_different() {
    let a = DoubleFloat::from_f64(42.0);
    assert!(a != 43.0_f64);
}

#[test]
fn test_eq_f64_with_lo() {
    // If lo != 0, should not equal bare f64
    let a = DoubleFloat::new(42.0, 1e-20);
    assert!(a != 42.0_f64);
}

// =============================================================================
// f64 PartialEq<DoubleFloat> Tests
// =============================================================================

#[test]
fn test_f64_eq_doublefloat_same() {
    let a = DoubleFloat::from_f64(42.0);
    assert!(42.0_f64 == a);
}

#[test]
fn test_f64_eq_doublefloat_different() {
    let a = DoubleFloat::from_f64(42.0);
    assert!(43.0_f64 != a);
}

// =============================================================================
// PartialOrd Tests
// =============================================================================

#[test]
fn test_partial_cmp_equal() {
    let a = DoubleFloat::from_f64(42.0);
    let b = DoubleFloat::from_f64(42.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
}

#[test]
fn test_partial_cmp_less_by_hi() {
    let a = DoubleFloat::from_f64(41.0);
    let b = DoubleFloat::from_f64(42.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[test]
fn test_partial_cmp_greater_by_hi() {
    let a = DoubleFloat::from_f64(43.0);
    let b = DoubleFloat::from_f64(42.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
}

#[test]
fn test_partial_cmp_equal_hi_less_lo() {
    let a = DoubleFloat::new(42.0, 1e-20);
    let b = DoubleFloat::new(42.0, 2e-20);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[test]
fn test_partial_cmp_equal_hi_greater_lo() {
    let a = DoubleFloat::new(42.0, 3e-20);
    let b = DoubleFloat::new(42.0, 2e-20);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
}

// =============================================================================
// PartialOrd operator tests
// =============================================================================

#[test]
fn test_lt_by_hi() {
    let a = DoubleFloat::from_f64(41.0);
    let b = DoubleFloat::from_f64(42.0);
    assert!(a < b);
}

#[test]
fn test_lt_by_lo() {
    let a = DoubleFloat::new(42.0, 1e-20);
    let b = DoubleFloat::new(42.0, 2e-20);
    assert!(a < b);
}

#[test]
fn test_le_less() {
    let a = DoubleFloat::from_f64(41.0);
    let b = DoubleFloat::from_f64(42.0);
    assert!(a <= b);
}

#[test]
fn test_le_equal() {
    let a = DoubleFloat::from_f64(42.0);
    let b = DoubleFloat::from_f64(42.0);
    assert!(a <= b);
}

#[test]
fn test_gt_by_hi() {
    let a = DoubleFloat::from_f64(43.0);
    let b = DoubleFloat::from_f64(42.0);
    assert!(a > b);
}

#[test]
fn test_gt_by_lo() {
    let a = DoubleFloat::new(42.0, 3e-20);
    let b = DoubleFloat::new(42.0, 2e-20);
    assert!(a > b);
}

#[test]
fn test_ge_greater() {
    let a = DoubleFloat::from_f64(43.0);
    let b = DoubleFloat::from_f64(42.0);
    assert!(a >= b);
}

#[test]
fn test_ge_equal() {
    let a = DoubleFloat::from_f64(42.0);
    let b = DoubleFloat::from_f64(42.0);
    assert!(a >= b);
}

// =============================================================================
// PartialOrd<f64> Tests
// =============================================================================

#[test]
fn test_partial_cmp_f64_less() {
    let a = DoubleFloat::from_f64(41.0);
    assert_eq!(a.partial_cmp(&42.0_f64), Some(Ordering::Less));
}

#[test]
fn test_partial_cmp_f64_equal() {
    let a = DoubleFloat::from_f64(42.0);
    assert_eq!(a.partial_cmp(&42.0_f64), Some(Ordering::Equal));
}

#[test]
fn test_partial_cmp_f64_greater() {
    let a = DoubleFloat::from_f64(43.0);
    assert_eq!(a.partial_cmp(&42.0_f64), Some(Ordering::Greater));
}

// =============================================================================
// f64 PartialOrd<DoubleFloat> Tests
// =============================================================================

#[test]
fn test_f64_partial_cmp_doublefloat_less() {
    let a = DoubleFloat::from_f64(42.0);
    assert_eq!(41.0_f64.partial_cmp(&a), Some(Ordering::Less));
}

#[test]
fn test_f64_partial_cmp_doublefloat_equal() {
    let a = DoubleFloat::from_f64(42.0);
    assert_eq!(42.0_f64.partial_cmp(&a), Some(Ordering::Equal));
}

#[test]
fn test_f64_partial_cmp_doublefloat_greater() {
    let a = DoubleFloat::from_f64(42.0);
    assert_eq!(43.0_f64.partial_cmp(&a), Some(Ordering::Greater));
}
