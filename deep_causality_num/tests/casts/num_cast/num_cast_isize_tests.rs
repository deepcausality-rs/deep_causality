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

mod to_isize_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_isize_success, u8, isize, 42, Some(42));
    test_cast!(from_u16_to_isize_success, u16, isize, 42, Some(42));
    test_cast!(from_u32_to_isize_success, u32, isize, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_u32_to_isize_max_64bit,
        u32,
        isize,
        u32::MAX,
        Some(u32::MAX as isize)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_u32_to_isize_fail_overflow_32bit,
        u32,
        isize,
        u32::MAX,
        None
    );
    test_cast!(from_u64_to_isize_success, u64, isize, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_u64_to_isize_fail_overflow_64bit,
        u64,
        isize,
        u64::MAX,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_u64_to_isize_fail_overflow_32bit,
        u64,
        isize,
        u64::MAX,
        None
    );
    test_cast!(from_u128_to_isize_success, u128, isize, 42, Some(42));
    test_cast!(
        from_u128_to_isize_fail_overflow,
        u128,
        isize,
        u128::MAX,
        None
    );
    test_cast!(from_usize_to_isize_success, usize, isize, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_isize_fail_overflow_64bit,
        usize,
        isize,
        usize::MAX,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_isize_fail_overflow_32bit,
        usize,
        isize,
        usize::MAX,
        None
    );

    // From signed integers
    test_cast!(from_i8_to_isize_success, i8, isize, 42, Some(42));
    test_cast!(from_i8_to_isize_negative, i8, isize, -42, Some(-42));
    test_cast!(from_i16_to_isize_success, i16, isize, 42, Some(42));
    test_cast!(from_i16_to_isize_negative, i16, isize, -42, Some(-42));
    test_cast!(from_i32_to_isize_success, i32, isize, 42, Some(42));
    test_cast!(from_i32_to_isize_negative, i32, isize, -42, Some(-42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_i32_to_isize_max_64bit,
        i32,
        isize,
        i32::MAX,
        Some(i32::MAX as isize)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_i32_to_isize_max_32bit,
        i32,
        isize,
        i32::MAX,
        Some(i32::MAX as isize)
    );
    test_cast!(from_i64_to_isize_success, i64, isize, 42, Some(42));
    test_cast!(from_i64_to_isize_negative, i64, isize, -42, Some(-42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_i64_to_isize_max_64bit,
        i64,
        isize,
        i64::MAX,
        Some(i64::MAX as isize)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_i64_to_isize_fail_overflow_32bit,
        i64,
        isize,
        i64::MAX,
        None
    );
    test_cast!(from_i128_to_isize_success, i128, isize, 42, Some(42));
    test_cast!(from_i128_to_isize_negative, i128, isize, -42, Some(-42));
    test_cast!(
        from_i128_to_isize_fail_overflow,
        i128,
        isize,
        i128::MAX,
        None
    );
    test_cast!(from_isize_identity, isize, isize, 42, Some(42));
    test_cast!(from_isize_identity_negative, isize, isize, -42, Some(-42));
    test_cast!(from_isize_max, isize, isize, isize::MAX, Some(isize::MAX));
    test_cast!(from_isize_min, isize, isize, isize::MIN, Some(isize::MIN));

    // From floats
    test_cast!(from_f32_to_isize_success, f32, isize, 42.0, Some(42));
    test_cast!(from_f32_to_isize_fail_negative, f32, isize, f32::MIN, None);
    test_cast!(from_f32_to_isize_fail_overflow, f32, isize, f32::MAX, None);
    test_cast!(from_f32_to_isize_fail_nan, f32, isize, f32::NAN, None);
    test_cast!(
        from_f32_to_isize_fail_infinity,
        f32,
        isize,
        f32::INFINITY,
        None
    );
    test_cast!(
        from_f32_to_isize_fail_neg_infinity,
        f32,
        isize,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_isize_success, f64, isize, 42.0, Some(42));
    test_cast!(from_f64_to_isize_fail_negative, f64, isize, f64::MIN, None);
    test_cast!(from_f64_to_isize_fail_overflow, f64, isize, f64::MAX, None);
    test_cast!(from_f64_to_isize_fail_nan, f64, isize, f64::NAN, None);
    test_cast!(
        from_f64_to_isize_fail_infinity,
        f64,
        isize,
        f64::INFINITY,
        None
    );
    test_cast!(
        from_f64_to_isize_fail_neg_infinity,
        f64,
        isize,
        f64::NEG_INFINITY,
        None
    );
}

mod to_i8_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_i8_success, u8, i8, 42, Some(42));
    test_cast!(from_u8_to_i8_fail_overflow, u8, i8, 200, None);
    test_cast!(from_u16_to_i8_success, u16, i8, 42, Some(42));
    test_cast!(from_u16_to_i8_fail_overflow, u16, i8, 200, None);
    test_cast!(from_u32_to_i8_success, u32, i8, 42, Some(42));
    test_cast!(from_u32_to_i8_fail_overflow, u32, i8, 200, None);
    test_cast!(from_u64_to_i8_success, u64, i8, 42, Some(42));
    test_cast!(from_u64_to_i8_fail_overflow, u64, i8, 200, None);
    test_cast!(from_u128_to_i8_success, u128, i8, 42, Some(42));
    test_cast!(from_u128_to_i8_fail_overflow, u128, i8, 200, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_i8_success_64bit, usize, i8, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_i8_fail_overflow_64bit, usize, i8, 200, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_i8_success_32bit, usize, i8, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_i8_fail_overflow_32bit, usize, i8, 200, None);

    // From signed integers
    test_cast!(from_i8_identity, i8, i8, 42, Some(42));
    test_cast!(from_i8_identity_negative, i8, i8, -42, Some(-42));
    test_cast!(from_i8_max, i8, i8, i8::MAX, Some(i8::MAX));
    test_cast!(from_i8_min, i8, i8, i8::MIN, Some(i8::MIN));
    test_cast!(from_i16_to_i8_success, i16, i8, 42, Some(42));
    test_cast!(from_i16_to_i8_fail_overflow, i16, i8, 300, None);
    test_cast!(from_i16_to_i8_fail_underflow, i16, i8, -300, None);
    test_cast!(from_i32_to_i8_success, i32, i8, 42, Some(42));
    test_cast!(from_i32_to_i8_fail_overflow, i32, i8, 300, None);
    test_cast!(from_i32_to_i8_fail_underflow, i32, i8, -300, None);
    test_cast!(from_i64_to_i8_success, i64, i8, 42, Some(42));
    test_cast!(from_i64_to_i8_fail_overflow, i64, i8, 300, None);
    test_cast!(from_i64_to_i8_fail_underflow, i64, i8, -300, None);
    test_cast!(from_i128_to_i8_success, i128, i8, 42, Some(42));
    test_cast!(from_i128_to_i8_fail_overflow, i128, i8, 300, None);
    test_cast!(from_i128_to_i8_fail_underflow, i128, i8, -300, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i8_success_64bit, isize, i8, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i8_fail_overflow_64bit, isize, i8, 300, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i8_fail_underflow_64bit, isize, i8, -300, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i8_success_32bit, isize, i8, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i8_fail_overflow_32bit, isize, i8, 300, None);
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i8_fail_underflow_32bit, isize, i8, -300, None);

    // From floats
    test_cast!(from_f32_to_i8_success, f32, i8, 42.0, Some(42));
    test_cast!(from_f32_to_i8_fail_negative, f32, i8, -130.0, None);
    test_cast!(from_f32_to_i8_fail_overflow, f32, i8, 130.0, None);
    test_cast!(from_f32_to_i8_fail_nan, f32, i8, f32::NAN, None);
    test_cast!(from_f32_to_i8_fail_infinity, f32, i8, f32::INFINITY, None);
    test_cast!(
        from_f32_to_i8_fail_neg_infinity,
        f32,
        i8,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_i8_success, f64, i8, 42.0, Some(42));
    test_cast!(from_f64_to_i8_fail_negative, f64, i8, -130.0, None);
    test_cast!(from_f64_to_i8_fail_overflow, f64, i8, 130.0, None);
    test_cast!(from_f64_to_i8_fail_nan, f64, i8, f64::NAN, None);
    test_cast!(from_f64_to_i8_fail_infinity, f64, i8, f64::INFINITY, None);
    test_cast!(
        from_f64_to_i8_fail_neg_infinity,
        f64,
        i8,
        f64::NEG_INFINITY,
        None
    );
}

