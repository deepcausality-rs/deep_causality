/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic operation tests for `DoubleFloat`.

use deep_causality_num::{Float, Float106, One, Zero};

// =============================================================================
// Helper Functions
// =============================================================================

fn d(x: f64) -> Float106 {
    Float106::from_f64(x)
}

fn approx_eq(a: Float106, b: Float106, epsilon: f64) -> bool {
    let diff = <Float106 as Float>::abs(a - b);
    diff.hi() < epsilon
}

// =============================================================================
// Basic Arithmetic Tests
// =============================================================================

#[test]
fn test_addition_basic() {
    let a = d(1.0);
    let b = d(2.0);
    let c = a + b;
    assert!(approx_eq(c, d(3.0), 1e-15));
}

#[test]
fn test_addition_with_cancellation() {
    // Test high-precision addition where f64 would lose precision
    let a = d(1.0);
    let small = Float106::new(0.0, 1e-20);
    let result = a + small;
    assert_eq!(result.hi(), 1.0);
    assert!(result.lo() > 0.0); // Small value preserved
}

#[test]
fn test_subtraction_basic() {
    let a = d(5.0);
    let b = d(3.0);
    let c = a - b;
    assert!(approx_eq(c, d(2.0), 1e-15));
}

#[test]
fn test_subtraction_self() {
    let a = d(42.0);
    let c = a - a;
    assert!(c.is_zero());
}

#[test]
fn test_multiplication_basic() {
    let a = d(3.0);
    let b = d(4.0);
    let c = a * b;
    assert!(approx_eq(c, d(12.0), 1e-15));
}

#[test]
fn test_multiplication_by_zero() {
    let a = d(42.0);
    let c = a * d(0.0);
    assert!(c.is_zero());
}

#[test]
fn test_multiplication_by_one() {
    let a = d(42.0);
    let c = a * d(1.0);
    assert!(approx_eq(c, a, 1e-15));
}

#[test]
fn test_division_basic() {
    let a = d(12.0);
    let b = d(4.0);
    let c = a / b;
    assert!(approx_eq(c, d(3.0), 1e-15));
}

#[test]
fn test_division_by_self() {
    let a = d(42.0);
    let c = a / a;
    assert!(approx_eq(c, d(1.0), 1e-15));
}

#[test]
fn test_negation() {
    let a = d(5.0);
    let b = -a;
    assert!(approx_eq(b, d(-5.0), 1e-15));
    assert!(approx_eq(-b, a, 1e-15));
}

#[test]
fn test_remainder() {
    let a = d(7.0);
    let b = d(3.0);
    let c = a % b;
    assert!(approx_eq(c, d(1.0), 1e-15));
}

// =============================================================================
// Special Values Tests
// =============================================================================

#[test]
fn test_nan_propagation() {
    let nan = <Float106 as Float>::nan();
    assert!(nan.is_nan());
    assert!((nan + d(1.0)).is_nan());
    assert!((nan * d(1.0)).is_nan());
}

#[test]
fn test_infinity() {
    let inf = <Float106 as Float>::infinity();
    assert!(inf.is_infinite());
    assert!(!inf.is_finite());
    assert!(inf > d(f64::MAX));
}

#[test]
fn test_neg_infinity() {
    let neg_inf = <Float106 as Float>::neg_infinity();
    assert!(neg_inf.is_infinite());
    assert!(neg_inf < d(f64::MIN));
}

#[test]
fn test_zero_and_one() {
    let zero = Float106::zero();
    let one = Float106::one();

    assert!(zero.is_zero());
    assert!(one.is_one());
    assert!(approx_eq(zero + one, one, 1e-15));
    assert!(approx_eq(one * one, one, 1e-15));
}

// =============================================================================
// Comparison Tests
// =============================================================================

#[test]
fn test_equality() {
    let a = d(1.0);
    let b = d(1.0);
    assert_eq!(a, b);
}

#[test]
fn test_inequality() {
    let a = d(1.0);
    let b = d(2.0);
    assert_ne!(a, b);
}

