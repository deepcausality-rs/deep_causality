/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::SignedInt;

/// Test absolute value for signed integers.
#[test]
fn test_abs() {
    assert_eq!((-5i32).abs(), 5);
    assert_eq!(5i32.abs(), 5);
    assert_eq!(0i32.abs(), 0);

    assert_eq!((-100i64).abs(), 100);
    assert_eq!((-1isize).abs(), 1);
}

/// Test signum function.
#[test]
fn test_signum() {
    assert_eq!((-42i32).signum(), -1);
    assert_eq!(0i32.signum(), 0);
    assert_eq!(42i32.signum(), 1);

    assert_eq!(i64::MIN.signum(), -1);
    assert_eq!(i64::MAX.signum(), 1);
}

/// Test is_negative and is_positive.
#[test]
fn test_sign_predicates() {
    assert!((-1i32).is_negative());
    assert!(!(-1i32).is_positive());

    assert!(1i32.is_positive());
    assert!(!1i32.is_negative());

    // Zero is neither positive nor negative
    assert!(!0i32.is_positive());
    assert!(!0i32.is_negative());
}

/// Test checked_abs for MIN value edge case.
#[test]
fn test_checked_abs() {
    // Normal cases work
    assert_eq!((-5i32).checked_abs(), Some(5));
    assert_eq!(5i32.checked_abs(), Some(5));
    assert_eq!(0i32.checked_abs(), Some(0));

    // MIN cannot be represented as positive
    assert_eq!(i8::MIN.checked_abs(), None);
    assert_eq!(i32::MIN.checked_abs(), None);
    assert_eq!(i64::MIN.checked_abs(), None);
}

/// Test checked_neg for MIN value edge case.
#[test]
fn test_checked_neg() {
    assert_eq!(5i32.checked_neg(), Some(-5));
    assert_eq!((-5i32).checked_neg(), Some(5));
    assert_eq!(0i32.checked_neg(), Some(0));

    // MIN cannot be negated
    assert_eq!(i8::MIN.checked_neg(), None);
    assert_eq!(i32::MIN.checked_neg(), None);
}

/// Test saturating_abs.
#[test]
fn test_saturating_abs() {
    assert_eq!((-5i32).saturating_abs(), 5);
    assert_eq!(5i32.saturating_abs(), 5);

    // MIN saturates to MAX instead of overflowing
    assert_eq!(i8::MIN.saturating_abs(), i8::MAX);
    assert_eq!(i32::MIN.saturating_abs(), i32::MAX);
}

/// Test wrapping_abs.
#[test]
fn test_wrapping_abs() {
    assert_eq!((-5i32).wrapping_abs(), 5);
    assert_eq!(5i32.wrapping_abs(), 5);

    // MIN wraps to itself (two's complement)
    assert_eq!(i8::MIN.wrapping_abs(), i8::MIN);
    assert_eq!(i32::MIN.wrapping_abs(), i32::MIN);
}

/// Test wrapping_neg.
#[test]
fn test_wrapping_neg() {
    assert_eq!(5i32.wrapping_neg(), -5);
    assert_eq!((-5i32).wrapping_neg(), 5);
    assert_eq!(0i32.wrapping_neg(), 0);

    // MIN wraps to itself
    assert_eq!(i8::MIN.wrapping_neg(), i8::MIN);
}

/// Test negation via Neg trait (required by SignedInt).
#[test]
fn test_neg_trait() {
    let x: i32 = 42;
    assert_eq!(-x, -42);
    assert_eq!(-(-x), 42);

    let y: i64 = -100;
    assert_eq!(-y, 100);
}

/// Generic function test using SignedInt bound.
#[test]
fn test_generic_signed_function() {
    fn safe_abs<T: SignedInt>(val: T) -> Option<T> {
        val.checked_abs()
    }

    assert_eq!(safe_abs(-10i32), Some(10));
    assert_eq!(safe_abs(i8::MIN), None);
}

/// Test all signed types implement SignedInt.
#[test]
fn test_signed_types() {
    fn require_signed<T: SignedInt>() {}

    require_signed::<i8>();
    require_signed::<i16>();
    require_signed::<i32>();
    require_signed::<i64>();
    require_signed::<i128>();
    require_signed::<isize>();
}
