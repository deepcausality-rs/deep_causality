/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::ToPrimitive;

macro_rules! test_to {
    ($name:ident, $method:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let expected: Option<$to> = $expected;
            let actual = v.$method();
            assert_eq!(actual, expected);
        }
    };
}

macro_rules! test_to_float {
    ($name:ident, $method:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let expected: Option<$to> = $expected;
            let actual = v.$method();

            match (actual, expected) {
                (Some(a), Some(e)) if a.is_nan() && e.is_nan() => assert!(true),
                (Some(a), Some(e)) => assert_eq!(a, e),
                (None, None) => assert!(true),
                _ => panic!(
                    "assertion `left == right` failed\n  left: {:?}\n right: {:?}",
                    actual, expected
                ),
            }
        }
    };
}

mod f32_to_tests {
    use super::*;

    // Tests for to_isize
    test_to!(to_isize_ok, to_isize, f32, isize, 42.0, Some(42));
    test_to!(to_isize_fail_overflow, to_isize, f32, isize, f32::MAX, None);
    test_to!(
        to_isize_ok_min,
        to_isize,
        f32,
        isize,
        isize::MIN as f32,
        Some(isize::MIN)
    );
    test_to!(
        to_isize_fail_underflow,
        to_isize,
        f32,
        isize,
        f32::MIN,
        None
    );
    test_to!(to_isize_nan, to_isize, f32, isize, f32::NAN, None);
    test_to!(to_isize_infinity, to_isize, f32, isize, f32::INFINITY, None);
    test_to!(
        to_isize_neg_infinity,
        to_isize,
        f32,
        isize,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_isize_zero, to_isize, f32, isize, 0.0, Some(0));
    test_to!(to_isize_neg_zero, to_isize, f32, isize, -0.0, Some(0));

    // Tests for to_i8
    test_to!(to_i8_ok, to_i8, f32, i8, 42.0, Some(42));
    test_to!(
        to_i8_fail_overflow,
        to_i8,
        f32,
        i8,
        i8::MAX as f32 + 1.0,
        None
    );
    test_to!(to_i8_ok_max, to_i8, f32, i8, i8::MAX as f32, Some(i8::MAX));
    test_to!(to_i8_ok_min, to_i8, f32, i8, i8::MIN as f32, Some(i8::MIN));
    test_to!(
        to_i8_fail_underflow,
        to_i8,
        f32,
        i8,
        i8::MIN as f32 - 1.0,
        None
    );
    test_to!(to_i8_nan, to_i8, f32, i8, f32::NAN, None);
    test_to!(to_i8_infinity, to_i8, f32, i8, f32::INFINITY, None);
    test_to!(to_i8_neg_infinity, to_i8, f32, i8, f32::NEG_INFINITY, None);
    test_to!(to_i8_zero, to_i8, f32, i8, 0.0, Some(0));
    test_to!(to_i8_neg_zero, to_i8, f32, i8, -0.0, Some(0));

    // Tests for to_i16
    test_to!(to_i16_ok, to_i16, f32, i16, 42.0, Some(42));
    test_to!(
        to_i16_fail_overflow,
        to_i16,
        f32,
        i16,
        i16::MAX as f32 + 1.0,
        None
    );
    test_to!(
        to_i16_ok_max,
        to_i16,
        f32,
        i16,
        i16::MAX as f32,
        Some(i16::MAX)
    );
    test_to!(
        to_i16_ok_min,
        to_i16,
        f32,
        i16,
        i16::MIN as f32,
        Some(i16::MIN)
    );
    test_to!(
        to_i16_fail_underflow,
        to_i16,
        f32,
        i16,
        i16::MIN as f32 - 1.0,
        None
    );
    test_to!(to_i16_nan, to_i16, f32, i16, f32::NAN, None);
    test_to!(to_i16_infinity, to_i16, f32, i16, f32::INFINITY, None);
    test_to!(
        to_i16_neg_infinity,
        to_i16,
        f32,
        i16,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_i16_zero, to_i16, f32, i16, 0.0, Some(0));
    test_to!(to_i16_neg_zero, to_i16, f32, i16, -0.0, Some(0));

