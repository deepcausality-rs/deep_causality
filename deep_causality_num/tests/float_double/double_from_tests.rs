/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for From trait implementations for DoubleFloat.

use deep_causality_num::DoubleFloat;

const EPSILON: f64 = 1e-15;

// =============================================================================
// From<f64> for DoubleFloat
// =============================================================================

#[test]
fn test_from_f64_positive() {
    let x: DoubleFloat = 42.5_f64.into();
    assert!((x.hi() - 42.5).abs() < EPSILON);
    assert_eq!(x.lo(), 0.0);
}

#[test]
fn test_from_f64_negative() {
    let x: DoubleFloat = (-42.5_f64).into();
    assert!((x.hi() - (-42.5)).abs() < EPSILON);
}

#[test]
fn test_from_f64_zero() {
    let x: DoubleFloat = 0.0_f64.into();
    assert_eq!(x.hi(), 0.0);
    assert_eq!(x.lo(), 0.0);
}

#[test]
fn test_from_f64_very_small() {
    let x: DoubleFloat = 1e-300_f64.into();
    assert!((x.hi() - 1e-300).abs() < 1e-310);
}

// =============================================================================
// From<f32> for DoubleFloat
// =============================================================================

#[test]
fn test_from_f32_positive() {
    let x: DoubleFloat = 42.5_f32.into();
    assert!((x.hi() - 42.5).abs() < EPSILON);
}

#[test]
fn test_from_f32_negative() {
    let x: DoubleFloat = (-42.5_f32).into();
    assert!((x.hi() - (-42.5)).abs() < EPSILON);
}

#[test]
fn test_from_f32_zero() {
    let x: DoubleFloat = 0.0_f32.into();
    assert_eq!(x.hi(), 0.0);
}

// =============================================================================
// From<i32> for DoubleFloat
// =============================================================================

#[test]
fn test_from_i32_positive() {
    let x: DoubleFloat = 42_i32.into();
    assert!((x.hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_from_i32_negative() {
    let x: DoubleFloat = (-42_i32).into();
    assert!((x.hi() - (-42.0)).abs() < EPSILON);
}

#[test]
fn test_from_i32_zero() {
    let x: DoubleFloat = 0_i32.into();
    assert_eq!(x.hi(), 0.0);
}

#[test]
fn test_from_i32_max() {
    let x: DoubleFloat = i32::MAX.into();
    assert!((x.hi() - (i32::MAX as f64)).abs() < EPSILON);
}

#[test]
fn test_from_i32_min() {
    let x: DoubleFloat = i32::MIN.into();
    assert!((x.hi() - (i32::MIN as f64)).abs() < EPSILON);
}

// =============================================================================
// From<i64> for DoubleFloat
// =============================================================================

#[test]
fn test_from_i64_positive() {
    let x: DoubleFloat = 42_i64.into();
    assert!((x.hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_from_i64_negative() {
    let x: DoubleFloat = (-42_i64).into();
    assert!((x.hi() - (-42.0)).abs() < EPSILON);
}

#[test]
fn test_from_i64_zero() {
    let x: DoubleFloat = 0_i64.into();
    assert_eq!(x.hi(), 0.0);
}

// =============================================================================
// From<u32> for DoubleFloat
// =============================================================================

#[test]
fn test_from_u32_positive() {
    let x: DoubleFloat = 42_u32.into();
    assert!((x.hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_from_u32_zero() {
    let x: DoubleFloat = 0_u32.into();
    assert_eq!(x.hi(), 0.0);
}

#[test]
fn test_from_u32_max() {
    let x: DoubleFloat = u32::MAX.into();
    assert!((x.hi() - (u32::MAX as f64)).abs() < EPSILON);
}

// =============================================================================
// From<u64> for DoubleFloat
// =============================================================================

#[test]
fn test_from_u64_positive() {
    let x: DoubleFloat = 42_u64.into();
    assert!((x.hi() - 42.0).abs() < EPSILON);
}

#[test]
fn test_from_u64_zero() {
    let x: DoubleFloat = 0_u64.into();
    assert_eq!(x.hi(), 0.0);
}

// =============================================================================
// From<DoubleFloat> for f64
// =============================================================================

#[test]
fn test_to_f64_positive() {
    let x = DoubleFloat::from_f64(42.5);
    let y: f64 = x.into();
    assert!((y - 42.5).abs() < EPSILON);
}

#[test]
fn test_to_f64_negative() {
    let x = DoubleFloat::from_f64(-42.5);
    let y: f64 = x.into();
    assert!((y - (-42.5)).abs() < EPSILON);
}

#[test]
fn test_to_f64_zero() {
    let x = DoubleFloat::from_f64(0.0);
    let y: f64 = x.into();
    assert_eq!(y, 0.0);
}

// =============================================================================
// From<DoubleFloat> for f32
// =============================================================================

#[test]
fn test_to_f32_positive() {
    let x = DoubleFloat::from_f64(42.5);
    let y: f32 = x.into();
    assert!((y - 42.5_f32).abs() < 1e-6);
}

#[test]
fn test_to_f32_negative() {
    let x = DoubleFloat::from_f64(-42.5);
    let y: f32 = x.into();
    assert!((y - (-42.5_f32)).abs() < 1e-6);
}

#[test]
fn test_to_f32_zero() {
    let x = DoubleFloat::from_f64(0.0);
    let y: f32 = x.into();
    assert_eq!(y, 0.0_f32);
}
