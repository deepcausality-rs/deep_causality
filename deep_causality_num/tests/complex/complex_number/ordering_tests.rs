/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for Complex number ordering (PartialOrd and Ord).

use deep_causality_num::Complex64;
use std::cmp::Ordering;

// =============================================================================
// PartialOrd Tests
// =============================================================================

#[test]
fn test_partial_cmp_equal() {
    let a = Complex64::new(3.0, 4.0);
    let b = Complex64::new(3.0, 4.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
}

#[test]
fn test_partial_cmp_less_by_real() {
    let a = Complex64::new(2.0, 4.0);
    let b = Complex64::new(3.0, 4.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[test]
fn test_partial_cmp_greater_by_real() {
    let a = Complex64::new(5.0, 4.0);
    let b = Complex64::new(3.0, 4.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
}

#[test]
fn test_partial_cmp_equal_real_less_imag() {
    let a = Complex64::new(3.0, 2.0);
    let b = Complex64::new(3.0, 4.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[test]
fn test_partial_cmp_equal_real_greater_imag() {
    let a = Complex64::new(3.0, 6.0);
    let b = Complex64::new(3.0, 4.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Greater));
}

#[test]
fn test_partial_cmp_negative_values() {
    let a = Complex64::new(-5.0, -3.0);
    let b = Complex64::new(-5.0, -2.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

#[test]
fn test_partial_cmp_zero() {
    let a = Complex64::new(0.0, 0.0);
    let b = Complex64::new(0.0, 0.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
}

#[test]
fn test_partial_cmp_zero_vs_positive() {
    let a = Complex64::new(0.0, 0.0);
    let b = Complex64::new(1.0, 0.0);
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Less));
}

// =============================================================================
// Comparison Operator Tests
// =============================================================================

#[test]
fn test_lt() {
    let a = Complex64::new(1.0, 2.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a < b);
}

#[test]
fn test_lt_equal_real() {
    let a = Complex64::new(3.0, 2.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a < b);
}

#[test]
fn test_le_less() {
    let a = Complex64::new(1.0, 2.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a <= b);
}

#[test]
fn test_le_equal() {
    let a = Complex64::new(3.0, 4.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a <= b);
}

#[test]
fn test_gt() {
    let a = Complex64::new(5.0, 6.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a > b);
}

#[test]
fn test_gt_equal_real() {
    let a = Complex64::new(3.0, 6.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a > b);
}

#[test]
fn test_ge_greater() {
    let a = Complex64::new(5.0, 6.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a >= b);
}

#[test]
fn test_ge_equal() {
    let a = Complex64::new(3.0, 4.0);
    let b = Complex64::new(3.0, 4.0);
    assert!(a >= b);
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_ordering_with_negative_zero() {
    let a = Complex64::new(0.0, 0.0);
    let b = Complex64::new(-0.0, -0.0);
    // -0.0 == 0.0 in IEEE 754
    assert_eq!(a.partial_cmp(&b), Some(Ordering::Equal));
}

#[test]
fn test_ordering_small_differences() {
    let a = Complex64::new(1.0, 1e-15);
    let b = Complex64::new(1.0, 2e-15);
    assert!(a < b);
}
