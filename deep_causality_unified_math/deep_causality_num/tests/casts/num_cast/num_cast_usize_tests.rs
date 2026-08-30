/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::NumCast;

macro_rules! test_cast {
    ($name:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let expected: Option<$to> = $expected;
            let actual: Option<$to> = NumCast::from(v);
            assert_eq!(actual, expected);
        }
    };
}

mod to_usize_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_usize_success, u8, usize, 42, Some(42));
    test_cast!(from_u16_to_usize_success, u16, usize, 42, Some(42));
    test_cast!(from_u32_to_usize_success, u32, usize, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_u32_to_usize_max_64bit,
        u32,
        usize,
        u32::MAX,
        Some(u32::MAX as usize)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_u32_to_usize_max_32bit,
        u32,
        usize,
        u32::MAX,
        Some(u32::MAX as usize)
    );
    test_cast!(from_u64_to_usize_success, u64, usize, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_u64_to_usize_max_64bit,
        u64,
        usize,
        u64::MAX,
        Some(u64::MAX as usize)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_u64_to_usize_fail_overflow_32bit,
        u64,
        usize,
        u64::MAX,
        None
    );
    test_cast!(from_u128_to_usize_success, u128, usize, 42, Some(42));
    test_cast!(
        from_u128_to_usize_fail_overflow,
        u128,
        usize,
        u128::MAX,
        None
    );
    test_cast!(from_usize_identity, usize, usize, 42, Some(42));
    test_cast!(from_usize_max, usize, usize, usize::MAX, Some(usize::MAX));

    // From signed integers
    test_cast!(from_i8_to_usize_success, i8, usize, 42, Some(42));
    test_cast!(from_i8_to_usize_fail_negative, i8, usize, -42, None);
    test_cast!(from_i16_to_usize_success, i16, usize, 42, Some(42));
    test_cast!(from_i16_to_usize_fail_negative, i16, usize, -42, None);
    test_cast!(from_i32_to_usize_success, i32, usize, 42, Some(42));
    test_cast!(from_i32_to_usize_fail_negative, i32, usize, -42, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_i32_to_usize_max_64bit,
        i32,
        usize,
        i32::MAX,
        Some(i32::MAX as usize)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_i32_to_usize_max_32bit,
        i32,
        usize,
        i32::MAX,
        Some(i32::MAX as usize)
    );
    test_cast!(from_i64_to_usize_success, i64, usize, 42, Some(42));
    test_cast!(from_i64_to_usize_fail_negative, i64, usize, -42, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_i64_to_usize_max_64bit,
        i64,
        usize,
        i64::MAX,
        Some(i64::MAX as usize)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_i64_to_usize_fail_overflow_32bit,
        i64,
        usize,
        i64::MAX,
        None
    );
    test_cast!(from_i128_to_usize_success, i128, usize, 42, Some(42));
    test_cast!(from_i128_to_usize_fail_negative, i128, usize, -42, None);
    test_cast!(
        from_i128_to_usize_fail_overflow,
        i128,
        usize,
        i128::MAX,
        None
    );
    test_cast!(from_isize_to_usize_success, isize, usize, 42, Some(42));
    test_cast!(from_isize_to_usize_fail_negative, isize, usize, -42, None);
    test_cast!(
        from_isize_max,
        isize,
        usize,
        isize::MAX,
        Some(isize::MAX as usize)
    );
    test_cast!(from_isize_min, isize, usize, isize::MIN, None);

    // From floats
    test_cast!(from_f32_to_usize_success, f32, usize, 42.0, Some(42));
    test_cast!(from_f32_to_usize_fail_negative, f32, usize, -42.0, None);
    test_cast!(from_f32_to_usize_fail_overflow, f32, usize, f32::MAX, None);
    test_cast!(from_f32_to_usize_fail_nan, f32, usize, f32::NAN, None);
    test_cast!(
        from_f32_to_usize_fail_infinity,
        f32,
        usize,
        f32::INFINITY,
        None
    );
    test_cast!(
        from_f32_to_usize_fail_neg_infinity,
        f32,
        usize,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_usize_success, f64, usize, 42.0, Some(42));
    test_cast!(from_f64_to_usize_fail_negative, f64, usize, -42.0, None);
    test_cast!(from_f64_to_usize_fail_overflow, f64, usize, f64::MAX, None);
    test_cast!(from_f64_to_usize_fail_nan, f64, usize, f64::NAN, None);
    test_cast!(
        from_f64_to_usize_fail_infinity,
        f64,
        usize,
        f64::INFINITY,
        None
    );
    test_cast!(
        from_f64_to_usize_fail_neg_infinity,
        f64,
        usize,
        f64::NEG_INFINITY,
        None
    );
}

