/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for From trait implementations for DoubleFloat.

use deep_causality_num::Float106;

/// Both words of an exactly representable result.
///
/// Every value converted or accumulated in this file is exact in the high word — a small
/// integer, a dyadic fraction, or an integer bound — so the low word must be exactly zero.
/// A tolerance on the high word alone accepts any low word, including a wrong one.
fn assert_exact(got: Float106, want: f64) {
    assert_eq!(got.hi(), want, "high word");
    assert_eq!(
        got.lo(),
        0.0,
        "low word must be exactly zero for an exactly representable result"
    );
}

// =============================================================================
// From<f64> for DoubleFloat
// =============================================================================

#[test]
fn test_from_f64_positive() {
    let x: Float106 = 42.5_f64.into();
    assert_exact(x, 42.5);
}

#[test]
fn test_from_f64_negative() {
    let x: Float106 = (-42.5_f64).into();
    assert_exact(x, -42.5);
}

#[test]
fn test_from_f64_zero() {
    let x: Float106 = 0.0_f64.into();
    assert_eq!(x.hi(), 0.0);
    assert_eq!(x.lo(), 0.0);
}

#[test]
fn test_from_f64_very_small() {
    // Widening an f64 is exact at every magnitude, subnormals included.
    for v in [1e-300_f64, 1e300, f64::MIN_POSITIVE, f64::MAX, 5e-324] {
        assert_exact(Float106::from(v), v);
        assert_exact(Float106::from(-v), -v);
    }
}

// =============================================================================
// From<f32> for DoubleFloat
// =============================================================================

#[test]
fn test_from_f32_positive() {
    let x: Float106 = 42.5_f32.into();
    assert_exact(x, 42.5);
}

#[test]
fn test_from_f32_negative() {
    let x: Float106 = (-42.5_f32).into();
    assert_exact(x, -42.5);
}

#[test]
fn test_from_f32_zero() {
    let x: Float106 = 0.0_f32.into();
    assert_eq!(x.hi(), 0.0);
}

// =============================================================================
// From<i32> for DoubleFloat
// =============================================================================

#[test]
fn test_from_i32_positive() {
    let x: Float106 = 42_i32.into();
    assert_exact(x, 42.0);
}

#[test]
fn test_from_i32_negative() {
    let x: Float106 = (-42_i32).into();
    assert_exact(x, -42.0);
}

#[test]
fn test_from_i32_zero() {
    let x: Float106 = 0_i32.into();
    assert_eq!(x.hi(), 0.0);
}

#[test]
fn test_from_i32_max() {
    let x: Float106 = i32::MAX.into();
    assert_exact(x, i32::MAX as f64);
}

#[test]
fn test_from_i32_min() {
    let x: Float106 = i32::MIN.into();
    assert_exact(x, i32::MIN as f64);
}

// =============================================================================
// From<i64> for DoubleFloat
// =============================================================================

#[test]
fn test_from_i64_positive() {
    let x: Float106 = 42_i64.into();
    assert_exact(x, 42.0);
}

#[test]
fn test_from_i64_negative() {
    let x: Float106 = (-42_i64).into();
    assert_exact(x, -42.0);
}

#[test]
fn test_from_i64_zero() {
    let x: Float106 = 0_i64.into();
    assert_eq!(x.hi(), 0.0);
}

// =============================================================================
// From<u32> for DoubleFloat
// =============================================================================

#[test]
fn test_from_u32_positive() {
    let x: Float106 = 42_u32.into();
    assert_exact(x, 42.0);
}

#[test]
fn test_from_u32_zero() {
    let x: Float106 = 0_u32.into();
    assert_eq!(x.hi(), 0.0);
}

#[test]
fn test_from_u32_max() {
    let x: Float106 = u32::MAX.into();
    assert_exact(x, u32::MAX as f64);
}

// =============================================================================
// From<u64> for DoubleFloat
// =============================================================================

#[test]
fn test_from_u64_positive() {
    let x: Float106 = 42_u64.into();
    assert_exact(x, 42.0);
}

#[test]
fn test_from_u64_zero() {
    let x: Float106 = 0_u64.into();
    assert_eq!(x.hi(), 0.0);
}

// =============================================================================
// From<DoubleFloat> for f64
// =============================================================================

#[test]
fn test_to_f64_positive() {
    // 42.5 is exactly representable, so widening and narrowing must return it unchanged.
    let x = Float106::from(42.5);
    let y: f64 = x.into();
    assert_eq!(y, 42.5);
    // The round trip is the identity on every exactly representable f64.
    for v in [
        42.5_f64,
        0.5,
        1.0,
        -1.0,
        1e-300,
        1e300,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        let back: f64 = Float106::from(v).into();
        assert_eq!(back, v, "round trip at {v}");
    }
}

#[test]
fn test_to_f64_negative() {
    let x = Float106::from(-42.5);
    let y: f64 = x.into();
    assert_eq!(y, -42.5);
    for v in [-42.5_f64, -0.5, -1e-300, -1e300, f64::MIN] {
        let back: f64 = Float106::from(v).into();
        assert_eq!(back, v, "round trip at {v}");
    }
}

#[test]
fn test_to_f64_zero() {
    let x = Float106::from(0.0);
    let y: f64 = x.into();
    assert_eq!(y, 0.0);
}

// =============================================================================
// From<DoubleFloat> for f32
// =============================================================================

#[test]
fn test_to_f32_positive() {
    // 42.5 is exactly representable in f32, so the narrowing is exact, not approximate.
    let x = Float106::from(42.5);
    let y: f32 = x.into();
    assert_eq!(y, 42.5_f32);
    for v in [42.5_f32, 0.5, 1.0, 1024.0, f32::MIN_POSITIVE] {
        let back: f32 = Float106::from(v as f64).into();
        assert_eq!(back, v, "narrowing at {v}");
    }
}

#[test]
fn test_to_f32_negative() {
    let x = Float106::from(-42.5);
    let y: f32 = x.into();
    assert_eq!(y, -42.5_f32);
}

#[test]
fn test_to_f32_zero() {
    let x = Float106::from(0.0);
    let y: f32 = x.into();
    assert_eq!(y, 0.0_f32);
}
