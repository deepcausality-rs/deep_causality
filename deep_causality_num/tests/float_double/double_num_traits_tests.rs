/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for ToPrimitive, FromPrimitive, NumCast, and iterator traits on DoubleFloat.

use deep_causality_num::{DoubleFloat, FromPrimitive, ToPrimitive};

// =============================================================================
// ToPrimitive Tests - Signed Integers
// =============================================================================

#[test]
fn test_to_isize() {
    let x = DoubleFloat::from_f64(42.0);
    assert_eq!(ToPrimitive::to_isize(&x), Some(42));
}

#[test]
fn test_to_isize_negative() {
    let x = DoubleFloat::from_f64(-42.0);
    assert_eq!(ToPrimitive::to_isize(&x), Some(-42));
}

#[test]
fn test_to_i8() {
    let x = DoubleFloat::from_f64(42.0);
    assert_eq!(ToPrimitive::to_i8(&x), Some(42));
}

#[test]
fn test_to_i8_negative() {
    let x = DoubleFloat::from_f64(-42.0);
    assert_eq!(ToPrimitive::to_i8(&x), Some(-42));
}

#[test]
fn test_to_i16() {
    let x = DoubleFloat::from_f64(1000.0);
    assert_eq!(ToPrimitive::to_i16(&x), Some(1000));
}

#[test]
fn test_to_i32() {
    let x = DoubleFloat::from_f64(100000.0);
    assert_eq!(ToPrimitive::to_i32(&x), Some(100000));
}

#[test]
fn test_to_i64() {
    let x = DoubleFloat::from_f64(1e15);
    assert_eq!(ToPrimitive::to_i64(&x), Some(1000000000000000));
}

#[test]
fn test_to_i128() {
    let x = DoubleFloat::from_f64(1e15);
    assert_eq!(ToPrimitive::to_i128(&x), Some(1000000000000000));
}

// =============================================================================
// ToPrimitive Tests - Unsigned Integers
// =============================================================================

#[test]
fn test_to_usize() {
    let x = DoubleFloat::from_f64(42.0);
    assert_eq!(ToPrimitive::to_usize(&x), Some(42));
}

#[test]
fn test_to_u8() {
    let x = DoubleFloat::from_f64(200.0);
    assert_eq!(ToPrimitive::to_u8(&x), Some(200));
}

#[test]
fn test_to_u16() {
    let x = DoubleFloat::from_f64(60000.0);
    assert_eq!(ToPrimitive::to_u16(&x), Some(60000));
}

#[test]
fn test_to_u32() {
    let x = DoubleFloat::from_f64(4000000000.0);
    assert_eq!(ToPrimitive::to_u32(&x), Some(4000000000));
}

#[test]
fn test_to_u64() {
    let x = DoubleFloat::from_f64(1e15);
    assert_eq!(ToPrimitive::to_u64(&x), Some(1000000000000000));
}

#[test]
fn test_to_u128() {
    let x = DoubleFloat::from_f64(1e15);
    assert_eq!(ToPrimitive::to_u128(&x), Some(1000000000000000));
}

// =============================================================================
// ToPrimitive Tests - Floats
// =============================================================================

#[test]
fn test_to_f32() {
    let x = DoubleFloat::from_f64(42.5);
    assert_eq!(ToPrimitive::to_f32(&x), Some(42.5_f32));
}

#[test]
fn test_to_f64() {
    let x = DoubleFloat::from_f64(42.5);
    assert_eq!(ToPrimitive::to_f64(&x), Some(42.5_f64));
}

// =============================================================================
// FromPrimitive Tests - Signed Integers
// =============================================================================

