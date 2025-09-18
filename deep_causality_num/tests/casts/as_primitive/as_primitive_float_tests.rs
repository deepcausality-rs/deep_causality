/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::AsPrimitive;

macro_rules! test_float_as {
    ($test_name:ident, $from_type:ty, $to_type:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $test_name() {
            let val: $from_type = $val;
            let expected: $to_type = $expected;
            assert_eq!(AsPrimitive::<$to_type>::as_(val), expected);
        }
    };
}

mod f32_tests {
    use super::*;

    // f32 -> u8
    test_float_as!(to_u8_truncate, f32, u8, 42.7, 42);
    test_float_as!(to_u8_saturate_pos, f32, u8, 300.0, u8::MAX);
    test_float_as!(to_u8_saturate_neg, f32, u8, -1.0, 0);
    test_float_as!(to_u8_nan, f32, u8, f32::NAN, 0);
    test_float_as!(to_u8_inf, f32, u8, f32::INFINITY, u8::MAX);
    test_float_as!(to_u8_neg_inf, f32, u8, f32::NEG_INFINITY, 0);

    // f32 -> u16
    test_float_as!(to_u16_truncate, f32, u16, 42.7, 42);
    test_float_as!(to_u16_saturate_pos, f32, u16, 70000.0, u16::MAX);
    test_float_as!(to_u16_saturate_neg, f32, u16, -1.0, 0);
    test_float_as!(to_u16_nan, f32, u16, f32::NAN, 0);
    test_float_as!(to_u16_inf, f32, u16, f32::INFINITY, u16::MAX);
    test_float_as!(to_u16_neg_inf, f32, u16, f32::NEG_INFINITY, 0);

    // f32 -> u32
    test_float_as!(to_u32_truncate, f32, u32, 42.7, 42);
    test_float_as!(to_u32_saturate_pos, f32, u32, 4294967296.0, u32::MAX);
    test_float_as!(to_u32_saturate_neg, f32, u32, -1.0, 0);
    test_float_as!(to_u32_nan, f32, u32, f32::NAN, 0);
    test_float_as!(to_u32_inf, f32, u32, f32::INFINITY, u32::MAX);
    test_float_as!(to_u32_neg_inf, f32, u32, f32::NEG_INFINITY, 0);

    // f32 -> u64
    test_float_as!(to_u64_truncate, f32, u64, 42.7, 42);
    test_float_as!(to_u64_saturate_pos, f32, u64, f32::MAX, u64::MAX);
    test_float_as!(to_u64_saturate_neg, f32, u64, -1.0, 0);
    test_float_as!(to_u64_nan, f32, u64, f32::NAN, 0);
    test_float_as!(to_u64_inf, f32, u64, f32::INFINITY, u64::MAX);
    test_float_as!(to_u64_neg_inf, f32, u64, f32::NEG_INFINITY, 0);

    // f32 -> u128
    test_float_as!(to_u128_truncate, f32, u128, 42.7, 42);
    test_float_as!(
        to_u128_f32_max,
        f32,
        u128,
        f32::MAX,
        340282346638528859811704183484516925440_u128
    );
    test_float_as!(to_u128_saturate_neg, f32, u128, -1.0, 0);
    test_float_as!(to_u128_nan, f32, u128, f32::NAN, 0);
    test_float_as!(to_u128_inf, f32, u128, f32::INFINITY, u128::MAX);
    test_float_as!(to_u128_neg_inf, f32, u128, f32::NEG_INFINITY, 0);

    // f32 -> usize
    test_float_as!(to_usize_truncate, f32, usize, 42.7, 42);
    test_float_as!(to_usize_saturate_pos, f32, usize, f32::MAX, usize::MAX);
    test_float_as!(to_usize_saturate_neg, f32, usize, -1.0, 0);
    test_float_as!(to_usize_nan, f32, usize, f32::NAN, 0);
    test_float_as!(to_usize_inf, f32, usize, f32::INFINITY, usize::MAX);
    test_float_as!(to_usize_neg_inf, f32, usize, f32::NEG_INFINITY, 0);