mod to_i16_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_i16_success, u8, i16, 42, Some(42));
    test_cast!(from_u16_to_i16_success, u16, i16, 42, Some(42));
    test_cast!(from_u16_to_i16_fail_overflow, u16, i16, 40000, None);
    test_cast!(from_u32_to_i16_success, u32, i16, 42, Some(42));
    test_cast!(from_u32_to_i16_fail_overflow, u32, i16, 40000, None);
    test_cast!(from_u64_to_i16_success, u64, i16, 42, Some(42));
    test_cast!(from_u64_to_i16_fail_overflow, u64, i16, 40000, None);
    test_cast!(from_u128_to_i16_success, u128, i16, 42, Some(42));
    test_cast!(from_u128_to_i16_fail_overflow, u128, i16, 40000, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_i16_success_64bit, usize, i16, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_i16_fail_overflow_64bit,
        usize,
        i16,
        40000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_i16_success_32bit, usize, i16, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_i16_fail_overflow_32bit,
        usize,
        i16,
        40000,
        None
    );

    // From signed integers
    test_cast!(from_i8_to_i16_success, i8, i16, 42, Some(42));
    test_cast!(from_i8_to_i16_negative, i8, i16, -42, Some(-42));
    test_cast!(from_i16_identity, i16, i16, 42, Some(42));
    test_cast!(from_i16_identity_negative, i16, i16, -42, Some(-42));
    test_cast!(from_i16_max, i16, i16, i16::MAX, Some(i16::MAX));
    test_cast!(from_i16_min, i16, i16, i16::MIN, Some(i16::MIN));
    test_cast!(from_i32_to_i16_success, i32, i16, 42, Some(42));
    test_cast!(from_i32_to_i16_fail_overflow, i32, i16, 70000, None);
    test_cast!(from_i32_to_i16_fail_underflow, i32, i16, -70000, None);
    test_cast!(from_i64_to_i16_success, i64, i16, 42, Some(42));
    test_cast!(from_i64_to_i16_fail_overflow, i64, i16, 70000, None);
    test_cast!(from_i64_to_i16_fail_underflow, i64, i16, -70000, None);
    test_cast!(from_i128_to_i16_success, i128, i16, 42, Some(42));
    test_cast!(from_i128_to_i16_fail_overflow, i128, i16, 70000, None);
    test_cast!(from_i128_to_i16_fail_underflow, i128, i16, -70000, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i16_success_64bit, isize, i16, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_i16_fail_overflow_64bit,
        isize,
        i16,
        70000,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_i16_fail_underflow_64bit,
        isize,
        i16,
        -70000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i16_success_32bit, isize, i16, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_i16_fail_overflow_32bit,
        isize,
        i16,
        70000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_i16_fail_underflow_32bit,
        isize,
        i16,
        -70000,
        None
    );

    // From floats
    test_cast!(from_f32_to_i16_success, f32, i16, 42.0, Some(42));
    test_cast!(from_f32_to_i16_fail_negative, f32, i16, -40000.0, None);
    test_cast!(from_f32_to_i16_fail_overflow, f32, i16, 40000.0, None);
    test_cast!(from_f32_to_i16_fail_nan, f32, i16, f32::NAN, None);
    test_cast!(from_f32_to_i16_fail_infinity, f32, i16, f32::INFINITY, None);
    test_cast!(
        from_f32_to_i16_fail_neg_infinity,
        f32,
        i16,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_i16_success, f64, i16, 42.0, Some(42));
    test_cast!(from_f64_to_i16_fail_negative, f64, i16, -40000.0, None);
    test_cast!(from_f64_to_i16_fail_overflow, f64, i16, 40000.0, None);
    test_cast!(from_f64_to_i16_fail_nan, f64, i16, f64::NAN, None);
    test_cast!(from_f64_to_i16_fail_infinity, f64, i16, f64::INFINITY, None);
    test_cast!(
        from_f64_to_i16_fail_neg_infinity,
        f64,
        i16,
        f64::NEG_INFINITY,
        None
    );
}

