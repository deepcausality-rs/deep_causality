/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::AsPrimitive;

macro_rules! test_as {
    ($name:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let a: $to = v.as_();
            assert_eq!(a, $expected);
        }
    };
}

mod u8_tests {
    use super::*;
    test_as!(to_u8, u8, u8, 42, 42);
    test_as!(to_u16, u8, u16, 42, 42);
    test_as!(to_u32, u8, u32, 42, 42);
    test_as!(to_u64, u8, u64, 42, 42);
    test_as!(to_u128, u8, u128, 42, 42);
    test_as!(to_usize, u8, usize, 42, 42);
    test_as!(to_i8, u8, i8, 42, 42);
    test_as!(to_i8_overflow, u8, i8, 128, -128);
    test_as!(to_i16, u8, i16, 42, 42);
    test_as!(to_i32, u8, i32, 42, 42);
    test_as!(to_i64, u8, i64, 42, 42);
    test_as!(to_i128, u8, i128, 42, 42);
    test_as!(to_isize, u8, isize, 42, 42);
    test_as!(to_f32, u8, f32, 42, 42.0f32);
    test_as!(to_f64, u8, f64, 42, 42.0f64);
    test_as!(to_char, u8, char, 65, 'A');
}

mod i8_tests {
    use super::*;
    test_as!(to_u8, i8, u8, 42, 42);
    test_as!(to_u8_neg, i8, u8, -1, 255);
    test_as!(to_i8, i8, i8, 42, 42);
    test_as!(to_i16, i8, i16, 42, 42);
    test_as!(to_f32, i8, f32, 42, 42.0f32);
    test_as!(to_f32_neg, i8, f32, -42, -42.0f32);
    test_as!(to_f64, i8, f64, 42, 42.0f64);
    test_as!(to_f64_neg, i8, f64, -42, -42.0f64);
}

mod u16_tests {
    use super::*;
    test_as!(to_u8_truncation, u16, u8, 300, 44);
    test_as!(to_u16, u16, u16, 300, 300);
}

mod i16_tests {
    use super::*;
    test_as!(to_i8_truncation, i16, i8, 300, 44);
    test_as!(to_u16_negative, i16, u16, -1, 65535);
    test_as!(to_i16, i16, i16, 300, 300);
}

mod u32_tests {
    use super::*;
    test_as!(to_u16_truncation, u32, u16, 70000, 4464);
    test_as!(to_u32, u32, u32, 70000, 70000);
}

mod i32_tests {
    use super::*;
    test_as!(to_i16_truncation, i32, i16, 70000, 4464);
    test_as!(to_i32, i32, i32, 70000, 70000);
}

mod u64_tests {
    use super::*;
    test_as!(to_u32_truncation, u64, u32, u32::MAX as u64 + 1, 0);
    test_as!(to_u64, u64, u64, u32::MAX as u64 + 1, u32::MAX as u64 + 1);
}

mod i64_tests {
    use super::*;
    test_as!(to_i32_truncation, i64, i32, i32::MAX as i64 + 1, i32::MIN);
    test_as!(to_i64, i64, i64, i32::MAX as i64 + 1, i32::MAX as i64 + 1);
}

mod u128_tests {
    use super::*;
    test_as!(to_u64_truncation, u128, u64, u64::MAX as u128 + 1, 0);
    test_as!(
        to_u128,
        u128,
        u128,
        u64::MAX as u128 + 1,
        u64::MAX as u128 + 1
    );
}

mod i128_tests {
    use super::*;
    test_as!(to_i64_truncation, i128, i64, i64::MAX as i128 + 1, i64::MIN);
    test_as!(
        to_i128,
        i128,
        i128,
        i64::MAX as i128 + 1,
        i64::MAX as i128 + 1
    );
}

mod f32_tests {
    use super::*;
    test_as!(to_i32_truncate, f32, i32, 3.0, 3);
    test_as!(to_i32_truncate_neg, f32, i32, -3.0, -3);
    test_as!(to_u32_truncate, f32, u32, 3.0, 3);
    test_as!(to_f64, f32, f64, 3.0, 3.0f32 as f64);
}

mod f64_tests {
    use super::*;
    test_as!(to_i64_truncate, f64, i64, 3.0, 3);
    test_as!(to_i64_truncate_neg, f64, i64, -3.0, -3);
    test_as!(to_u64_truncate, f64, u64, 3.0, 3);
    test_as!(to_f32, f64, f32, 3.0, 3.0f64 as f32);
}

mod char_tests {
    use super::*;
    test_as!(to_u8, char, u8, 'A', 65);
    test_as!(to_i8, char, i8, 'A', 65);
    test_as!(to_u8_overflow, char, u8, '\u{FF}', 255);
    test_as!(to_i8_overflow, char, i8, '\u{FF}', -1);
}

mod bool_tests {
    use super::*;
    test_as!(to_u8_true, bool, u8, true, 1);
    test_as!(to_u8_false, bool, u8, false, 0);
    test_as!(to_i8_true, bool, i8, true, 1);
    test_as!(to_i8_false, bool, i8, false, 0);
}
