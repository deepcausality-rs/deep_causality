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

    // to integer types
    test_float_as!(to_u8_truncate, f32, u8, 42.7, 42);
    test_float_as!(to_u8_saturate_pos, f32, u8, 300.0, u8::MAX);
    test_float_as!(to_u8_saturate_neg, f32, u8, -1.0, 0);
    test_float_as!(to_u8_nan, f32, u8, f32::NAN, 0);
    test_float_as!(to_u8_inf, f32, u8, f32::INFINITY, u8::MAX);
    test_float_as!(to_u8_neg_inf, f32, u8, f32::NEG_INFINITY, 0);

    test_float_as!(to_i8_truncate, f32, i8, 42.7, 42);
    test_float_as!(to_i8_truncate_neg, f32, i8, -42.7, -42);
    test_float_as!(to_i8_saturate_pos, f32, i8, 130.0, i8::MAX);
    test_float_as!(to_i8_saturate_neg, f32, i8, -130.0, i8::MIN);
    test_float_as!(to_i8_nan, f32, i8, f32::NAN, 0);
    test_float_as!(to_i8_inf, f32, i8, f32::INFINITY, i8::MAX);
    test_float_as!(to_i8_neg_inf, f32, i8, f32::NEG_INFINITY, i8::MIN);

    test_float_as!(to_u32_truncate, f32, u32, 42.7, 42);
    test_float_as!(to_i32_truncate, f32, i32, 42.7, 42);
    test_float_as!(to_i32_truncate_neg, f32, i32, -42.7, -42);

    // to float types
    test_float_as!(to_f32_self, f32, f32, 42.5, 42.5);
    test_float_as!(to_f64, f32, f64, 42.5f32, 42.5f64);
}

mod f64_tests {
    use super::*;

    // to integer types
    test_float_as!(to_u64_truncate, f64, u64, 42.7, 42);
    test_float_as!(to_u64_saturate_pos, f64, u64, f64::MAX, u64::MAX);
    test_float_as!(to_u64_saturate_neg, f64, u64, -1.0, 0);
    test_float_as!(to_u64_nan, f64, u64, f64::NAN, 0);
    test_float_as!(to_u64_inf, f64, u64, f64::INFINITY, u64::MAX);
    test_float_as!(to_u64_neg_inf, f64, u64, f64::NEG_INFINITY, 0);

    test_float_as!(to_i64_truncate, f64, i64, 42.7, 42);
    test_float_as!(to_i64_truncate_neg, f64, i64, -42.7, -42);
    test_float_as!(to_i64_saturate_pos, f64, i64, f64::MAX, i64::MAX);
    test_float_as!(to_i64_saturate_neg, f64, i64, f64::MIN, i64::MIN);
    test_float_as!(to_i64_nan, f64, i64, f64::NAN, 0);
    test_float_as!(to_i64_inf, f64, i64, f64::INFINITY, i64::MAX);
    test_float_as!(to_i64_neg_inf, f64, i64, f64::NEG_INFINITY, i64::MIN);

    // to float types
    test_float_as!(to_f64_self, f64, f64, 42.5, 42.5);
    test_float_as!(to_f32, f64, f32, 42.5, 42.5f32);
    test_float_as!(to_f32_inf, f64, f32, f64::MAX, f32::INFINITY);
}
