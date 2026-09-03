/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for ToPrimitive, FromPrimitive, NumCast, and iterator traits on DoubleFloat.

use deep_causality_num::{Float106, FromPrimitive, ToPrimitive};

// =============================================================================
// ToPrimitive Tests - Signed Integers
// =============================================================================

#[test]
fn test_to_isize() {
    let x = Float106::from(42.0);
    assert_eq!(ToPrimitive::to_isize(&x), Some(42));
}

#[test]
fn test_to_isize_negative() {
    let x = Float106::from(-42.0);
    assert_eq!(ToPrimitive::to_isize(&x), Some(-42));
}

#[test]
fn test_to_i8() {
    let x = Float106::from(42.0);
    assert_eq!(ToPrimitive::to_i8(&x), Some(42));
}

#[test]
fn test_to_i8_negative() {
    let x = Float106::from(-42.0);
    assert_eq!(ToPrimitive::to_i8(&x), Some(-42));
}

#[test]
fn test_to_i16() {
    let x = Float106::from(1000.0);
    assert_eq!(ToPrimitive::to_i16(&x), Some(1000));
}

#[test]
fn test_to_i32() {
    let x = Float106::from(100000.0);
    assert_eq!(ToPrimitive::to_i32(&x), Some(100000));
}

#[test]
fn test_to_i64() {
    let x = Float106::from(1e15);
    assert_eq!(ToPrimitive::to_i64(&x), Some(1000000000000000));
}

#[test]
fn test_to_i128() {
    let x = Float106::from(1e15);
    assert_eq!(ToPrimitive::to_i128(&x), Some(1000000000000000));
}

// =============================================================================
// ToPrimitive Tests - Unsigned Integers
// =============================================================================

#[test]
fn test_to_usize() {
    let x = Float106::from(42.0);
    assert_eq!(ToPrimitive::to_usize(&x), Some(42));
}

#[test]
fn test_to_u8() {
    let x = Float106::from(200.0);
    assert_eq!(ToPrimitive::to_u8(&x), Some(200));
}

#[test]
fn test_to_u16() {
    let x = Float106::from(60000.0);
    assert_eq!(ToPrimitive::to_u16(&x), Some(60000));
}

#[test]
fn test_to_u32() {
    let x = Float106::from(4000000000.0);
    assert_eq!(ToPrimitive::to_u32(&x), Some(4000000000));
}

#[test]
fn test_to_u64() {
    let x = Float106::from(1e15);
    assert_eq!(ToPrimitive::to_u64(&x), Some(1000000000000000));
}

#[test]
fn test_to_u128() {
    let x = Float106::from(1e15);
    assert_eq!(ToPrimitive::to_u128(&x), Some(1000000000000000));
}

// =============================================================================
// ToPrimitive Tests - Floats
// =============================================================================

#[test]
fn test_to_f32() {
    let x = Float106::from(42.5);
    assert_eq!(ToPrimitive::to_f32(&x), Some(42.5_f32));
}

#[test]
fn test_to_f64() {
    let x = Float106::from(42.5);
    assert_eq!(ToPrimitive::to_f64(&x), Some(42.5_f64));
}

// =============================================================================
// FromPrimitive Tests - Signed Integers
// =============================================================================