mod to_i32_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_i32_success, u8, i32, 42, Some(42));
    test_cast!(from_u16_to_i32_success, u16, i32, 42, Some(42));
    test_cast!(from_u32_to_i32_success, u32, i32, 42, Some(42));
    test_cast!(from_u32_to_i32_fail_overflow, u32, i32, u32::MAX, None);
    test_cast!(from_u64_to_i32_success, u64, i32, 42, Some(42));
    test_cast!(from_u64_to_i32_fail_overflow, u64, i32, 5_000_000_000, None);
    test_cast!(from_u128_to_i32_success, u128, i32, 42, Some(42));
    test_cast!(
        from_u128_to_i32_fail_overflow,
        u128,
        i32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_i32_success_64bit, usize, i32, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_i32_fail_overflow_64bit,
        usize,
        i32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_i32_success_32bit, usize, i32, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_i32_fail_overflow_32bit,
        usize,
        i32,
        5_000_000_000,
        None
    );

    // From signed integers
    test_cast!(from_i8_to_i32_success, i8, i32, 42, Some(42));
    test_cast!(from_i8_to_i32_negative, i8, i32, -42, Some(-42));
    test_cast!(from_i16_to_i32_success, i16, i32, 42, Some(42));
    test_cast!(from_i16_to_i32_negative, i16, i32, -42, Some(-42));
    test_cast!(from_i32_identity, i32, i32, 42, Some(42));
    test_cast!(from_i32_identity_negative, i32, i32, -42, Some(-42));
    test_cast!(from_i32_max, i32, i32, i32::MAX, Some(i32::MAX));
    test_cast!(from_i32_min, i32, i32, i32::MIN, Some(i32::MIN));
    test_cast!(from_i64_to_i32_success, i64, i32, 42, Some(42));
    test_cast!(from_i64_to_i32_fail_overflow, i64, i32, 5_000_000_000, None);
    test_cast!(
        from_i64_to_i32_fail_underflow,
        i64,
        i32,
        -5_000_000_000,
        None
    );
    test_cast!(from_i128_to_i32_success, i128, i32, 42, Some(42));
    test_cast!(
        from_i128_to_i32_fail_overflow,
        i128,
        i32,
        5_000_000_000,
        None
    );
    test_cast!(
        from_i128_to_i32_fail_underflow,
        i128,
        i32,
        -5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i32_success_64bit, isize, i32, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_i32_fail_overflow_64bit,
        isize,
        i32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_i32_fail_underflow_64bit,
        isize,
        i32,
        -5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i32_success_32bit, isize, i32, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_i32_fail_overflow_32bit,
        isize,
        i32,
        5_000_000_000,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_i32_fail_underflow_32bit,
        isize,
        i32,
        -5_000_000_000,
        None
    );

    // From floats
    test_cast!(from_f32_to_i32_success, f32, i32, 42.0, Some(42));
    test_cast!(from_f32_to_i32_fail_negative, f32, i32, f32::MIN, None);
    test_cast!(from_f32_to_i32_fail_overflow, f32, i32, f32::MAX, None);
    test_cast!(from_f32_to_i32_fail_nan, f32, i32, f32::NAN, None);
    test_cast!(from_f32_to_i32_fail_infinity, f32, i32, f32::INFINITY, None);
    test_cast!(
        from_f32_to_i32_fail_neg_infinity,
        f32,
        i32,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_i32_success, f64, i32, 42.0, Some(42));
    test_cast!(from_f64_to_i32_fail_negative, f64, i32, f64::MIN, None);
    test_cast!(from_f64_to_i32_fail_overflow, f64, i32, f64::MAX, None);
    test_cast!(from_f64_to_i32_fail_nan, f64, i32, f64::NAN, None);
    test_cast!(from_f64_to_i32_fail_infinity, f64, i32, f64::INFINITY, None);
    test_cast!(
        from_f64_to_i32_fail_neg_infinity,
        f64,
        i32,
        f64::NEG_INFINITY,
        None
    );
}

