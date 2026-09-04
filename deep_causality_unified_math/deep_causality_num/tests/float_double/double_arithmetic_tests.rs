/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for remaining arithmetic operations on DoubleFloat.
#![allow(clippy::op_ref)]

use deep_causality_num::Float106;

/// Both words of an exactly representable result.
///
/// Every expectation in this file is a small integer produced by one addition, subtraction,
/// multiplication or division of small integers, so the value fits the high word exactly and the
/// low word must be exactly zero. Checking the high word against a tolerance — as this file did —
/// accepts any low word at all, including a wrong one.
fn assert_exact(got: Float106, want: f64) {
    assert_eq!(got.hi(), want, "high word");
    assert_eq!(
        got.lo(),
        0.0,
        "low word must be exactly zero for an exactly representable result"
    );
}

// =============================================================================
// Cross-type Add with f64
// =============================================================================

#[test]
fn test_add_doublefloat_f64() {
    let a = Float106::from(3.0);
    let b = 2.0_f64;
    let result = a + b;
    assert_exact(result, 5.0);
}

#[test]
fn test_add_f64_doublefloat() {
    let a = 3.0_f64;
    let b = Float106::from(2.0);
    let result = a + b;
    assert_exact(result, 5.0);
}

// =============================================================================
// Cross-type Sub with f64
// =============================================================================

#[test]
fn test_sub_doublefloat_f64() {
    let a = Float106::from(5.0);
    let b = 2.0_f64;
    let result = a - b;
    assert_exact(result, 3.0);
}

#[test]
fn test_sub_f64_doublefloat() {
    let a = 5.0_f64;
    let b = Float106::from(2.0);
    let result = a - b;
    assert_exact(result, 3.0);
}

// =============================================================================
// Cross-type Mul with f64
// =============================================================================

#[test]
fn test_mul_doublefloat_f64() {
    let a = Float106::from(3.0);
    let b = 4.0_f64;
    let result = a * b;
    assert_exact(result, 12.0);
}

#[test]
fn test_mul_f64_doublefloat() {
    let a = 3.0_f64;
    let b = Float106::from(4.0);
    let result = a * b;
    assert_exact(result, 12.0);
}

// =============================================================================
// Cross-type Div with f64
// =============================================================================

#[test]
fn test_div_doublefloat_f64() {
    let a = Float106::from(12.0);
    let b = 4.0_f64;
    let result = a / b;
    assert_exact(result, 3.0);
}

#[test]
fn test_div_f64_doublefloat() {
    let a = 12.0_f64;
    let b = Float106::from(4.0);
    let result = a / b;
    assert_exact(result, 3.0);
}

// =============================================================================
// AddAssign with f64
// =============================================================================

#[test]
fn test_add_assign_f64() {
    let mut a = Float106::from(3.0);
    a += 2.0_f64;
    assert_exact(a, 5.0);
}

// =============================================================================
// SubAssign with f64
// =============================================================================

#[test]
fn test_sub_assign_f64() {
    let mut a = Float106::from(5.0);
    a -= 2.0_f64;
    assert_exact(a, 3.0);
}

// =============================================================================
// MulAssign with f64
// =============================================================================

#[test]
fn test_mul_assign_f64() {
    let mut a = Float106::from(3.0);
    a *= 4.0_f64;
    assert_exact(a, 12.0);
}

// =============================================================================
// DivAssign with f64
// =============================================================================

#[test]
fn test_div_assign_f64() {
    let mut a = Float106::from(12.0);
    a /= 4.0_f64;
    assert_exact(a, 3.0);
}

// =============================================================================
// Remainder (Rem) Operations
// =============================================================================

#[test]
fn test_rem() {
    let a = Float106::from(7.0);
    let b = Float106::from(3.0);
    let result = a % b;
    assert_exact(result, 1.0);
}

#[test]
fn test_rem_f64() {
    let a = Float106::from(7.0);
    let b = 3.0_f64;
    let result = a % b;
    assert_exact(result, 1.0);
}

#[test]
fn test_rem_assign() {
    let mut a = Float106::from(7.0);
    let b = Float106::from(3.0);
    a %= b;
    assert_exact(a, 1.0);
}

#[test]
fn test_rem_assign_f64() {
    let mut a = Float106::from(7.0);
    a %= 3.0_f64;
    assert_exact(a, 1.0);
}

