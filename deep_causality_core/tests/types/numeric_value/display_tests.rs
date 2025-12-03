/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::NumericValue;

#[test]
fn test_display_none() {
    let val = NumericValue::None;
    assert_eq!(format!("{}", val), "None");
}

#[test]
fn test_display_u8() {
    let val = NumericValue::U8(123);
    assert_eq!(format!("{}", val), "U8(123)");
}

#[test]
fn test_display_u16() {
    let val = NumericValue::U16(12345);
    assert_eq!(format!("{}", val), "U16(12345)");
}

#[test]
fn test_display_u32() {
    let val = NumericValue::U32(123456789);
    assert_eq!(format!("{}", val), "U32(123456789)");
}

#[test]
fn test_display_u64() {
    let val = NumericValue::U64(1234567890123456789);
    assert_eq!(format!("{}", val), "U64(1234567890123456789)");
}

#[test]
fn test_display_u128() {
    let val = NumericValue::U128(123456789012345678901234567890123456789);
    assert_eq!(format!("{}", val), "U128(123456789012345678901234567890123456789)");
}

#[test]
fn test_display_i8() {
    let val = NumericValue::I8(-123);
    assert_eq!(format!("{}", val), "I8(-123)");
}

#[test]
fn test_display_i16() {
    let val = NumericValue::I16(-12345);
    assert_eq!(format!("{}", val), "I16(-12345)");
}

#[test]
fn test_display_i32() {
    let val = NumericValue::I32(-123456789);
    assert_eq!(format!("{}", val), "I32(-123456789)");
}

#[test]
fn test_display_i64() {
    let val = NumericValue::I64(-1234567890123456789);
    assert_eq!(format!("{}", val), "I64(-1234567890123456789)");
}

#[test]
fn test_display_i128() {
    let val = NumericValue::I128(-123456789012345678901234567890123456789);
    assert_eq!(format!("{}", val), "I128(-123456789012345678901234567890123456789)");
}

#[test]
fn test_display_f32() {
    let val = NumericValue::F32(3.14f32);
    assert_eq!(format!("{}", val), "F32(3.14)");
}

#[test]
fn test_display_f64() {
    let val = NumericValue::F64(std::f64::consts::PI);
    assert_eq!(format!("{}", val), "F64(3.141592653589793)");
}