mod to_u8_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_identity, u8, u8, 42, Some(42));
    test_cast!(from_u8_max, u8, u8, u8::MAX, Some(u8::MAX));
    test_cast!(from_u16_success, u16, u8, 42, Some(42));
    test_cast!(from_u16_fail_overflow, u16, u8, 300, None);
    test_cast!(from_u32_success, u32, u8, 42, Some(42));
    test_cast!(from_u32_fail_overflow, u32, u8, 300, None);
    test_cast!(from_u64_success, u64, u8, 42, Some(42));
    test_cast!(from_u64_fail_overflow, u64, u8, 300, None);
    test_cast!(from_u128_success, u128, u8, 42, Some(42));
    test_cast!(from_u128_fail_overflow, u128, u8, 300, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_success_64bit, usize, u8, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_fail_overflow_64bit, usize, u8, 300, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_success_32bit, usize, u8, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_fail_overflow_32bit, usize, u8, 300, None);

    // From signed integers
    test_cast!(from_i8_success, i8, u8, 42, Some(42));
    test_cast!(from_i8_fail_negative, i8, u8, -42, None);
    test_cast!(from_i8_max, i8, u8, i8::MAX, Some(i8::MAX as u8));
    test_cast!(from_i8_min, i8, u8, i8::MIN, None);
    test_cast!(from_i16_success, i16, u8, 42, Some(42));
    test_cast!(from_i16_fail_negative, i16, u8, -42, None);
    test_cast!(from_i16_fail_overflow, i16, u8, 300, None);
    test_cast!(from_i32_success, i32, u8, 42, Some(42));
    test_cast!(from_i32_fail_negative, i32, u8, -42, None);
    test_cast!(from_i32_fail_overflow, i32, u8, 300, None);
    test_cast!(from_i64_success, i64, u8, 42, Some(42));
    test_cast!(from_i64_fail_negative, i64, u8, -42, None);
    test_cast!(from_i64_fail_overflow, i64, u8, 300, None);
    test_cast!(from_i128_success, i128, u8, 42, Some(42));
    test_cast!(from_i128_fail_negative, i128, u8, -42, None);
    test_cast!(from_i128_fail_overflow, i128, u8, 300, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_success_64bit, isize, u8, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_fail_negative_64bit, isize, u8, -42, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_fail_overflow_64bit, isize, u8, 300, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_success_32bit, isize, u8, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_fail_negative_32bit, isize, u8, -42, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_fail_overflow_32bit, isize, u8, 300, None);

    // From floats
    test_cast!(from_f32_success, f32, u8, 42.0, Some(42));
    test_cast!(from_f32_fail_negative, f32, u8, -42.0, None);
    test_cast!(from_f32_fail_overflow, f32, u8, 300.0, None);
    test_cast!(from_f32_fail_nan, f32, u8, f32::NAN, None);
    test_cast!(from_f32_fail_infinity, f32, u8, f32::INFINITY, None);
    test_cast!(from_f32_fail_neg_infinity, f32, u8, f32::NEG_INFINITY, None);
    test_cast!(from_f64_success, f64, u8, 42.0, Some(42));
    test_cast!(from_f64_fail_negative, f64, u8, -42.0, None);
    test_cast!(from_f64_fail_overflow, f64, u8, 300.0, None);
    test_cast!(from_f64_fail_nan, f64, u8, f64::NAN, None);
    test_cast!(from_f64_fail_infinity, f64, u8, f64::INFINITY, None);
    test_cast!(from_f64_fail_neg_infinity, f64, u8, f64::NEG_INFINITY, None);
}

