/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NumericValue;

impl From<u8> for NumericValue {
    fn from(n: u8) -> Self {
        NumericValue::U8(n)
    }
}

impl From<u16> for NumericValue {
    fn from(n: u16) -> Self {
        NumericValue::U16(n)
    }
}

impl From<u32> for NumericValue {
    fn from(n: u32) -> Self {
        NumericValue::U32(n)
    }
}

impl From<u64> for NumericValue {
    fn from(n: u64) -> Self {
        NumericValue::U64(n)
    }
}

impl From<u128> for NumericValue {
    fn from(n: u128) -> Self {
        NumericValue::U128(n)
    }
}

impl From<i8> for NumericValue {
    fn from(n: i8) -> Self {
        NumericValue::I8(n)
    }
}

impl From<i16> for NumericValue {
    fn from(n: i16) -> Self {
        NumericValue::I16(n)
    }
}

impl From<i32> for NumericValue {
    fn from(n: i32) -> Self {
        NumericValue::I32(n)
    }
}

impl From<i64> for NumericValue {
    fn from(n: i64) -> Self {
        NumericValue::I64(n)
    }
}

impl From<i128> for NumericValue {
    fn from(n: i128) -> Self {
        NumericValue::I128(n)
    }
}

impl From<f32> for NumericValue {
    fn from(n: f32) -> Self {
        NumericValue::F32(n)
    }
}

impl From<f64> for NumericValue {
    fn from(n: f64) -> Self {
        NumericValue::F64(n)
    }
}
