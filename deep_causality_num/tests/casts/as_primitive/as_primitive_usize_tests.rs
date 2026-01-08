/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::AsPrimitive;

macro_rules! test_uint_as {
    ($test_name:ident, $from_type:ty, $to_type:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $test_name() {
            let val: $from_type = $val;
            let expected: $to_type = $expected;
            assert_eq!(AsPrimitive::<$to_type>::as_(val), expected);
        }
    };
}

mod u8_tests {
    use super::*;
    test_uint_as!(to_char, u8, char, 65, 'A');
    test_uint_as!(to_f32, u8, f32, 42, 42.0);
    test_uint_as!(to_f64, u8, f64, 42, 42.0);
    test_uint_as!(to_u8, u8, u8, 42, 42);
    test_uint_as!(to_u16, u8, u16, 42, 42);
    test_uint_as!(to_u32, u8, u32, 42, 42);
    test_uint_as!(to_u64, u8, u64, 42, 42);
    test_uint_as!(to_u128, u8, u128, 42, 42);
    test_uint_as!(to_usize, u8, usize, 42, 42);
    test_uint_as!(to_i8, u8, i8, 42, 42);
    test_uint_as!(to_i16, u8, i16, 42, 42);
    test_uint_as!(to_i32, u8, i32, 42, 42);
    test_uint_as!(to_i64, u8, i64, 42, 42);
    test_uint_as!(to_i128, u8, i128, 42, 42);
    test_uint_as!(to_isize, u8, isize, 42, 42);
}

mod u16_tests {
    use super::*;
    test_uint_as!(to_f32, u16, f32, 42, 42.0);
    test_uint_as!(to_f64, u16, f64, 42, 42.0);
    test_uint_as!(to_u8, u16, u8, 42, 42);
    test_uint_as!(to_u16, u16, u16, 42, 42);
    test_uint_as!(to_u32, u16, u32, 42, 42);
    test_uint_as!(to_u64, u16, u64, 42, 42);
    test_uint_as!(to_u128, u16, u128, 42, 42);
    test_uint_as!(to_usize, u16, usize, 42, 42);
    test_uint_as!(to_i8, u16, i8, 42, 42);
    test_uint_as!(to_i16, u16, i16, 42, 42);
    test_uint_as!(to_i32, u16, i32, 42, 42);
    test_uint_as!(to_i64, u16, i64, 42, 42);
    test_uint_as!(to_i128, u16, i128, 42, 42);
    test_uint_as!(to_isize, u16, isize, 42, 42);
}

mod u32_tests {
    use super::*;
    test_uint_as!(to_f32, u32, f32, 42, 42.0);
    test_uint_as!(to_f64, u32, f64, 42, 42.0);
    test_uint_as!(to_u8, u32, u8, 42, 42);
    test_uint_as!(to_u16, u32, u16, 42, 42);
    test_uint_as!(to_u32, u32, u32, 42, 42);
    test_uint_as!(to_u64, u32, u64, 42, 42);
    test_uint_as!(to_u128, u32, u128, 42, 42);
    test_uint_as!(to_usize, u32, usize, 42, 42);
    test_uint_as!(to_i8, u32, i8, 42, 42);
    test_uint_as!(to_i16, u32, i16, 42, 42);
    test_uint_as!(to_i32, u32, i32, 42, 42);
    test_uint_as!(to_i64, u32, i64, 42, 42);
    test_uint_as!(to_i128, u32, i128, 42, 42);
    test_uint_as!(to_isize, u32, isize, 42, 42);
}

mod u64_tests {
    use super::*;
    test_uint_as!(to_f32, u64, f32, 42, 42.0);
    test_uint_as!(to_f64, u64, f64, 42, 42.0);
    test_uint_as!(to_u8, u64, u8, 42, 42);
    test_uint_as!(to_u16, u64, u16, 42, 42);
    test_uint_as!(to_u32, u64, u32, 42, 42);
    test_uint_as!(to_u64, u64, u64, 42, 42);
    test_uint_as!(to_u128, u64, u128, 42, 42);
    test_uint_as!(to_usize, u64, usize, 42, 42);
    test_uint_as!(to_i8, u64, i8, 42, 42);
    test_uint_as!(to_i16, u64, i16, 42, 42);
    test_uint_as!(to_i32, u64, i32, 42, 42);
    test_uint_as!(to_i64, u64, i64, 42, 42);
    test_uint_as!(to_i128, u64, i128, 42, 42);
    test_uint_as!(to_isize, u64, isize, 42, 42);
}

mod u128_tests {
    use super::*;
    test_uint_as!(to_f32, u128, f32, 42, 42.0);
    test_uint_as!(to_f64, u128, f64, 42, 42.0);
    test_uint_as!(to_u8, u128, u8, 42, 42);
    test_uint_as!(to_u16, u128, u16, 42, 42);
    test_uint_as!(to_u32, u128, u32, 42, 42);
    test_uint_as!(to_u64, u128, u64, 42, 42);
    test_uint_as!(to_u128, u128, u128, 42, 42);
    test_uint_as!(to_usize, u128, usize, 42, 42);
    test_uint_as!(to_i8, u128, i8, 42, 42);
    test_uint_as!(to_i16, u128, i16, 42, 42);
    test_uint_as!(to_i32, u128, i32, 42, 42);
    test_uint_as!(to_i64, u128, i64, 42, 42);
    test_uint_as!(to_i128, u128, i128, 42, 42);
    test_uint_as!(to_isize, u128, isize, 42, 42);
}

mod usize_tests {
    use super::*;
    test_uint_as!(to_f32, usize, f32, 42, 42.0);
    test_uint_as!(to_f64, usize, f64, 42, 42.0);
    test_uint_as!(to_u8, usize, u8, 42, 42);
    test_uint_as!(to_u16, usize, u16, 42, 42);
    test_uint_as!(to_u32, usize, u32, 42, 42);
    test_uint_as!(to_u64, usize, u64, 42, 42);
    test_uint_as!(to_u128, usize, u128, 42, 42);
    test_uint_as!(to_usize, usize, usize, 42, 42);
    test_uint_as!(to_i8, usize, i8, 42, 42);
    test_uint_as!(to_i16, usize, i16, 42, 42);
    test_uint_as!(to_i32, usize, i32, 42, 42);
    test_uint_as!(to_i64, usize, i64, 42, 42);
    test_uint_as!(to_i128, usize, i128, 42, 42);
    test_uint_as!(to_isize, usize, isize, 42, 42);
}