mod to_u16_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_u16_success, u8, u16, 42, Some(42));
    test_cast!(from_u16_identity, u16, u16, 42, Some(42));
    test_cast!(from_u16_max, u16, u16, u16::MAX, Some(u16::MAX));
    test_cast!(from_u32_to_u16_success, u32, u16, 42, Some(42));
    test_cast!(from_u32_to_u16_fail_overflow, u32, u16, 70000, None);
    test_cast!(from_u64_to_u16_success, u64, u16, 42, Some(42));
    test_cast!(from_u64_to_u16_fail_overflow, u64, u16, 70000, None);
    test_cast!(from_u128_to_u16_success, u128, u16, 42, Some(42));
    test_cast!(from_u128_to_u16_fail_overflow, u128, u16, 70000, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_u16_success_64bit, usize, u16, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_u16_fail_overflow_64bit,
        usize,
        u16,
        70000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_u16_success_32bit, usize, u16, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_u16_fail_overflow_32bit,
        usize,
        u16,
        70000,
        None
    );

    // From signed integers
    test_cast!(from_i8_to_u16_success, i8, u16, 42, Some(42));
    test_cast!(from_i8_to_u16_fail_negative, i8, u16, -42, None);
    test_cast!(from_i16_to_u16_success, i16, u16, 42, Some(42));
    test_cast!(from_i16_to_u16_fail_negative, i16, u16, -42, None);
    test_cast!(from_i16_max, i16, u16, i16::MAX, Some(i16::MAX as u16));
    test_cast!(from_i16_min, i16, u16, i16::MIN, None);
    test_cast!(from_i32_to_u16_success, i32, u16, 42, Some(42));
    test_cast!(from_i32_to_u16_fail_negative, i32, u16, -42, None);
    test_cast!(from_i32_to_u16_fail_overflow, i32, u16, 70000, None);
    test_cast!(from_i64_to_u16_success, i64, u16, 42, Some(42));
    test_cast!(from_i64_to_u16_fail_negative, i64, u16, -42, None);
    test_cast!(from_i64_to_u16_fail_overflow, i64, u16, 70000, None);
    test_cast!(from_i128_to_u16_success, i128, u16, 42, Some(42));
    test_cast!(from_i128_to_u16_fail_negative, i128, u16, -42, None);
    test_cast!(from_i128_to_u16_fail_overflow, i128, u16, 70000, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_u16_success_64bit, isize, u16, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_u16_fail_negative_64bit, isize, u16, -42, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_u16_fail_overflow_64bit,
        isize,
        u16,
        70000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_u16_success_32bit, isize, u16, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_u16_fail_negative_32bit, isize, u16, -42, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_u16_fail_overflow_32bit,
        isize,
        u16,
        70000,
        None
    );

    // From floats
    test_cast!(from_f32_to_u16_success, f32, u16, 42.0, Some(42));
    test_cast!(from_f32_to_u16_fail_negative, f32, u16, -42.0, None);
    test_cast!(from_f32_to_u16_fail_overflow, f32, u16, 70000.0, None);
    test_cast!(from_f32_to_u16_fail_nan, f32, u16, f32::NAN, None);
    test_cast!(from_f32_to_u16_fail_infinity, f32, u16, f32::INFINITY, None);
    test_cast!(
        from_f32_to_u16_fail_neg_infinity,
        f32,
        u16,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_u16_success, f64, u16, 42.0, Some(42));
    test_cast!(from_f64_to_u16_fail_negative, f64, u16, -42.0, None);
    test_cast!(from_f64_to_u16_fail_overflow, f64, u16, 70000.0, None);
    test_cast!(from_f64_to_u16_fail_nan, f64, u16, f64::NAN, None);
    test_cast!(from_f64_to_u16_fail_infinity, f64, u16, f64::INFINITY, None);
    test_cast!(
        from_f64_to_u16_fail_neg_infinity,
        f64,
        u16,
        f64::NEG_INFINITY,
        None
    );
}

