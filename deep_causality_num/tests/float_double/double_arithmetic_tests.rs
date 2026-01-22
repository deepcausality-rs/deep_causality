/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for remaining arithmetic operations on DoubleFloat.
#![allow(clippy::op_ref)]

use deep_causality_num::Float106;

const EPSILON: f64 = 1e-14;

// =============================================================================
// Cross-type Add with f64
// =============================================================================

#[test]
fn test_add_doublefloat_f64() {
    let a = Float106::from_f64(3.0);
    let b = 2.0_f64;
    let result = a + b;
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

#[test]
fn test_add_f64_doublefloat() {
    let a = 3.0_f64;
    let b = Float106::from_f64(2.0);
    let result = a + b;
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

// =============================================================================
// Cross-type Sub with f64
// =============================================================================

#[test]
fn test_sub_doublefloat_f64() {
    let a = Float106::from_f64(5.0);
    let b = 2.0_f64;
    let result = a - b;
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_sub_f64_doublefloat() {
    let a = 5.0_f64;
    let b = Float106::from_f64(2.0);
    let result = a - b;
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

// =============================================================================
// Cross-type Mul with f64
// =============================================================================

#[test]
fn test_mul_doublefloat_f64() {
    let a = Float106::from_f64(3.0);
    let b = 4.0_f64;
    let result = a * b;
    assert!((result.hi() - 12.0).abs() < EPSILON);
}

#[test]
fn test_mul_f64_doublefloat() {
    let a = 3.0_f64;
    let b = Float106::from_f64(4.0);
    let result = a * b;
    assert!((result.hi() - 12.0).abs() < EPSILON);
}

// =============================================================================
// Cross-type Div with f64
// =============================================================================

#[test]
fn test_div_doublefloat_f64() {
    let a = Float106::from_f64(12.0);
    let b = 4.0_f64;
    let result = a / b;
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_div_f64_doublefloat() {
    let a = 12.0_f64;
    let b = Float106::from_f64(4.0);
    let result = a / b;
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

// =============================================================================
// AddAssign with f64
// =============================================================================

#[test]
fn test_add_assign_f64() {
    let mut a = Float106::from_f64(3.0);
    a += 2.0_f64;
    assert!((a.hi() - 5.0).abs() < EPSILON);
}

// =============================================================================
// SubAssign with f64
// =============================================================================

#[test]
fn test_sub_assign_f64() {
    let mut a = Float106::from_f64(5.0);
    a -= 2.0_f64;
    assert!((a.hi() - 3.0).abs() < EPSILON);
}

// =============================================================================
// MulAssign with f64
// =============================================================================

#[test]
fn test_mul_assign_f64() {
    let mut a = Float106::from_f64(3.0);
    a *= 4.0_f64;
    assert!((a.hi() - 12.0).abs() < EPSILON);
}

// =============================================================================
// DivAssign with f64
// =============================================================================

#[test]
fn test_div_assign_f64() {
    let mut a = Float106::from_f64(12.0);
    a /= 4.0_f64;
    assert!((a.hi() - 3.0).abs() < EPSILON);
}

// =============================================================================
// Remainder (Rem) Operations
// =============================================================================

#[test]
fn test_rem() {
    let a = Float106::from_f64(7.0);
    let b = Float106::from_f64(3.0);
    let result = a % b;
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_rem_f64() {
    let a = Float106::from_f64(7.0);
    let b = 3.0_f64;
    let result = a % b;
    assert!((result.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_rem_assign() {
    let mut a = Float106::from_f64(7.0);
    let b = Float106::from_f64(3.0);
    a %= b;
    assert!((a.hi() - 1.0).abs() < EPSILON);
}

#[test]
fn test_rem_assign_f64() {
    let mut a = Float106::from_f64(7.0);
    a %= 3.0_f64;
    assert!((a.hi() - 1.0).abs() < EPSILON);
}

// =============================================================================
// Negation
// =============================================================================

#[test]
fn test_neg_positive() {
    let a = Float106::from_f64(42.0);
    let result = -a;
    assert!((result.hi() - (-42.0)).abs() < EPSILON);
}

#[test]
fn test_neg_negative() {
    let a = Float106::from_f64(-42.0);
    let result = -a;
    assert!((result.hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_neg_zero() {
    let a = Float106::from_f64(0.0);
    let result = -a;
    assert!((result.hi() - 0.0).abs() < EPSILON);
}

#[test]
fn test_neg_with_lo() {
    let a = Float106::new(42.0, 1e-20);
    let result = -a;
    assert!((result.hi() - (-42.0)).abs() < EPSILON);
    assert!((result.lo() - (-1e-20)).abs() < 1e-30);
}

// =============================================================================
// Reference Operations
// =============================================================================

#[test]
fn test_add_ref_ref() {
    let a = Float106::from_f64(3.0);
    let b = Float106::from_f64(2.0);
    let result = &a + &b;
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

#[test]
fn test_sub_ref_ref() {
    let a = Float106::from_f64(5.0);
    let b = Float106::from_f64(2.0);
    let result = &a - &b;
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_mul_ref_ref() {
    let a = Float106::from_f64(3.0);
    let b = Float106::from_f64(4.0);
    let result = &a * &b;
    assert!((result.hi() - 12.0).abs() < EPSILON);
}

#[test]
fn test_div_ref_ref() {
    let a = Float106::from_f64(12.0);
    let b = Float106::from_f64(4.0);
    let result = &a / &b;
    assert!((result.hi() - 3.0).abs() < EPSILON);
}

#[test]
fn test_add_ref_owned() {
    let a = Float106::from_f64(3.0);
    let b = Float106::from_f64(2.0);
    let result = &a + b;
    assert!((result.hi() - 5.0).abs() < EPSILON);
}

#[test]
fn test_add_owned_ref() {
    let a = Float106::from_f64(3.0);
    let b = Float106::from_f64(2.0);
    let result = a + &b;
    assert!((result.hi() - 5.0).abs() < EPSILON);
}