#[test]
fn test_from_i64() {
    let x = DoubleFloat::from_i64(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_i64_negative() {
    let x = DoubleFloat::from_i64(-42).unwrap();
    assert_eq!(x.hi(), -42.0);
}

#[test]
fn test_from_isize() {
    let x = DoubleFloat::from_isize(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_i8() {
    let x = DoubleFloat::from_i8(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_i16() {
    let x = DoubleFloat::from_i16(1000).unwrap();
    assert_eq!(x.hi(), 1000.0);
}

#[test]
fn test_from_i32() {
    let x = DoubleFloat::from_i32(100000).unwrap();
    assert_eq!(x.hi(), 100000.0);
}

#[test]
fn test_from_i128() {
    let x = DoubleFloat::from_i128(1000000).unwrap();
    assert_eq!(x.hi(), 1000000.0);
}

// =============================================================================
// FromPrimitive Tests - Unsigned Integers
// =============================================================================

#[test]
fn test_from_u64() {
    let x = DoubleFloat::from_u64(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_usize() {
    let x = DoubleFloat::from_usize(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_u8() {
    let x = DoubleFloat::from_u8(200).unwrap();
    assert_eq!(x.hi(), 200.0);
}

#[test]
fn test_from_u16() {
    let x = DoubleFloat::from_u16(60000).unwrap();
    assert_eq!(x.hi(), 60000.0);
}

#[test]
fn test_from_u32() {
    let x = DoubleFloat::from_u32(100000).unwrap();
    assert_eq!(x.hi(), 100000.0);
}

#[test]
fn test_from_u128() {
    let x = DoubleFloat::from_u128(1000000).unwrap();
    assert_eq!(x.hi(), 1000000.0);
}

// =============================================================================
// FromPrimitive Tests - Floats
// =============================================================================

#[test]
fn test_from_f32() {
    let x = DoubleFloat::from_f32(42.5_f32).unwrap();
    assert!((x.hi() - 42.5).abs() < 1e-6);
}

#[test]
fn test_from_f64_primitive() {
    let x = <DoubleFloat as FromPrimitive>::from_f64(42.5).unwrap();
    assert_eq!(x.hi(), 42.5);
}

// =============================================================================
// Sum Trait Tests
// =============================================================================

#[test]
fn test_sum_owned() {
    let values = vec![
        DoubleFloat::from_f64(1.0),
        DoubleFloat::from_f64(2.0),
        DoubleFloat::from_f64(3.0),
    ];
    let sum: DoubleFloat = values.into_iter().sum();
    assert!((sum.hi() - 6.0).abs() < 1e-14);
}

#[test]
fn test_sum_borrowed() {
    let values = vec![
        DoubleFloat::from_f64(1.0),
        DoubleFloat::from_f64(2.0),
        DoubleFloat::from_f64(3.0),
    ];
    let sum: DoubleFloat = values.iter().sum();
    assert!((sum.hi() - 6.0).abs() < 1e-14);
}

#[test]
fn test_sum_empty() {
    let values: Vec<DoubleFloat> = vec![];
    let sum: DoubleFloat = values.into_iter().sum();
    assert_eq!(sum.hi(), 0.0);
}

// =============================================================================
// Product Trait Tests
// =============================================================================

#[test]
fn test_product_owned() {
    let values = vec![
        DoubleFloat::from_f64(2.0),
        DoubleFloat::from_f64(3.0),
        DoubleFloat::from_f64(4.0),
    ];
    let product: DoubleFloat = values.into_iter().product();
    assert!((product.hi() - 24.0).abs() < 1e-14);
}

#[test]
fn test_product_borrowed() {
    let values = vec![
        DoubleFloat::from_f64(2.0),
        DoubleFloat::from_f64(3.0),
        DoubleFloat::from_f64(4.0),
    ];
    let product: DoubleFloat = values.iter().product();
    assert!((product.hi() - 24.0).abs() < 1e-14);
}

#[test]
fn test_product_empty() {
    let values: Vec<DoubleFloat> = vec![];
    let product: DoubleFloat = values.into_iter().product();
    assert_eq!(product.hi(), 1.0);
}