mod to_i64_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_i64_success, u8, i64, 42, Some(42));
    test_cast!(from_u16_to_i64_success, u16, i64, 42, Some(42));
    test_cast!(from_u32_to_i64_success, u32, i64, 42, Some(42));
    test_cast!(from_u64_to_i64_success, u64, i64, 42, Some(42));
    test_cast!(from_u64_to_i64_fail_overflow, u64, i64, u64::MAX, None);
    test_cast!(from_u128_to_i64_success, u128, i64, 42, Some(42));
    test_cast!(from_u128_to_i64_fail_overflow, u128, i64, u128::MAX, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_i64_success_64bit, usize, i64, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_i64_fail_overflow_64bit,
        usize,
        i64,
        usize::MAX,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_i64_success_32bit, usize, i64, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_i64_fail_overflow_32bit,
        usize,
        i64,
        usize::MAX,
        None
    );

    // From signed integers
    test_cast!(from_i8_to_i64_success, i8, i64, 42, Some(42));
    test_cast!(from_i8_to_i64_negative, i8, i64, -42, Some(-42));
    test_cast!(from_i16_to_i64_success, i16, i64, 42, Some(42));
    test_cast!(from_i16_to_i64_negative, i16, i64, -42, Some(-42));
    test_cast!(from_i32_to_i64_success, i32, i64, 42, Some(42));
    test_cast!(from_i32_to_i64_negative, i32, i64, -42, Some(-42));
    test_cast!(from_i64_identity, i64, i64, 42, Some(42));
    test_cast!(from_i64_identity_negative, i64, i64, -42, Some(-42));
    test_cast!(from_i64_max, i64, i64, i64::MAX, Some(i64::MAX));
    test_cast!(from_i64_min, i64, i64, i64::MIN, Some(i64::MIN));
    test_cast!(from_i128_to_i64_success, i128, i64, 42, Some(42));
    test_cast!(from_i128_to_i64_fail_overflow, i128, i64, i128::MAX, None);
    test_cast!(from_i128_to_i64_fail_underflow, i128, i64, i128::MIN, None);
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i64_success_64bit, isize, i64, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i64_negative_64bit, isize, i64, -42, Some(-42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i64_success_32bit, isize, i64, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i64_negative_32bit, isize, i64, -42, Some(-42));

    // From floats
    test_cast!(from_f32_to_i64_success, f32, i64, 42.0, Some(42));
    test_cast!(from_f32_to_i64_fail_negative, f32, i64, f32::MIN, None);
    test_cast!(from_f32_to_i64_fail_overflow, f32, i64, f32::MAX, None);
    test_cast!(from_f32_to_i64_fail_nan, f32, i64, f32::NAN, None);
    test_cast!(from_f32_to_i64_fail_infinity, f32, i64, f32::INFINITY, None);
    test_cast!(
        from_f32_to_i64_fail_neg_infinity,
        f32,
        i64,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_i64_success, f64, i64, 42.0, Some(42));
    test_cast!(from_f64_to_i64_fail_negative, f64, i64, f64::MIN, None);
    test_cast!(from_f64_to_i64_fail_overflow, f64, i64, f64::MAX, None);
    test_cast!(from_f64_to_i64_fail_nan, f64, i64, f64::NAN, None);
    test_cast!(from_f64_to_i64_fail_infinity, f64, i64, f64::INFINITY, None);
    test_cast!(
        from_f64_to_i64_fail_neg_infinity,
        f64,
        i64,
        f64::NEG_INFINITY,
        None
    );
}