    // Tests for to_i32
    test_to!(to_i32_ok, to_i32, f32, i32, 42.0, Some(42));
    test_to!(to_i32_fail_overflow, to_i32, f32, i32, f32::MAX, None);
    test_to!(
        to_i32_ok_min,
        to_i32,
        f32,
        i32,
        i32::MIN as f32,
        Some(i32::MIN)
    );
    test_to!(to_i32_fail_underflow, to_i32, f32, i32, f32::MIN, None);
    test_to!(to_i32_nan, to_i32, f32, i32, f32::NAN, None);
    test_to!(to_i32_infinity, to_i32, f32, i32, f32::INFINITY, None);
    test_to!(
        to_i32_neg_infinity,
        to_i32,
        f32,
        i32,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_i32_zero, to_i32, f32, i32, 0.0, Some(0));
    test_to!(to_i32_neg_zero, to_i32, f32, i32, -0.0, Some(0));

    // Tests for to_i64
    test_to!(to_i64_ok, to_i64, f32, i64, 42.0, Some(42));
    test_to!(to_i64_fail_overflow, to_i64, f32, i64, f32::MAX, None);
    test_to!(to_i64_ok_min, to_i64, f32, i64, f32::MIN, None);
    test_to!(to_i64_nan, to_i64, f32, i64, f32::NAN, None);
    test_to!(to_i64_infinity, to_i64, f32, i64, f32::INFINITY, None);
    test_to!(
        to_i64_neg_infinity,
        to_i64,
        f32,
        i64,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_i64_zero, to_i64, f32, i64, 0.0, Some(0));
    test_to!(to_i64_neg_zero, to_i64, f32, i64, -0.0, Some(0));

    // Tests for to_i128
    test_to!(to_i128_ok, to_i128, f32, i128, 42.0, Some(42));
    test_to!(to_i128_fail_overflow, to_i128, f32, i128, f32::MAX, None);
    test_to!(to_i128_ok_min, to_i128, f32, i128, f32::MIN, None);
    test_to!(to_i128_nan, to_i128, f32, i128, f32::NAN, None);
    test_to!(to_i128_infinity, to_i128, f32, i128, f32::INFINITY, None);
    test_to!(
        to_i128_neg_infinity,
        to_i128,
        f32,
        i128,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_i128_zero, to_i128, f32, i128, 0.0, Some(0));
    test_to!(to_i128_neg_zero, to_i128, f32, i128, -0.0, Some(0));