    // f32 -> i8
    test_float_as!(to_i8_truncate, f32, i8, 42.7, 42);
    test_float_as!(to_i8_truncate_neg, f32, i8, -42.7, -42);
    test_float_as!(to_i8_saturate_pos, f32, i8, 130.0, i8::MAX);
    test_float_as!(to_i8_saturate_neg, f32, i8, -130.0, i8::MIN);
    test_float_as!(to_i8_nan, f32, i8, f32::NAN, 0);
    test_float_as!(to_i8_inf, f32, i8, f32::INFINITY, i8::MAX);
    test_float_as!(to_i8_neg_inf, f32, i8, f32::NEG_INFINITY, i8::MIN);

    // f32 -> i16
    test_float_as!(to_i16_truncate, f32, i16, 42.7, 42);
    test_float_as!(to_i16_truncate_neg, f32, i16, -42.7, -42);
    test_float_as!(to_i16_saturate_pos, f32, i16, 32768.0, i16::MAX);
    test_float_as!(to_i16_saturate_neg, f32, i16, -32769.0, i16::MIN);
    test_float_as!(to_i16_nan, f32, i16, f32::NAN, 0);
    test_float_as!(to_i16_inf, f32, i16, f32::INFINITY, i16::MAX);
    test_float_as!(to_i16_neg_inf, f32, i16, f32::NEG_INFINITY, i16::MIN);

    // f32 -> i32
    test_float_as!(to_i32_truncate, f32, i32, 42.7, 42);
    test_float_as!(to_i32_truncate_neg, f32, i32, -42.7, -42);
    test_float_as!(to_i32_saturate_pos, f32, i32, 2147483648.0, i32::MAX);
    test_float_as!(to_i32_saturate_neg, f32, i32, -2147483649.0, i32::MIN);
    test_float_as!(to_i32_nan, f32, i32, f32::NAN, 0);
    test_float_as!(to_i32_inf, f32, i32, f32::INFINITY, i32::MAX);
    test_float_as!(to_i32_neg_inf, f32, i32, f32::NEG_INFINITY, i32::MIN);

    // f32 -> i64
    test_float_as!(to_i64_truncate, f32, i64, 42.7, 42);
    test_float_as!(to_i64_truncate_neg, f32, i64, -42.7, -42);
    test_float_as!(to_i64_saturate_pos, f32, i64, f32::MAX, i64::MAX);
    test_float_as!(to_i64_saturate_neg, f32, i64, -f32::MAX, i64::MIN);
    test_float_as!(to_i64_nan, f32, i64, f32::NAN, 0);
    test_float_as!(to_i64_inf, f32, i64, f32::INFINITY, i64::MAX);
    test_float_as!(to_i64_neg_inf, f32, i64, f32::NEG_INFINITY, i64::MIN);

    // f32 -> i128
    test_float_as!(to_i128_truncate, f32, i128, 42.7, 42);
    test_float_as!(to_i128_truncate_neg, f32, i128, -42.7, -42);
    test_float_as!(to_i128_saturate_pos, f32, i128, f32::MAX, i128::MAX);
    test_float_as!(to_i128_saturate_neg, f32, i128, -f32::MAX, i128::MIN);
    test_float_as!(to_i128_nan, f32, i128, f32::NAN, 0);
    test_float_as!(to_i128_inf, f32, i128, f32::INFINITY, i128::MAX);
    test_float_as!(to_i128_neg_inf, f32, i128, f32::NEG_INFINITY, i128::MIN);

    // f32 -> isize
    test_float_as!(to_isize_truncate, f32, isize, 42.7, 42);
    test_float_as!(to_isize_truncate_neg, f32, isize, -42.7, -42);
    test_float_as!(to_isize_saturate_pos, f32, isize, f32::MAX, isize::MAX);
    test_float_as!(to_isize_saturate_neg, f32, isize, -f32::MAX, isize::MIN);
    test_float_as!(to_isize_nan, f32, isize, f32::NAN, 0);
    test_float_as!(to_isize_inf, f32, isize, f32::INFINITY, isize::MAX);
    test_float_as!(to_isize_neg_inf, f32, isize, f32::NEG_INFINITY, isize::MIN);

