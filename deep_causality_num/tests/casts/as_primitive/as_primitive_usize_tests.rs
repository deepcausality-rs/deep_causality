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

mod u16_tests {
    use super::*;
    test_as!(to_u8_truncation, u16, u8, 300, 44);
    test_as!(to_u16, u16, u16, 300, 300);
}

mod u32_tests {
    use super::*;
    test_as!(to_u16_truncation, u32, u16, 70000, 4464);
    test_as!(to_u32, u32, u32, 70000, 70000);
}

mod u64_tests {
    use super::*;
    test_as!(to_u32_truncation, u64, u32, u32::MAX as u64 + 1, 0);
    test_as!(to_u64, u64, u64, u32::MAX as u64 + 1, u32::MAX as u64 + 1);
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
