/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::AsPrimitive;

macro_rules! test_int_as {
    ($test_name:ident, $from_type:ty, $to_type:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $test_name() {
            let val: $from_type = $val;
            let expected: $to_type = $expected;
            assert_eq!(AsPrimitive::<$to_type>::as_(val), expected);
        }
    };
}

mod isize_tests {
    use super::*;
    test_int_as!(to_f32, isize, f32, 42, 42.0);
    test_int_as!(to_f64, isize, f64, 42, 42.0);
    test_int_as!(to_u8, isize, u8, 42, 42);
    test_int_as!(to_u16, isize, u16, 42, 42);
    test_int_as!(to_u32, isize, u32, 42, 42);
    test_int_as!(to_u64, isize, u64, 42, 42);
    test_int_as!(to_u128, isize, u128, 42, 42);
    test_int_as!(to_usize, isize, usize, 42, 42);
    test_int_as!(to_i8, isize, i8, 42, 42);
    test_int_as!(to_i16, isize, i16, 42, 42);
    test_int_as!(to_i32, isize, i32, 42, 42);
    test_int_as!(to_i64, isize, i64, 42, 42);
    test_int_as!(to_i128, isize, i128, 42, 42);
    test_int_as!(to_isize, isize, isize, 42, 42);
}

mod i8_tests {
    use super::*;
    test_int_as!(to_f32, i8, f32, 42, 42.0);
    test_int_as!(to_f64, i8, f64, 42, 42.0);
    test_int_as!(to_u8, i8, u8, 42, 42);
    test_int_as!(to_u16, i8, u16, 42, 42);
    test_int_as!(to_u32, i8, u32, 42, 42);
    test_int_as!(to_u64, i8, u64, 42, 42);
    test_int_as!(to_u128, i8, u128, 42, 42);
    test_int_as!(to_usize, i8, usize, 42, 42);
    test_int_as!(to_i8, i8, i8, 42, 42);
    test_int_as!(to_i16, i8, i16, 42, 42);
    test_int_as!(to_i32, i8, i32, 42, 42);
    test_int_as!(to_i64, i8, i64, 42, 42);
    test_int_as!(to_i128, i8, i128, 42, 42);
    test_int_as!(to_isize, i8, isize, 42, 42);
}

mod i16_tests {
    use super::*;
    test_int_as!(to_f32, i16, f32, 42, 42.0);
    test_int_as!(to_f64, i16, f64, 42, 42.0);
    test_int_as!(to_u8, i16, u8, 42, 42);
    test_int_as!(to_u16, i16, u16, 42, 42);
    test_int_as!(to_u32, i16, u32, 42, 42);
    test_int_as!(to_u64, i16, u64, 42, 42);
    test_int_as!(to_u128, i16, u128, 42, 42);
    test_int_as!(to_usize, i16, usize, 42, 42);
    test_int_as!(to_i8, i16, i8, 42, 42);
    test_int_as!(to_i16, i16, i16, 42, 42);
    test_int_as!(to_i32, i16, i32, 42, 42);
    test_int_as!(to_i64, i16, i64, 42, 42);
    test_int_as!(to_i128, i16, i128, 42, 42);
    test_int_as!(to_isize, i16, isize, 42, 42);
}

mod i32_tests {
    use super::*;
    test_int_as!(to_f32, i32, f32, 42, 42.0);
    test_int_as!(to_f64, i32, f64, 42, 42.0);
    test_int_as!(to_u8, i32, u8, 42, 42);
    test_int_as!(to_u16, i32, u16, 42, 42);
    test_int_as!(to_u32, i32, u32, 42, 42);
    test_int_as!(to_u64, i32, u64, 42, 42);
    test_int_as!(to_u128, i32, u128, 42, 42);
    test_int_as!(to_usize, i32, usize, 42, 42);
    test_int_as!(to_i8, i32, i8, 42, 42);
    test_int_as!(to_i16, i32, i16, 42, 42);
    test_int_as!(to_i32, i32, i32, 42, 42);
    test_int_as!(to_i64, i32, i64, 42, 42);
    test_int_as!(to_i128, i32, i128, 42, 42);
    test_int_as!(to_isize, i32, isize, 42, 42);
}

mod i64_tests {
    use super::*;
    test_int_as!(to_f32, i64, f32, 42, 42.0);
    test_int_as!(to_f64, i64, f64, 42, 42.0);
    test_int_as!(to_u8, i64, u8, 42, 42);
    test_int_as!(to_u16, i64, u16, 42, 42);
    test_int_as!(to_u32, i64, u32, 42, 42);
    test_int_as!(to_u64, i64, u64, 42, 42);
    test_int_as!(to_u128, i64, u128, 42, 42);
    test_int_as!(to_usize, i64, usize, 42, 42);
    test_int_as!(to_i8, i64, i8, 42, 42);
    test_int_as!(to_i16, i64, i16, 42, 42);
    test_int_as!(to_i32, i64, i32, 42, 42);
    test_int_as!(to_i64, i64, i64, 42, 42);
    test_int_as!(to_i128, i64, i128, 42, 42);
    test_int_as!(to_isize, i64, isize, 42, 42);
}

mod i128_tests {
    use super::*;
    test_int_as!(to_f32, i128, f32, 42, 42.0);
    test_int_as!(to_f64, i128, f64, 42, 42.0);
    test_int_as!(to_u8, i128, u8, 42, 42);
    test_int_as!(to_u16, i128, u16, 42, 42);
    test_int_as!(to_u32, i128, u32, 42, 42);
    test_int_as!(to_u64, i128, u64, 42, 42);
    test_int_as!(to_u128, i128, u128, 42, 42);
    test_int_as!(to_usize, i128, usize, 42, 42);
    test_int_as!(to_i8, i128, i8, 42, 42);
    test_int_as!(to_i16, i128, i16, 42, 42);
    test_int_as!(to_i32, i128, i32, 42, 42);
    test_int_as!(to_i64, i128, i64, 42, 42);
    test_int_as!(to_i128, i128, i128, 42, 42);
    test_int_as!(to_isize, i128, isize, 42, 42);
}