    // to float types
    test_float_as!(to_f32_self, f32, f32, 42.5, 42.5);
    test_float_as!(to_f64, f32, f64, 42.5f32, 42.5f64);
}

mod f64_tests {
    use super::*;

    // f64 -> u8
    test_float_as!(to_u8_truncate, f64, u8, 42.7, 42);
    test_float_as!(to_u8_saturate_pos, f64, u8, 300.0, u8::MAX);
    test_float_as!(to_u8_saturate_neg, f64, u8, -1.0, 0);
    test_float_as!(to_u8_nan, f64, u8, f64::NAN, 0);
    test_float_as!(to_u8_inf, f64, u8, f64::INFINITY, u8::MAX);
    test_float_as!(to_u8_neg_inf, f64, u8, f64::NEG_INFINITY, 0);

    // f64 -> u16
    test_float_as!(to_u16_truncate, f64, u16, 42.7, 42);
    test_float_as!(to_u16_saturate_pos, f64, u16, 70000.0, u16::MAX);
    test_float_as!(to_u16_saturate_neg, f64, u16, -1.0, 0);
    test_float_as!(to_u16_nan, f64, u16, f64::NAN, 0);
    test_float_as!(to_u16_inf, f64, u16, f64::INFINITY, u16::MAX);
    test_float_as!(to_u16_neg_inf, f64, u16, f64::NEG_INFINITY, 0);

    // f64 -> u32
    test_float_as!(to_u32_truncate, f64, u32, 42.7, 42);
    test_float_as!(to_u32_saturate_pos, f64, u32, 4294967296.0, u32::MAX);
    test_float_as!(to_u32_saturate_neg, f64, u32, -1.0, 0);
    test_float_as!(to_u32_nan, f64, u32, f64::NAN, 0);
    test_float_as!(to_u32_inf, f64, u32, f64::INFINITY, u32::MAX);
    test_float_as!(to_u32_neg_inf, f64, u32, f64::NEG_INFINITY, 0);

    // f64 -> u64
    test_float_as!(to_u64_truncate, f64, u64, 42.7, 42);
    test_float_as!(to_u64_saturate_pos, f64, u64, f64::MAX, u64::MAX);
    test_float_as!(to_u64_saturate_neg, f64, u64, -1.0, 0);
    test_float_as!(to_u64_nan, f64, u64, f64::NAN, 0);
    test_float_as!(to_u64_inf, f64, u64, f64::INFINITY, u64::MAX);
    test_float_as!(to_u64_neg_inf, f64, u64, f64::NEG_INFINITY, 0);

    // f64 -> u128
    test_float_as!(to_u128_truncate, f64, u128, 42.7, 42);
    test_float_as!(to_u128_saturate_pos, f64, u128, f64::MAX, u128::MAX);
    test_float_as!(to_u128_saturate_neg, f64, u128, -1.0, 0);
    test_float_as!(to_u128_nan, f64, u128, f64::NAN, 0);
    test_float_as!(to_u128_inf, f64, u128, f64::INFINITY, u128::MAX);
    test_float_as!(to_u128_neg_inf, f64, u128, f64::NEG_INFINITY, 0);

    // f64 -> usize
    test_float_as!(to_usize_truncate, f64, usize, 42.7, 42);
    test_float_as!(to_usize_saturate_pos, f64, usize, f64::MAX, usize::MAX);
    test_float_as!(to_usize_saturate_neg, f64, usize, -1.0, 0);
    test_float_as!(to_usize_nan, f64, usize, f64::NAN, 0);
    test_float_as!(to_usize_inf, f64, usize, f64::INFINITY, usize::MAX);
    test_float_as!(to_usize_neg_inf, f64, usize, f64::NEG_INFINITY, 0);

    // f64 -> i8
    test_float_as!(to_i8_truncate, f64, i8, 42.7, 42);
    test_float_as!(to_i8_truncate_neg, f64, i8, -42.7, -42);
    test_float_as!(to_i8_saturate_pos, f64, i8, 130.0, i8::MAX);
    test_float_as!(to_i8_saturate_neg, f64, i8, -130.0, i8::MIN);
    test_float_as!(to_i8_nan, f64, i8, f64::NAN, 0);
    test_float_as!(to_i8_inf, f64, i8, f64::INFINITY, i8::MAX);
    test_float_as!(to_i8_neg_inf, f64, i8, f64::NEG_INFINITY, i8::MIN);