mod to_i128_tests {
    use super::*;
    // From unsigned integers
    test_cast!(from_u8_to_i128_success, u8, i128, 42, Some(42));
    test_cast!(from_u16_to_i128_success, u16, i128, 42, Some(42));
    test_cast!(from_u32_to_i128_success, u32, i128, 42, Some(42));
    test_cast!(from_u64_to_i128_success, u64, i128, 42, Some(42));
    test_cast!(from_u64_max, u64, i128, u64::MAX, Some(u64::MAX as i128));
    test_cast!(from_u128_identity, u128, i128, 42, Some(42));
    test_cast!(from_u128_max, u128, i128, u128::MAX, None);
    test_cast!(from_u128_min, u128, i128, 0, Some(0));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_usize_to_i128_success_64bit, usize, i128, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_usize_to_i128_max_64bit,
        usize,
        i128,
        usize::MAX,
        Some(usize::MAX as i128)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_usize_to_i128_success_32bit, usize, i128, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_usize_to_i128_max_32bit,
        usize,
        i128,
        usize::MAX,
        Some(usize::MAX as i128)
    );

    // From signed integers
    test_cast!(from_i8_to_i128_success, i8, i128, 42, Some(42));
    test_cast!(from_i8_to_i128_negative, i8, i128, -42, Some(-42));
    test_cast!(from_i16_to_i128_success, i16, i128, 42, Some(42));
    test_cast!(from_i16_to_i128_negative, i16, i128, -42, Some(-42));
    test_cast!(from_i32_to_i128_success, i32, i128, 42, Some(42));
    test_cast!(from_i32_to_i128_negative, i32, i128, -42, Some(-42));
    test_cast!(from_i64_to_i128_success, i64, i128, 42, Some(42));
    test_cast!(from_i64_to_i128_negative, i64, i128, -42, Some(-42));
    test_cast!(from_i128_identity, i128, i128, 42, Some(42));
    test_cast!(from_i128_identity_negative, i128, i128, -42, Some(-42));
    test_cast!(from_i128_max, i128, i128, i128::MAX, Some(i128::MAX));
    test_cast!(from_i128_min, i128, i128, i128::MIN, Some(i128::MIN));
    #[cfg(target_pointer_width = "64")]
    test_cast!(from_isize_to_i128_success_64bit, isize, i128, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_cast!(
        from_isize_to_i128_negative_64bit,
        isize,
        i128,
        -42,
        Some(-42)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast!(from_isize_to_i128_success_32bit, isize, i128, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_cast!(
        from_isize_to_i128_negative_32bit,
        isize,
        i128,
        -42,
        Some(-42)
    );

    // From floats
    test_cast!(from_f32_to_i128_success, f32, i128, 42.0, Some(42));
    test_cast!(from_f32_to_i128_fail_negative, f32, i128, f32::MIN, None);
    test_cast!(from_f32_to_i128_fail_overflow, f32, i128, f32::MAX, None);
    test_cast!(from_f32_to_i128_fail_nan, f32, i128, f32::NAN, None);
    test_cast!(
        from_f32_to_i128_fail_infinity,
        f32,
        i128,
        f32::INFINITY,
        None
    );
    test_cast!(
        from_f32_to_i128_fail_neg_infinity,
        f32,
        i128,
        f32::NEG_INFINITY,
        None
    );
    test_cast!(from_f64_to_i128_success, f64, i128, 42.0, Some(42));
    test_cast!(from_f64_to_i128_fail_negative, f64, i128, f64::MIN, None);
    test_cast!(from_f64_to_i128_fail_overflow, f64, i128, f64::MAX, None);
    test_cast!(from_f64_to_i128_fail_nan, f64, i128, f64::NAN, None);
    test_cast!(
        from_f64_to_i128_fail_infinity,
        f64,
        i128,
        f64::INFINITY,
        None
    );
    test_cast!(
        from_f64_to_i128_fail_neg_infinity,
        f64,
        i128,
        f64::NEG_INFINITY,
        None
    );
}
