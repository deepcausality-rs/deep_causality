/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::ToPrimitive;
use std::num::Wrapping;

macro_rules! test_wrapping_to {
    ($name:ident, $method:ident, $from_ty:ty, $to_ty:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v = Wrapping(<$from_ty>::from($val));
            let expected: Option<$to_ty> = $expected;
            let actual = v.$method();
            assert_eq!(actual, expected);
        }
    };
}

mod wrapping_i32_to_tests {
    use super::*;

    // Test successful conversions from Wrapping<i32>
    test_wrapping_to!(to_isize_ok, to_isize, i32, isize, 42, Some(42));
    test_wrapping_to!(to_i8_ok, to_i8, i32, i8, 42, Some(42));
    test_wrapping_to!(to_i16_ok, to_i16, i32, i16, 42, Some(42));
    test_wrapping_to!(to_i32_ok, to_i32, i32, i32, 42, Some(42));
    test_wrapping_to!(to_i64_ok, to_i64, i32, i64, 42, Some(42));
    test_wrapping_to!(to_i128_ok, to_i128, i32, i128, 42, Some(42));

    test_wrapping_to!(to_usize_ok, to_usize, i32, usize, 42, Some(42));
    test_wrapping_to!(to_u8_ok, to_u8, i32, u8, 42, Some(42));
    test_wrapping_to!(to_u16_ok, to_u16, i32, u16, 42, Some(42));
    test_wrapping_to!(to_u32_ok, to_u32, i32, u32, 42, Some(42));
    test_wrapping_to!(to_u64_ok, to_u64, i32, u64, 42, Some(42));
    test_wrapping_to!(to_u128_ok, to_u128, i32, u128, 42, Some(42));

    test_wrapping_to!(to_f32_ok, to_f32, i32, f32, 42, Some(42.0));
    test_wrapping_to!(to_f64_ok, to_f64, i32, f64, 42, Some(42.0));

    // Test failing conversions from Wrapping<i32>
    test_wrapping_to!(to_i8_fail, to_i8, i32, i8, 128, None);
    test_wrapping_to!(to_u8_fail, to_u8, i32, u8, -1, None);
    test_wrapping_to!(to_u16_fail, to_u16, i32, u16, -1, None);
    test_wrapping_to!(to_u32_fail, to_u32, i32, u32, -1, None);
    test_wrapping_to!(to_u64_fail, to_u64, i32, u64, -1, None);
    test_wrapping_to!(to_u128_fail, to_u128, i32, u128, -1, None);
    test_wrapping_to!(to_usize_fail, to_usize, i32, usize, -1, None);
}