mod to_u32_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_u32_success, u8, u32, 42, Some(42));
    test_cast!(from_u16_to_u32_success, u16, u32, 42, Some(42));
    test_cast!(from_u32_identity, u32, u32, 42, Some(42));
    test_cast!(from_u32_max, u32, u32, u32::MAX, Some(u32::MAX));
    test_cast!(from_u64_to_u32_success, u64, u32, 42, Some(42));
    test_cast!(from_u64_to_u32_fail_overflow, u64, u32, 5_000_000_000, None);
    test_cast!(from_u128_to_u32_success, u128, u32, 42, Some(42));
    test_cast!(
        from_u128_to_u32_fail_overflow,
        u128,
        u32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_u32_success_64bit, usize, u32, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_u32_fail_overflow_64bit,
        usize,
        u32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_u32_success_32bit, usize, u32, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_u32_fail_overflow_32bit,
        usize,
        u32,
        5_000_000_000,
        None
    );

    // From signed integers
    test_cast!(from_i8_to_u32_success, i8, u32, 42, Some(42));
    test_cast!(from_i8_to_u32_fail_negative, i8, u32, -42, None);
    test_cast!(from_i16_to_u32_success, i16, u32, 42, Some(42));
    test_cast!(from_i16_to_u32_fail_negative, i16, u32, -42, None);
    test_cast!(from_i32_to_u32_success, i32, u32, 42, Some(42));
    test_cast!(from_i32_to_u32_fail_negative, i32, u32, -42, None);
    test_cast!(from_i32_max, i32, u32, i32::MAX, Some(i32::MAX as u32));
    test_cast!(from_i32_min, i32, u32, i32::MIN, None);
    test_cast!(from_i64_to_u32_success, i64, u32, 42, Some(42));
    test_cast!(from_i64_to_u32_fail_negative, i64, u32, -42, None);
    test_cast!(from_i64_to_u32_fail_overflow, i64, u32, 5_000_000_000, None);
    test_cast!(from_i128_to_u32_success, i128, u32, 42, Some(42));
    test_cast!(from_i128_to_u32_fail_negative, i128, u32, -42, None);
    test_cast!(
        from_i128_to_u32_fail_overflow,
        i128,
        u32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_u32_success_64bit, isize, u32, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_u32_fail_negative_64bit, isize, u32, -42, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_u32_fail_overflow_64bit,
        isize,
        u32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_u32_success_32bit, isize, u32, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_u32_fail_negative_32bit, isize, u32, -42, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_u32_fail_overflow_32bit,
        isize,
        u32,
        5_000_000_000,
        None
    );

    // From floats
    test_cast!(from_f32_to_u32_success, f32, u32, 42.0, Some(42));
    test_cast!(from_f32_to_u32_fail_negative, f32, u32, -42.0, None);
    test_cast!(from_f32_to_u32_fail_overflow, f32, u32, f32::MAX, None);
    test_cast!(from_f32_to_u32_fail_nan, f32, u32, f32::NAN, None);
    test_cast!(from_f32_to_u32_fail_infinity, f32, u32, f32::INFINITY, None);
    test_cast!(
        from_f32_to_u32_fail_neg_infinity,
        f32,
        u32,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_u32_success, f64, u32, 42.0, Some(42));
    test_cast!(from_f64_to_u32_fail_negative, f64, u32, -42.0, None);
    test_cast!(from_f64_to_u32_fail_overflow, f64, u32, f64::MAX, None);
    test_cast!(from_f64_to_u32_fail_nan, f64, u32, f64::NAN, None);
    test_cast!(from_f64_to_u32_fail_infinity, f64, u32, f64::INFINITY, None);
    test_cast!(
        from_f64_to_u32_fail_neg_infinity,
        f64,
        u32,
        f64::NEG_INFINITY,
        None
    );
}