// =============================================================================
// Negation
// =============================================================================

#[test]
fn test_neg_positive() {
    let a = Float106::from(42.0);
    let result = -a;
    assert_exact(result, -42.0);
}

#[test]
fn test_neg_negative() {
    let a = Float106::from(-42.0);
    let result = -a;
    assert_exact(result, 42.0);
}

#[test]
fn test_neg_zero() {
    let a = Float106::from(0.0);
    let result = -a;
    assert_exact(result, 0.0);
}

#[test]
fn test_neg_with_lo() {
    // Negation flips both words exactly; nothing is rounded, so this is bit equality rather
    // than a tolerance.
    let a = Float106::new(42.0, 1e-20);
    let result = -a;
    assert_eq!(result.hi(), -42.0);
    assert_eq!(result.lo(), -1e-20);
    // And it is an involution.
    assert_eq!((-result).hi(), a.hi());
    assert_eq!((-result).lo(), a.lo());
}

// =============================================================================
// Reference Operations
// =============================================================================

#[test]
fn test_add_ref_ref() {
    let a = Float106::from(3.0);
    let b = Float106::from(2.0);
    let result = &a + &b;
    assert_exact(result, 5.0);
}

#[test]
fn test_sub_ref_ref() {
    let a = Float106::from(5.0);
    let b = Float106::from(2.0);
    let result = &a - &b;
    assert_exact(result, 3.0);
}

#[test]
fn test_mul_ref_ref() {
    let a = Float106::from(3.0);
    let b = Float106::from(4.0);
    let result = &a * &b;
    assert_exact(result, 12.0);
}

#[test]
fn test_div_ref_ref() {
    let a = Float106::from(12.0);
    let b = Float106::from(4.0);
    let result = &a / &b;
    assert_exact(result, 3.0);
}

#[test]
fn test_add_ref_owned() {
    let a = Float106::from(3.0);
    let b = Float106::from(2.0);
    let result = &a + b;
    assert_exact(result, 5.0);
}

#[test]
fn test_add_owned_ref() {
    let a = Float106::from(3.0);
    let b = Float106::from(2.0);
    let result = a + &b;
    assert_exact(result, 5.0);
}

// =============================================================================
// Additional Reference Ops Coverage
// =============================================================================

#[test]
fn test_sub_ref_owned() {
    let a = Float106::from(5.0);
    let b = Float106::from(2.0);
    let result = &a - b;
    assert_exact(result, 3.0);
}

#[test]
fn test_sub_owned_ref() {
    let a = Float106::from(5.0);
    let b = Float106::from(2.0);
    let result = a - &b;
    assert_exact(result, 3.0);
}

#[test]
fn test_mul_ref_owned() {
    let a = Float106::from(3.0);
    let b = Float106::from(4.0);
    let result = &a * b;
    assert_exact(result, 12.0);
}

#[test]
fn test_mul_owned_ref() {
    let a = Float106::from(3.0);
    let b = Float106::from(4.0);
    let result = a * &b;
    assert_exact(result, 12.0);
}

#[test]
fn test_div_ref_owned() {
    let a = Float106::from(12.0);
    let b = Float106::from(4.0);
    let result = &a / b;
    assert_exact(result, 3.0);
}

#[test]
fn test_div_owned_ref() {
    let a = Float106::from(12.0);
    let b = Float106::from(4.0);
    let result = a / &b;
    assert_exact(result, 3.0);
}

#[test]
fn test_rem_ref_ref() {
    let a = Float106::from(7.0);
    let b = Float106::from(3.0);
    let result = &a % &b;
    assert_exact(result, 1.0);
}

#[test]
fn test_rem_ref_owned() {
    let a = Float106::from(7.0);
    let b = Float106::from(3.0);
    let result = &a % b;
    assert_exact(result, 1.0);
}

#[test]
fn test_rem_owned_ref() {
    let a = Float106::from(7.0);
    let b = Float106::from(3.0);
    let result = a % &b;
    assert_exact(result, 1.0);
}

#[test]
fn test_rem_f64_lhs() {
    let a = 7.0_f64;
    let b = Float106::from(3.0);
    let result = a % b;
    assert_exact(result, 1.0);
}