    // Tests for to_usize
    test_to!(to_usize_ok, to_usize, f32, usize, 42.0, Some(42));
    test_to!(to_usize_fail_negative, to_usize, f32, usize, -1.0, None);
    test_to!(to_usize_fail_overflow, to_usize, f32, usize, f32::MAX, None);
    test_to!(to_usize_nan, to_usize, f32, usize, f32::NAN, None);
    test_to!(to_usize_infinity, to_usize, f32, usize, f32::INFINITY, None);
    test_to!(
        to_usize_neg_infinity,
        to_usize,
        f32,
        usize,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_usize_zero, to_usize, f32, usize, 0.0, Some(0));
    test_to!(to_usize_neg_zero, to_usize, f32, usize, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u8
    test_to!(to_u8_ok, to_u8, f32, u8, 42.0, Some(42));
    test_to!(
        to_u8_fail_overflow,
        to_u8,
        f32,
        u8,
        u8::MAX as f32 + 1.0,
        None
    );
    test_to!(to_u8_ok_max, to_u8, f32, u8, u8::MAX as f32, Some(u8::MAX));
    test_to!(to_u8_ok_zero, to_u8, f32, u8, 0.0, Some(0));
    test_to!(to_u8_fail_negative, to_u8, f32, u8, -1.0, None);
    test_to!(to_u8_nan, to_u8, f32, u8, f32::NAN, None);
    test_to!(to_u8_infinity, to_u8, f32, u8, f32::INFINITY, None);
    test_to!(to_u8_neg_infinity, to_u8, f32, u8, f32::NEG_INFINITY, None);
    test_to!(to_u8_neg_zero, to_u8, f32, u8, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u16
    test_to!(to_u16_ok, to_u16, f32, u16, 42.0, Some(42));
    test_to!(
        to_u16_fail_overflow,
        to_u16,
        f32,
        u16,
        u16::MAX as f32 + 1.0,
        None
    );
    test_to!(
        to_u16_ok_max,
        to_u16,
        f32,
        u16,
        u16::MAX as f32,
        Some(u16::MAX)
    );
    test_to!(to_u16_ok_zero, to_u16, f32, u16, 0.0, Some(0));
    test_to!(to_u16_fail_negative, to_u16, f32, u16, -1.0, None);
    test_to!(to_u16_nan, to_u16, f32, u16, f32::NAN, None);
    test_to!(to_u16_infinity, to_u16, f32, u16, f32::INFINITY, None);
    test_to!(
        to_u16_neg_infinity,
        to_u16,
        f32,
        u16,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_u16_neg_zero, to_u16, f32, u16, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u32
    test_to!(to_u32_ok, to_u32, f32, u32, 42.0, Some(42));
    test_to!(to_u32_fail_overflow, to_u32, f32, u32, f32::MAX, None);
    test_to!(to_u32_ok_max, to_u32, f32, u32, u32::MAX as f32, None);
    test_to!(to_u32_ok_zero, to_u32, f32, u32, 0.0, Some(0));
    test_to!(to_u32_fail_negative, to_u32, f32, u32, -1.0, None);
    test_to!(to_u32_nan, to_u32, f32, u32, f32::NAN, None);
    test_to!(to_u32_infinity, to_u32, f32, u32, f32::INFINITY, None);
    test_to!(
        to_u32_neg_infinity,
        to_u32,
        f32,
        u32,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_u32_neg_zero, to_u32, f32, u32, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u64
    test_to!(to_u64_ok, to_u64, f32, u64, 42.0, Some(42));
    test_to!(to_u64_fail_overflow, to_u64, f32, u64, f32::MAX, None);
    test_to!(to_u64_ok_zero, to_u64, f32, u64, 0.0, Some(0));
    test_to!(to_u64_fail_negative, to_u64, f32, u64, -1.0, None);
    test_to!(to_u64_nan, to_u64, f32, u64, f32::NAN, None);
    test_to!(to_u64_infinity, to_u64, f32, u64, f32::INFINITY, None);
    test_to!(
        to_u64_neg_infinity,
        to_u64,
        f32,
        u64,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_u64_neg_zero, to_u64, f32, u64, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u128
    test_to!(to_u128_ok, to_u128, f32, u128, 42.0, Some(42));
    test_to!(to_u128_ok_max, to_u128, f32, u128, u128::MAX as f32, None);
    test_to!(to_u128_fail_negative, to_u128, f32, u128, -1.0, None);
    test_to!(to_u128_ok_zero, to_u128, f32, u128, 0.0, Some(0));
    test_to!(to_u128_nan, to_u128, f32, u128, f32::NAN, None);
    test_to!(to_u128_infinity, to_u128, f32, u128, f32::INFINITY, None);
    test_to!(
        to_u128_neg_infinity,
        to_u128,
        f32,
        u128,
        f32::NEG_INFINITY,
        None
    );
    test_to!(to_u128_neg_zero, to_u128, f32, u128, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_f32
    test_to_float!(to_f32_ok, to_f32, f32, f32, 42.0, Some(42.0));
    test_to_float!(to_f32_nan, to_f32, f32, f32, f32::NAN, Some(f32::NAN));
    test_to_float!(
        to_f32_infinity,
        to_f32,
        f32,
        f32,
        f32::INFINITY,
        Some(f32::INFINITY)
    );
    test_to_float!(
        to_f32_neg_infinity,
        to_f32,
        f32,
        f32,
        f32::NEG_INFINITY,
        Some(f32::NEG_INFINITY)
    );
    test_to_float!(to_f32_zero, to_f32, f32, f32, 0.0, Some(0.0));
    test_to_float!(to_f32_neg_zero, to_f32, f32, f32, -0.0, Some(-0.0));

    // Tests for to_f64
    test_to_float!(to_f64_ok, to_f64, f32, f64, 42.0, Some(42.0));
    test_to_float!(to_f64_nan, to_f64, f32, f64, f32::NAN, Some(f64::NAN));
    test_to_float!(
        to_f64_infinity,
        to_f64,
        f32,
        f64,
        f32::INFINITY,
        Some(f64::INFINITY)
    );
    test_to_float!(
        to_f64_neg_infinity,
        to_f64,
        f32,
        f64,
        f32::NEG_INFINITY,
        Some(f64::NEG_INFINITY)
    );
    test_to_float!(to_f64_zero, to_f64, f32, f64, 0.0, Some(0.0));
    test_to_float!(to_f64_neg_zero, to_f64, f32, f64, -0.0, Some(-0.0));
}

mod f64_to_tests {
    use super::*;

    // Tests for to_isize
    test_to!(to_isize_ok, to_isize, f64, isize, 42.0, Some(42));
    test_to!(to_isize_fail_overflow, to_isize, f64, isize, f64::MAX, None);
    test_to!(
        to_isize_ok_max,
        to_isize,
        f64,
        isize,
        isize::MAX as f64,
        Some(isize::MAX)
    );
    test_to!(
        to_isize_ok_min,
        to_isize,
        f64,
        isize,
        isize::MIN as f64,
        Some(isize::MIN)
    );
    test_to!(
        to_isize_fail_underflow,
        to_isize,
        f64,
        isize,
        f64::MIN,
        None
    );
    test_to!(to_isize_nan, to_isize, f64, isize, f64::NAN, None);
    test_to!(to_isize_infinity, to_isize, f64, isize, f64::INFINITY, None);
    test_to!(
        to_isize_neg_infinity,
        to_isize,
        f64,
        isize,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_isize_zero, to_isize, f64, isize, 0.0, Some(0));
    test_to!(to_isize_neg_zero, to_isize, f64, isize, -0.0, Some(0));

    // Tests for to_i8
    test_to!(to_i8_ok, to_i8, f64, i8, 42.0, Some(42));
    test_to!(
        to_i8_fail_overflow,
        to_i8,
        f64,
        i8,
        i8::MAX as f64 + 1.0,
        None
    );
    test_to!(to_i8_ok_max, to_i8, f64, i8, i8::MAX as f64, Some(i8::MAX));
    test_to!(to_i8_ok_min, to_i8, f64, i8, i8::MIN as f64, Some(i8::MIN));
    test_to!(
        to_i8_fail_underflow,
        to_i8,
        f64,
        i8,
        i8::MIN as f64 - 1.0,
        None
    );
    test_to!(to_i8_nan, to_i8, f64, i8, f64::NAN, None);
    test_to!(to_i8_infinity, to_i8, f64, i8, f64::INFINITY, None);
    test_to!(to_i8_neg_infinity, to_i8, f64, i8, f64::NEG_INFINITY, None);
    test_to!(to_i8_zero, to_i8, f64, i8, 0.0, Some(0));
    test_to!(to_i8_neg_zero, to_i8, f64, i8, -0.0, Some(0));

    // Tests for to_i16
    test_to!(to_i16_ok, to_i16, f64, i16, 42.0, Some(42));
    test_to!(
        to_i16_fail_overflow,
        to_i16,
        f64,
        i16,
        i16::MAX as f64 + 1.0,
        None
    );
    test_to!(
        to_i16_ok_max,
        to_i16,
        f64,
        i16,
        i16::MAX as f64,
        Some(i16::MAX)
    );
    test_to!(
        to_i16_ok_min,
        to_i16,
        f64,
        i16,
        i16::MIN as f64,
        Some(i16::MIN)
    );
    test_to!(
        to_i16_fail_underflow,
        to_i16,
        f64,
        i16,
        i16::MIN as f64 - 1.0,
        None
    );
    test_to!(to_i16_nan, to_i16, f64, i16, f64::NAN, None);
    test_to!(to_i16_infinity, to_i16, f64, i16, f64::INFINITY, None);
    test_to!(
        to_i16_neg_infinity,
        to_i16,
        f64,
        i16,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_i16_zero, to_i16, f64, i16, 0.0, Some(0));
    test_to!(to_i16_neg_zero, to_i16, f64, i16, -0.0, Some(0));

    // Tests for to_i32
    test_to!(to_i32_ok, to_i32, f64, i32, 42.0, Some(42));
    test_to!(
        to_i32_fail_overflow,
        to_i32,
        f64,
        i32,
        i32::MAX as f64 + 1.0,
        None
    );
    test_to!(
        to_i32_ok_max,
        to_i32,
        f64,
        i32,
        i32::MAX as f64,
        Some(i32::MAX)
    );
    test_to!(
        to_i32_ok_min,
        to_i32,
        f64,
        i32,
        i32::MIN as f64,
        Some(i32::MIN)
    );
    test_to!(
        to_i32_fail_underflow,
        to_i32,
        f64,
        i32,
        i32::MIN as f64 - 1.0,
        None
    );
    test_to!(to_i32_nan, to_i32, f64, i32, f64::NAN, None);
    test_to!(to_i32_infinity, to_i32, f64, i32, f64::INFINITY, None);
    test_to!(
        to_i32_neg_infinity,
        to_i32,
        f64,
        i32,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_i32_zero, to_i32, f64, i32, 0.0, Some(0));
    test_to!(to_i32_neg_zero, to_i32, f64, i32, -0.0, Some(0));

    // Tests for to_i64
    test_to!(to_i64_ok, to_i64, f64, i64, 42.0, Some(42));
    test_to!(to_i64_fail_overflow, to_i64, f64, i64, f64::MAX, None);
    test_to!(
        to_i64_ok_max,
        to_i64,
        f64,
        i64,
        i64::MAX as f64,
        Some(i64::MAX)
    );
    test_to!(
        to_i64_ok_min,
        to_i64,
        f64,
        i64,
        i64::MIN as f64,
        Some(i64::MIN)
    );
    test_to!(to_i64_fail_underflow, to_i64, f64, i64, f64::MIN, None);
    test_to!(to_i64_nan, to_i64, f64, i64, f64::NAN, None);
    test_to!(to_i64_infinity, to_i64, f64, i64, f64::INFINITY, None);
    test_to!(
        to_i64_neg_infinity,
        to_i64,
        f64,
        i64,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_i64_zero, to_i64, f64, i64, 0.0, Some(0));
    test_to!(to_i64_neg_zero, to_i64, f64, i64, -0.0, Some(0));

    // Tests for to_i128
    test_to!(to_i128_ok, to_i128, f64, i128, 42.0, Some(42));
    test_to!(to_i128_fail_overflow, to_i128, f64, i128, f64::MAX, None);
    test_to!(to_i128_ok_min, to_i128, f64, i128, f64::MIN, None);
    test_to!(to_i128_nan, to_i128, f64, i128, f64::NAN, None);
    test_to!(to_i128_infinity, to_i128, f64, i128, f64::INFINITY, None);
    test_to!(
        to_i128_neg_infinity,
        to_i128,
        f64,
        i128,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_i128_zero, to_i128, f64, i128, 0.0, Some(0));
    test_to!(to_i128_neg_zero, to_i128, f64, i128, -0.0, Some(0));

    // Tests for to_usize
    test_to!(to_usize_ok, to_usize, f64, usize, 42.0, Some(42));
    test_to!(to_usize_fail_negative, to_usize, f64, usize, -1.0, None);
    test_to!(to_usize_fail_overflow, to_usize, f64, usize, f64::MAX, None);
    test_to!(to_usize_nan, to_usize, f64, usize, f64::NAN, None);
    test_to!(to_usize_infinity, to_usize, f64, usize, f64::INFINITY, None);
    test_to!(
        to_usize_neg_infinity,
        to_usize,
        f64,
        usize,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_usize_zero, to_usize, f64, usize, 0.0, Some(0));
    test_to!(to_usize_neg_zero, to_usize, f64, usize, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u8
    test_to!(to_u8_ok, to_u8, f64, u8, 42.0, Some(42));
    test_to!(
        to_u8_fail_overflow,
        to_u8,
        f64,
        u8,
        u8::MAX as f64 + 1.0,
        None
    );
    test_to!(to_u8_ok_max, to_u8, f64, u8, u8::MAX as f64, Some(u8::MAX));
    test_to!(to_u8_ok_zero, to_u8, f64, u8, 0.0, Some(0));
    test_to!(to_u8_fail_negative, to_u8, f64, u8, -1.0, None);
    test_to!(to_u8_nan, to_u8, f64, u8, f64::NAN, None);
    test_to!(to_u8_infinity, to_u8, f64, u8, f64::INFINITY, None);
    test_to!(to_u8_neg_infinity, to_u8, f64, u8, f64::NEG_INFINITY, None);
    test_to!(to_u8_neg_zero, to_u8, f64, u8, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u16
    test_to!(to_u16_ok, to_u16, f64, u16, 42.0, Some(42));
    test_to!(
        to_u16_fail_overflow,
        to_u16,
        f64,
        u16,
        u16::MAX as f64 + 1.0,
        None
    );
    test_to!(
        to_u16_ok_max,
        to_u16,
        f64,
        u16,
        u16::MAX as f64,
        Some(u16::MAX)
    );
    test_to!(to_u16_ok_zero, to_u16, f64, u16, 0.0, Some(0));
    test_to!(to_u16_fail_negative, to_u16, f64, u16, -1.0, None);
    test_to!(to_u16_nan, to_u16, f64, u16, f64::NAN, None);
    test_to!(to_u16_infinity, to_u16, f64, u16, f64::INFINITY, None);
    test_to!(
        to_u16_neg_infinity,
        to_u16,
        f64,
        u16,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_u16_neg_zero, to_u16, f64, u16, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u32
    test_to!(to_u32_ok, to_u32, f64, u32, 42.0, Some(42));
    test_to!(
        to_u32_fail_overflow,
        to_u32,
        f64,
        u32,
        u32::MAX as f64 + 1.0,
        None
    );
    test_to!(
        to_u32_ok_max,
        to_u32,
        f64,
        u32,
        u32::MAX as f64,
        Some(u32::MAX)
    );
    test_to!(to_u32_ok_zero, to_u32, f64, u32, 0.0, Some(0));
    test_to!(to_u32_fail_negative, to_u32, f64, u32, -1.0, None);
    test_to!(to_u32_nan, to_u32, f64, u32, f64::NAN, None);
    test_to!(to_u32_infinity, to_u32, f64, u32, f64::INFINITY, None);
    test_to!(
        to_u32_neg_infinity,
        to_u32,
        f64,
        u32,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_u32_neg_zero, to_u32, f64, u32, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u64
    test_to!(to_u64_ok, to_u64, f64, u64, 42.0, Some(42));
    test_to!(to_u64_fail_overflow, to_u64, f64, u64, f64::MAX, None);
    test_to!(
        to_u64_ok_max,
        to_u64,
        f64,
        u64,
        u64::MAX as f64,
        Some(u64::MAX)
    );
    test_to!(to_u64_ok_zero, to_u64, f64, u64, 0.0, Some(0));
    test_to!(to_u64_fail_negative, to_u64, f64, u64, -1.0, None);
    test_to!(to_u64_nan, to_u64, f64, u64, f64::NAN, None);
    test_to!(to_u64_infinity, to_u64, f64, u64, f64::INFINITY, None);
    test_to!(
        to_u64_neg_infinity,
        to_u64,
        f64,
        u64,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_u64_neg_zero, to_u64, f64, u64, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_u128
    test_to!(to_u128_ok, to_u128, f64, u128, 42.0, Some(42));
    test_to!(
        to_u128_ok_max,
        to_u128,
        f64,
        u128,
        u128::MAX as f64,
        Some(u128::MAX)
    );
    test_to!(to_u128_ok_zero, to_u128, f64, u128, 0.0, Some(0));
    test_to!(to_u128_fail_negative, to_u128, f64, u128, -1.0, None);
    test_to!(to_u128_nan, to_u128, f64, u128, f64::NAN, None);
    test_to!(to_u128_infinity, to_u128, f64, u128, f64::INFINITY, None);
    test_to!(
        to_u128_neg_infinity,
        to_u128,
        f64,
        u128,
        f64::NEG_INFINITY,
        None
    );
    test_to!(to_u128_neg_zero, to_u128, f64, u128, -0.0, None); // Unsigned types cannot represent -0.0

    // Tests for to_f32
    test_to_float!(to_f32_ok, to_f32, f64, f32, 42.0, Some(42.0));
    test_to_float!(to_f32_nan, to_f32, f64, f32, f64::NAN, Some(f32::NAN));
    test_to_float!(
        to_f32_infinity,
        to_f32,
        f64,
        f32,
        f64::INFINITY,
        Some(f32::INFINITY)
    );
    test_to_float!(
        to_f32_neg_infinity,
        to_f32,
        f64,
        f32,
        f64::NEG_INFINITY,
        Some(f32::NEG_INFINITY)
    );
    test_to_float!(to_f32_zero, to_f32, f64, f32, 0.0, Some(0.0));
    test_to_float!(to_f32_neg_zero, to_f32, f64, f32, -0.0, Some(-0.0));

    // Tests for to_f64
    test_to_float!(to_f64_ok, to_f64, f64, f64, 42.0, Some(42.0));
    test_to_float!(to_f64_nan, to_f64, f64, f64, f64::NAN, Some(f64::NAN));
    test_to_float!(
        to_f64_infinity,
        to_f64,
        f64,
        f64,
        f64::INFINITY,
        Some(f64::INFINITY)
    );
    test_to_float!(
        to_f64_neg_infinity,
        to_f64,
        f64,
        f64,
        f64::NEG_INFINITY,
        Some(f64::NEG_INFINITY)
    );
    test_to_float!(to_f64_zero, to_f64, f64, f64, 0.0, Some(0.0));
    test_to_float!(to_f64_neg_zero, to_f64, f64, f64, -0.0, Some(-0.0));
}