#[test]
fn test_ordering() {
    let a = d(1.0);
    let b = d(2.0);
    assert!(a < b);
    assert!(b > a);
    assert!(a <= b);
    assert!(b >= a);
}

#[test]
fn test_ordering_with_lo_component() {
    let a = Float106::new(1.0, 1e-20);
    let b = Float106::new(1.0, 2e-20);
    assert!(a < b);
}

// =============================================================================
// Assignment Operator Tests
// =============================================================================

#[test]
fn test_add_assign() {
    let mut a = d(1.0);
    a += d(2.0);
    assert!(approx_eq(a, d(3.0), 1e-15));
}

#[test]
fn test_sub_assign() {
    let mut a = d(5.0);
    a -= d(3.0);
    assert!(approx_eq(a, d(2.0), 1e-15));
}

#[test]
fn test_mul_assign() {
    let mut a = d(3.0);
    a *= d(4.0);
    assert!(approx_eq(a, d(12.0), 1e-15));
}

#[test]
fn test_div_assign() {
    let mut a = d(12.0);
    a /= d(4.0);
    assert!(approx_eq(a, d(3.0), 1e-15));
}

#[test]
fn test_rem_assign() {
    let mut a = d(7.0);
    a %= d(3.0);
    assert!(approx_eq(a, d(1.0), 1e-15));
}

// =============================================================================
// Algebraic Property Tests
// =============================================================================

#[test]
fn test_commutativity_addition() {
    let a = d(2.5);
    let b = d(3.7);
    assert_eq!(a + b, b + a);
}

#[test]
fn test_commutativity_multiplication() {
    let a = d(2.5);
    let b = d(3.7);
    assert!(approx_eq(a * b, b * a, 1e-15));
}

#[test]
fn test_associativity_addition() {
    let a = d(1.1);
    let b = d(2.2);
    let c = d(3.3);
    // Note: f64 may fail this due to rounding, but DoubleFloat should maintain it better
    let lhs = (a + b) + c;
    let rhs = a + (b + c);
    assert!(approx_eq(lhs, rhs, 1e-14));
}

#[test]
fn test_associativity_multiplication() {
    let a = d(1.1);
    let b = d(2.2);
    let c = d(3.3);
    let lhs = (a * b) * c;
    let rhs = a * (b * c);
    assert!(approx_eq(lhs, rhs, 1e-14));
}

#[test]
fn test_distributivity() {
    let a = d(2.0);
    let b = d(3.0);
    let c = d(4.0);
    let lhs = a * (b + c);
    let rhs = a * b + a * c;
    assert!(approx_eq(lhs, rhs, 1e-14));
}

// =============================================================================
// Precision Tests
// =============================================================================

#[test]
fn test_high_precision_constants() {
    let pi = Float106::PI;
    // Verify pi.hi + pi.lo is close to the true value
    let sum = pi.hi() + pi.lo();
    let expected = core::f64::consts::PI;
    assert!((sum - expected).abs() < 1e-15);
}

#[test]
fn test_precision_beyond_f64() {
    // Create a value that requires DoubleFloat precision
    let a = d(1.0);
    let tiny = Float106::new(0.0, 1e-17);
    let result = a + tiny;
    // The tiny addition should be preserved in lo component
    assert_eq!(result.hi(), 1.0);
    assert!(result.lo() > 0.0);
}

// =============================================================================
// Conversion Tests
// =============================================================================

#[test]
fn test_from_f64() {
    let x = Float106::from_f64(42.0);
    assert_eq!(x.hi(), 42.0);
    assert_eq!(x.lo(), 0.0);
}

#[test]
fn test_to_f64() {
    let x = d(42.0);
    assert_eq!(x.to_f64(), 42.0);
}

#[test]
fn test_from_i32() {
    let x: Float106 = 42_i32.into();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_into_f64() {
    let x = d(42.0);
    let y: f64 = x.into();
    assert_eq!(y, 42.0);
}

// Note: Reference operation impls exist but are trivial passthrough
// to owned operations, so we don't test them separately to avoid
// clippy::op_ref warnings.