mod to_u64_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_u64_success, u8, u64, 42, Some(42));
    test_cast!(from_u16_to_u64_success, u16, u64, 42, Some(42));
    test_cast!(from_u32_to_u64_success, u32, u64, 42, Some(42));
    test_cast!(from_u64_identity, u64, u64, 42, Some(42));
    test_cast!(from_u64_max, u64, u64, u64::MAX, Some(u64::MAX));
    test_cast!(from_u128_to_u64_success, u128, u64, 42, Some(42));
    test_cast!(from_u128_to_u64_fail_overflow, u128, u64, u128::MAX, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_u64_success_64bit, usize, u64, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_u64_max_64bit,
        usize,
        u64,
        usize::MAX,
        Some(usize::MAX as u64)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_u64_success_32bit, usize, u64, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_u64_fail_overflow_32bit,
        usize,
        u64,
        usize::MAX,
        None
    );

    // From signed integers
    test_cast!(from_i8_to_u64_success, i8, u64, 42, Some(42));
    test_cast!(from_i8_to_u64_fail_negative, i8, u64, -42, None);
    test_cast!(from_i16_to_u64_success, i16, u64, 42, Some(42));
    test_cast!(from_i16_to_u64_fail_negative, i16, u64, -42, None);
    test_cast!(from_i32_to_u64_success, i32, u64, 42, Some(42));
    test_cast!(from_i32_to_u64_fail_negative, i32, u64, -42, None);
    test_cast!(from_i64_to_u64_success, i64, u64, 42, Some(42));
    test_cast!(from_i64_to_u64_fail_negative, i64, u64, -42, None);
    test_cast!(from_i64_max, i64, u64, i64::MAX, Some(i64::MAX as u64));
    test_cast!(from_i64_min, i64, u64, i64::MIN, None);
    test_cast!(from_i128_to_u64_success, i128, u64, 42, Some(42));
    test_cast!(from_i128_to_u64_fail_negative, i128, u64, -42, None);
    test_cast!(from_i128_to_u64_fail_overflow, i128, u64, i128::MAX, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_u64_success_64bit, isize, u64, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_u64_fail_negative_64bit, isize, u64, -42, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_u64_success_32bit, isize, u64, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_u64_fail_negative_32bit, isize, u64, -42, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_u64_fail_overflow_32bit,
        isize,
        u64,
        isize::MAX as u64 + 1,
        None
    );

    // From floats
    test_cast!(from_f32_to_u64_success, f32, u64, 42.0, Some(42));
    test_cast!(from_f32_to_u64_fail_negative, f32, u64, -42.0, None);
    test_cast!(from_f32_to_u64_fail_overflow, f32, u64, f32::MAX, None);
    test_cast!(from_f32_to_u64_fail_nan, f32, u64, f32::NAN, None);
    test_cast!(from_f32_to_u64_fail_infinity, f32, u64, f32::INFINITY, None);
    test_cast!(
        from_f32_to_u64_fail_neg_infinity,
        f32,
        u64,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_u64_success, f64, u64, 42.0, Some(42));
    test_cast!(from_f64_to_u64_fail_negative, f64, u64, -42.0, None);
    test_cast!(from_f64_to_u64_fail_overflow, f64, u64, f64::MAX, None);
    test_cast!(from_f64_to_u64_fail_nan, f64, u64, f64::NAN, None);
    test_cast!(from_f64_to_u64_fail_infinity, f64, u64, f64::INFINITY, None);
    test_cast!(
        from_f64_to_u64_fail_neg_infinity,
        f64,
        u64,
        f64::NEG_INFINITY,
        None
    );
}

