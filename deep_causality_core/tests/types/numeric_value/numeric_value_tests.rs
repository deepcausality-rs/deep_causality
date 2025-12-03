/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::NumericValue;

#[test]
fn test_default() {
    let val = NumericValue::default();
    assert!(matches!(val, NumericValue::None));
}

#[test]
fn test_from_u8() {
    let val: NumericValue = 42u8.into();
    assert_eq!(val, NumericValue::U8(42));
}

#[test]
fn test_from_u16() {
    let val: NumericValue = 1000u16.into();
    assert_eq!(val, NumericValue::U16(1000));
}

#[test]
fn test_from_u32() {
    let val: NumericValue = 100000u32.into();
    assert_eq!(val, NumericValue::U32(100000));
}

#[test]
fn test_from_u64() {
    let val: NumericValue = 10000000000u64.into();
    assert_eq!(val, NumericValue::U64(10000000000));
}

#[test]
fn test_from_u128() {
    let val: NumericValue = 100000000000000000000u128.into();
    assert_eq!(val, NumericValue::U128(100000000000000000000));
}

#[test]
fn test_from_i8() {
    let val: NumericValue = (-42i8).into();
    assert_eq!(val, NumericValue::I8(-42));
}

#[test]
fn test_from_i16() {
    let val: NumericValue = (-1000i16).into();
    assert_eq!(val, NumericValue::I16(-1000));
}

#[test]
fn test_from_i32() {
    let val: NumericValue = (-100000i32).into();
    assert_eq!(val, NumericValue::I32(-100000));
}

#[test]
fn test_from_i64() {
    let val: NumericValue = (-10000000000i64).into();
    assert_eq!(val, NumericValue::I64(-10000000000));
}

#[test]
fn test_from_i128() {
    let val: NumericValue = (-100000000000000000000i128).into();
    assert_eq!(val, NumericValue::I128(-100000000000000000000));
}

#[test]
fn test_from_f32() {
    let val: NumericValue = std::f32::consts::PI.into();
    if let NumericValue::F32(f) = val {
        assert!((f - std::f32::consts::PI).abs() < 0.001);
    } else {
        panic!("Expected F32 variant");
    }
}

#[test]
fn test_from_f64() {
    let val: NumericValue = std::f64::consts::PI.into();
    if let NumericValue::F64(f) = val {
        assert!((f - std::f64::consts::PI).abs() < 0.00001);
    } else {
        panic!("Expected F64 variant");
    }
}

#[test]
fn test_partial_eq() {
    assert_eq!(NumericValue::U8(42), NumericValue::U8(42));
    assert_ne!(NumericValue::U8(42), NumericValue::U8(43));
    assert_ne!(NumericValue::U8(42), NumericValue::I32(42));
}

#[test]
fn test_partial_ord() {
    assert!(NumericValue::U8(1) < NumericValue::U8(2));
    assert!(NumericValue::I32(-10) < NumericValue::I32(10));
}