    // f64 -> i16
    test_float_as!(to_i16_truncate, f64, i16, 42.7, 42);
    test_float_as!(to_i16_truncate_neg, f64, i16, -42.7, -42);
    test_float_as!(to_i16_saturate_pos, f64, i16, 32768.0, i16::MAX);
    test_float_as!(to_i16_saturate_neg, f64, i16, -32769.0, i16::MIN);
    test_float_as!(to_i16_nan, f64, i16, f64::NAN, 0);
    test_float_as!(to_i16_inf, f64, i16, f64::INFINITY, i16::MAX);
    test_float_as!(to_i16_neg_inf, f64, i16, f64::NEG_INFINITY, i16::MIN);

    // f64 -> i32
    test_float_as!(to_i32_truncate, f64, i32, 42.7, 42);
    test_float_as!(to_i32_truncate_neg, f64, i32, -42.7, -42);
    test_float_as!(to_i32_saturate_pos, f64, i32, 2147483648.0, i32::MAX);
    test_float_as!(to_i32_saturate_neg, f64, i32, -2147483649.0, i32::MIN);
    test_float_as!(to_i32_nan, f64, i32, f64::NAN, 0);
    test_float_as!(to_i32_inf, f64, i32, f64::INFINITY, i32::MAX);
    test_float_as!(to_i32_neg_inf, f64, i32, f64::NEG_INFINITY, i32::MIN);

    // f64 -> i64
    test_float_as!(to_i64_truncate, f64, i64, 42.7, 42);
    test_float_as!(to_i64_truncate_neg, f64, i64, -42.7, -42);
    test_float_as!(to_i64_saturate_pos, f64, i64, f64::MAX, i64::MAX);
    test_float_as!(to_i64_saturate_neg, f64, i64, f64::MIN, i64::MIN);
    test_float_as!(to_i64_nan, f64, i64, f64::NAN, 0);
    test_float_as!(to_i64_inf, f64, i64, f64::INFINITY, i64::MAX);
    test_float_as!(to_i64_neg_inf, f64, i64, f64::NEG_INFINITY, i64::MIN);

    // f64 -> i128
    test_float_as!(to_i128_truncate, f64, i128, 42.7, 42);
    test_float_as!(to_i128_truncate_neg, f64, i128, -42.7, -42);
    test_float_as!(to_i128_saturate_pos, f64, i128, f64::MAX, i128::MAX);
    test_float_as!(to_i128_saturate_neg, f64, i128, f64::MIN, i128::MIN);
    test_float_as!(to_i128_nan, f64, i128, f64::NAN, 0);
    test_float_as!(to_i128_inf, f64, i128, f64::INFINITY, i128::MAX);
    test_float_as!(to_i128_neg_inf, f64, i128, f64::NEG_INFINITY, i128::MIN);

    // f64 -> isize
    test_float_as!(to_isize_truncate, f64, isize, 42.7, 42);
    test_float_as!(to_isize_truncate_neg, f64, isize, -42.7, -42);
    test_float_as!(to_isize_saturate_pos, f64, isize, f64::MAX, isize::MAX);
    test_float_as!(to_isize_saturate_neg, f64, isize, f64::MIN, isize::MIN);
    test_float_as!(to_isize_nan, f64, isize, f64::NAN, 0);
    test_float_as!(to_isize_inf, f64, isize, f64::INFINITY, isize::MAX);
    test_float_as!(to_isize_neg_inf, f64, isize, f64::NEG_INFINITY, isize::MIN);

    // to float types
    test_float_as!(to_f64_self, f64, f64, 42.5, 42.5);
    test_float_as!(to_f32, f64, f32, 42.5, 42.5f32);
    test_float_as!(to_f32_inf, f64, f32, f64::MAX, f32::INFINITY);
}
