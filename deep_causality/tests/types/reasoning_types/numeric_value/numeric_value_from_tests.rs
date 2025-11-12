/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::NumericValue;

#[test]
fn test_from_u8() {
    let n: u8 = 8;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::U8(n));
}

#[test]
fn test_from_u16() {
    let n: u16 = 16;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::U16(n));
}

#[test]
fn test_from_u32() {
    let n: u32 = 32;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::U32(n));
}

#[test]
fn test_from_u64() {
    let n: u64 = 64;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::U64(n));
}

#[test]
fn test_from_u128() {
    let n: u128 = 128;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::U128(n));
}

#[test]
fn test_from_i8() {
    let n: i8 = -8;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::I8(n));
}

#[test]
fn test_from_i16() {
    let n: i16 = -16;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::I16(n));
}

#[test]
fn test_from_i32() {
    let n: i32 = -32;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::I32(n));
}

#[test]
fn test_from_i64() {
    let n: i64 = -64;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::I64(n));
}

#[test]
fn test_from_i128() {
    let n: i128 = -128;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::I128(n));
}

#[test]
fn test_from_f32() {
    let n: f32 = 3.4567;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::F32(n));
}

#[test]
fn test_from_f64() {
    let n: f64 = 2.34567;
    let numeric_value: NumericValue = n.into();
    assert_eq!(numeric_value, NumericValue::F64(n));
}
