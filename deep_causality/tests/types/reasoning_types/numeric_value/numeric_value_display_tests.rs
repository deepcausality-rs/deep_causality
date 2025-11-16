/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::NumericValue;

#[test]
fn test_display_none() {
    let value = NumericValue::None;
    assert_eq!(value.to_string(), "None");
}

#[test]
fn test_display_u8() {
    let value = NumericValue::U8(8);
    assert_eq!(value.to_string(), "U8(8)");
}

#[test]
fn test_display_u16() {
    let value = NumericValue::U16(16);
    assert_eq!(value.to_string(), "U16(16)");
}

#[test]
fn test_display_u32() {
    let value = NumericValue::U32(32);
    assert_eq!(value.to_string(), "U32(32)");
}

#[test]
fn test_display_u64() {
    let value = NumericValue::U64(64);
    assert_eq!(value.to_string(), "U64(64)");
}

#[test]
fn test_display_u128() {
    let value = NumericValue::U128(128);
    assert_eq!(value.to_string(), "U128(128)");
}

#[test]
fn test_display_i8() {
    let value = NumericValue::I8(-8);
    assert_eq!(value.to_string(), "I8(-8)");
}

#[test]
fn test_display_i16() {
    let value = NumericValue::I16(-16);
    assert_eq!(value.to_string(), "I16(-16)");
}

#[test]
fn test_display_i32() {
    let value = NumericValue::I32(-32);
    assert_eq!(value.to_string(), "I32(-32)");
}

#[test]
fn test_display_i64() {
    let value = NumericValue::I64(-64);
    assert_eq!(value.to_string(), "I64(-64)");
}

#[test]
fn test_display_i128() {
    let value = NumericValue::I128(-128);
    assert_eq!(value.to_string(), "I128(-128)");
}

#[test]
fn test_display_f32() {
    let value = NumericValue::F32(3.445);
    assert_eq!(value.to_string(), "F32(3.445)");
}

#[test]
fn test_display_f64() {
    let value = NumericValue::F64(2.776655);
    assert_eq!(value.to_string(), "F64(2.776655)");
}