mod to_u128_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_u128_success, u8, u128, 42, Some(42));
    test_cast!(from_u16_to_u128_success, u16, u128, 42, Some(42));
    test_cast!(from_u32_to_u128_success, u32, u128, 42, Some(42));
    test_cast!(from_u64_to_u128_success, u64, u128, 42, Some(42));
    test_cast!(from_u64_max, u64, u128, u64::MAX, Some(u64::MAX as u128));
    test_cast!(from_u128_identity, u128, u128, 42, Some(42));
    test_cast!(from_u128_max, u128, u128, u128::MAX, Some(u128::MAX));
    test_cast!(from_u128_min, u128, u128, 0, Some(0));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_u128_success_64bit, usize, u128, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_u128_max_64bit,
        usize,
        u128,
        usize::MAX,
        Some(usize::MAX as u128)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_u128_success_32bit, usize, u128, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_u128_max_32bit,
        usize,
        u128,
        usize::MAX,
        Some(usize::MAX as u128)
    );

    // From signed integers
    test_cast!(from_i8_to_u128_success, i8, u128, 42, Some(42));
    test_cast!(from_i8_to_u128_fail_negative, i8, u128, -42, None);
    test_cast!(from_i16_to_u128_success, i16, u128, 42, Some(42));
    test_cast!(from_i16_to_u128_fail_negative, i16, u128, -42, None);
    test_cast!(from_i32_to_u128_success, i32, u128, 42, Some(42));
    test_cast!(from_i32_to_u128_fail_negative, i32, u128, -42, None);
    test_cast!(from_i64_to_u128_success, i64, u128, 42, Some(42));
    test_cast!(from_i128_to_u128_success, i128, u128, 42, Some(42));
    test_cast!(from_i128_to_u128_fail_negative, i128, u128, -42, None);
    test_cast!(
        from_i128_max,
        i128,
        u128,
        i128::MAX,
        Some(i128::MAX as u128)
    );
    test_cast!(from_i128_min, i128, u128, i128::MIN, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_u128_success_64bit, isize, u128, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_u128_fail_negative_64bit,
        isize,
        u128,
        -42,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_u128_success_32bit, isize, u128, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_u128_fail_negative_32bit,
        isize,
        u128,
        -42,
        None
    );

    // From floats
    test_cast!(from_f32_to_u128_success, f32, u128, 42.0, Some(42));
    test_cast!(from_f32_to_u128_fail_negative, f32, u128, -42.0, None);
    test_cast!(
        from_f32_to_u128_fail_overflow,
        f32,
        u128,
        f32::MAX,
        Some(f32::MAX as u128)
    );
    test_cast!(from_f32_to_u128_fail_nan, f32, u128, f32::NAN, None);
    test_cast!(
        from_f32_to_u128_fail_infinity,
        f32,
        u128,
        f32::INFINITY,
        None
    );
    test_cast!(
        from_f32_to_u128_fail_neg_infinity,
        f32,
        u128,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_u128_success, f64, u128, 42.0, Some(42));
    test_cast!(from_f64_to_u128_fail_negative, f64, u128, -42.0, None);
    test_cast!(from_f64_to_u128_fail_overflow, f64, u128, f64::MAX, None);
    test_cast!(from_f64_to_u128_fail_nan, f64, u128, f64::NAN, None);
    test_cast!(
        from_f64_to_u128_fail_infinity,
        f64,
        u128,
        f64::INFINITY,
        None
    );
    test_cast!(
        from_f64_to_u128_fail_neg_infinity,
        f64,
        u128,
        f64::NEG_INFINITY,
        None
    );
}