#[test]
fn test_from_i64() {
    let x = Float106::from_i64(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_i64_negative() {
    let x = Float106::from_i64(-42).unwrap();
    assert_eq!(x.hi(), -42.0);
}

#[test]
fn test_from_isize() {
    let x = Float106::from_isize(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_i8() {
    let x = Float106::from_i8(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_i16() {
    let x = Float106::from_i16(1000).unwrap();
    assert_eq!(x.hi(), 1000.0);
}

#[test]
fn test_from_i32() {
    let x = Float106::from_i32(100000).unwrap();
    assert_eq!(x.hi(), 100000.0);
}

#[test]
fn test_from_i128() {
    let x = Float106::from_i128(1000000).unwrap();
    assert_eq!(x.hi(), 1000000.0);
}

// =============================================================================
// FromPrimitive Tests - Unsigned Integers
// =============================================================================

#[test]
fn test_from_u64() {
    let x = Float106::from_u64(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_usize() {
    let x = Float106::from_usize(42).unwrap();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_from_u8() {
    let x = Float106::from_u8(200).unwrap();
    assert_eq!(x.hi(), 200.0);
}

#[test]
fn test_from_u16() {
    let x = Float106::from_u16(60000).unwrap();
    assert_eq!(x.hi(), 60000.0);
}

#[test]
fn test_from_u32() {
    let x = Float106::from_u32(100000).unwrap();
    assert_eq!(x.hi(), 100000.0);
}

#[test]
fn test_from_u128() {
    let x = Float106::from_u128(1000000).unwrap();
    assert_eq!(x.hi(), 1000000.0);
}

// =============================================================================
// FromPrimitive Tests - Floats
// =============================================================================

#[test]
fn test_from_f32() {
    let x = Float106::from_f32(42.5_f32).unwrap();
    assert!((x.hi() - 42.5).abs() < 1e-6);
}

#[test]
fn test_from_f64_primitive() {
    let x = <Float106 as FromPrimitive>::from_f64(42.5).unwrap();
    assert_eq!(x.hi(), 42.5);
}

// =============================================================================
// Sum Trait Tests
// =============================================================================

#[test]
fn test_sum_owned() {
    let values = vec![
        Float106::from(1.0),
        Float106::from(2.0),
        Float106::from(3.0),
    ];
    let sum: Float106 = values.into_iter().sum();
    assert!((sum.hi() - 6.0).abs() < 1e-14);
}

#[test]
fn test_sum_borrowed() {
    let values = [
        Float106::from(1.0),
        Float106::from(2.0),
        Float106::from(3.0),
    ];
    let sum: Float106 = values.iter().sum();
    assert!((sum.hi() - 6.0).abs() < 1e-14);
}

#[test]
fn test_sum_empty() {
    let values: Vec<Float106> = vec![];
    let sum: Float106 = values.into_iter().sum();
    assert_eq!(sum.hi(), 0.0);
}

// =============================================================================
// Product Trait Tests
// =============================================================================

#[test]
fn test_product_owned() {
    let values = vec![
        Float106::from(2.0),
        Float106::from(3.0),
        Float106::from(4.0),
    ];
    let product: Float106 = values.into_iter().product();
    assert!((product.hi() - 24.0).abs() < 1e-14);
}

#[test]
fn test_product_borrowed() {
    let values = [
        Float106::from(2.0),
        Float106::from(3.0),
        Float106::from(4.0),
    ];
    let product: Float106 = values.iter().product();
    assert!((product.hi() - 24.0).abs() < 1e-14);
}

#[test]
fn test_product_empty() {
    let values: Vec<Float106> = vec![];
    let product: Float106 = values.into_iter().product();
    assert_eq!(product.hi(), 1.0);
}

// The integer crossings are exact past 2⁵³ in both directions.

#[test]
fn test_from_u64_keeps_every_bit() {
    let just_past = (1u64 << 53) + 1;
    let x = Float106::from_u64(just_past).unwrap();
    assert_eq!(x.to_u64(), Some(just_past));
    let max = Float106::from_u64(u64::MAX).unwrap();
    assert_eq!(max.to_u64(), Some(u64::MAX));
    assert_eq!(max.to_u128(), Some(u64::MAX as u128));
    assert_eq!(Float106::from_u64(0).unwrap().to_u64(), Some(0));
}

#[test]
fn test_from_i64_keeps_every_bit_and_the_sign() {
    for n in [
        i64::MIN,
        i64::MIN + 1,
        -((1i64 << 53) + 1),
        -1,
        0,
        1,
        (1i64 << 53) + 1,
        i64::MAX,
    ] {
        let x = Float106::from_i64(n).unwrap();
        assert_eq!(x.to_i64(), Some(n), "{n}");
        assert_eq!(x.to_i128(), Some(n as i128), "{n}");
    }
}

#[test]
fn test_from_u128_is_exact_below_two_to_the_106() {
    for n in [(1u128 << 100) + 1, (1u128 << 105) + 12345, 1u128 << 106] {
        let x = Float106::from_u128(n).unwrap();
        assert_eq!(x.to_u128(), Some(n), "{n}");
    }
    let x = Float106::from_u128(u128::MAX).unwrap();
    assert!(ToPrimitive::to_f64(&x).unwrap() >= 3.4e38);
    assert_eq!(
        Float106::from_i128(i128::MIN).unwrap().to_i128(),
        Some(i128::MIN)
    );
}

#[test]
fn test_integer_conversion_truncates_the_whole_value() {
    // hi = 3, lo just below zero: the value is below 3 and truncates to 2.
    let x = Float106::new(3.0, -1e-19);
    assert_eq!(x.to_u64(), Some(2));
    assert_eq!(x.to_i32(), Some(2));
    // hi = 2, lo just above one: the value is 3.0000…1 and truncates to 3.
    let y = Float106::new(2.0, 1.0 + 1e-17);
    assert_eq!(y.to_u64(), Some(3));
    assert_eq!(Float106::from(-0.5).to_i8(), Some(0));
    assert_eq!(Float106::from(-1.5).to_u8(), None);
    assert_eq!(Float106::from(300.0).to_u8(), None);
}

#[test]
fn test_to_f64_carries_the_low_half() {
    let y = Float106::new(1.0, 2e-16);
    assert_eq!(ToPrimitive::to_f64(&y), Some(1.0 + 2e-16));
    assert_eq!(ToPrimitive::to_f64(&Float106::new(1.0, 1e-17)), Some(1.0));
}
