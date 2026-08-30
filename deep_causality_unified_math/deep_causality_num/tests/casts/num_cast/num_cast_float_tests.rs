/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::NumCast;

macro_rules! test_cast_float {
    ($name:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let expected: Option<$to> = $expected;
            let actual: Option<$to> = NumCast::from(v);

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

mod to_f32_tests {
    use super::*;
    // From unsigned integers
    test_cast_float!(from_u8_to_f32_success, u8, f32, 42, Some(42.0));
    test_cast_float!(from_u16_to_f32_success, u16, f32, 42, Some(42.0));
    test_cast_float!(from_u32_to_f32_success, u32, f32, 42, Some(42.0));
    test_cast_float!(from_u64_to_f32_success, u64, f32, 42, Some(42.0));
    test_cast_float!(from_u128_to_f32_success, u128, f32, 42, Some(42.0));
    #[cfg(target_pointer_width = "64")]
    test_cast_float!(from_usize_to_f32_success_64bit, usize, f32, 42, Some(42.0));
    #[cfg(target_pointer_width = "32")]
    test_cast_float!(from_usize_to_f32_success_32bit, usize, f32, 42, Some(42.0));

    // From signed integers
    test_cast_float!(from_i8_to_f32_success, i8, f32, 42, Some(42.0));
    test_cast_float!(from_i8_to_f32_negative, i8, f32, -42, Some(-42.0));
    test_cast_float!(from_i16_to_f32_success, i16, f32, 42, Some(42.0));
    test_cast_float!(from_i16_to_f32_negative, i16, f32, -42, Some(-42.0));
    test_cast_float!(from_i32_to_f32_success, i32, f32, 42, Some(42.0));
    test_cast_float!(from_i32_to_f32_negative, i32, f32, -42, Some(-42.0));
    test_cast_float!(from_i64_to_f32_success, i64, f32, 42, Some(42.0));
    test_cast_float!(from_i64_to_f32_negative, i64, f32, -42, Some(-42.0));
    test_cast_float!(from_i128_to_f32_success, i128, f32, 42, Some(42.0));
    test_cast_float!(from_i128_to_f32_negative, i128, f32, -42, Some(-42.0));
    #[cfg(target_pointer_width = "64")]
    test_cast_float!(from_isize_to_f32_success_64bit, isize, f32, 42, Some(42.0));
    #[cfg(target_pointer_width = "64")]
    test_cast_float!(
        from_isize_to_f32_negative_64bit,
        isize,
        f32,
        -42,
        Some(-42.0)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast_float!(from_isize_to_f32_success_32bit, isize, f32, 42, Some(42.0));
    #[cfg(target_pointer_width = "32")]
    test_cast_float!(
        from_isize_to_f32_negative_32bit,
        isize,
        f32,
        -42,
        Some(-42.0)
    );

    // From floats
    test_cast_float!(from_f32_identity, f32, f32, 42.0, Some(42.0));
    test_cast_float!(from_f32_identity_negative, f32, f32, -42.0, Some(-42.0));
    test_cast_float!(from_f32_identity_nan, f32, f32, f32::NAN, Some(f32::NAN));
    test_cast_float!(
        from_f32_identity_infinity,
        f32,
        f32,
        f32::INFINITY,
        Some(f32::INFINITY)
    );
    test_cast_float!(
        from_f32_identity_neg_infinity,
        f32,
        f32,
        f32::NEG_INFINITY,
        Some(f32::NEG_INFINITY)
    );
    test_cast_float!(from_f64_to_f32_success, f64, f32, 42.0, Some(42.0));
    test_cast_float!(from_f64_to_f32_negative, f64, f32, -42.0, Some(-42.0));
    test_cast_float!(from_f64_to_f32_nan, f64, f32, f64::NAN, Some(f32::NAN));
    test_cast_float!(
        from_f64_to_f32_infinity,
        f64,
        f32,
        f64::INFINITY,
        Some(f32::INFINITY)
    );
    test_cast_float!(
        from_f64_to_f32_neg_infinity,
        f64,
        f32,
        f64::NEG_INFINITY,
        Some(f32::NEG_INFINITY)
    );
    test_cast_float!(
        from_f64_to_f32_precision_loss,
        f64,
        f32,
        1.2345678901234567,
        Some(1.2345679)
    ); // Example of precision loss
}

mod to_f64_tests {
    use super::*;
    // From unsigned integers
    test_cast_float!(from_u8_to_f64_success, u8, f64, 42, Some(42.0));
    test_cast_float!(from_u16_to_f64_success, u16, f64, 42, Some(42.0));
    test_cast_float!(from_u32_to_f64_success, u32, f64, 42, Some(42.0));
    test_cast_float!(from_u64_to_f64_success, u64, f64, 42, Some(42.0));
    test_cast_float!(from_u128_to_f64_success, u128, f64, 42, Some(42.0));
    #[cfg(target_pointer_width = "64")]
    test_cast_float!(from_usize_to_f64_success_64bit, usize, f64, 42, Some(42.0));
    #[cfg(target_pointer_width = "32")]
    test_cast_float!(from_usize_to_f64_success_32bit, usize, f64, 42, Some(42.0));

    // From signed integers
    test_cast_float!(from_i8_to_f64_success, i8, f64, 42, Some(42.0));
    test_cast_float!(from_i8_to_f64_negative, i8, f64, -42, Some(-42.0));
    test_cast_float!(from_i16_to_f64_success, i16, f64, 42, Some(42.0));
    test_cast_float!(from_i16_to_f64_negative, i16, f64, -42, Some(-42.0));
    test_cast_float!(from_i32_to_f64_success, i32, f64, 42, Some(42.0));
    test_cast_float!(from_i32_to_f64_negative, i32, f64, -42, Some(-42.0));
    test_cast_float!(from_i64_to_f64_success, i64, f64, 42, Some(42.0));
    test_cast_float!(from_i64_to_f64_negative, i64, f64, -42, Some(-42.0));
    test_cast_float!(from_i128_to_f64_success, i128, f64, 42, Some(42.0));
    test_cast_float!(from_i128_to_f64_negative, i128, f64, -42, Some(-42.0));
    #[cfg(target_pointer_width = "64")]
    test_cast_float!(from_isize_to_f64_success_64bit, isize, f64, 42, Some(42.0));
    #[cfg(target_pointer_width = "64")]
    test_cast_float!(
        from_isize_to_f64_negative_64bit,
        isize,
        f64,
        -42,
        Some(-42.0)
    );
    #[cfg(target_pointer_width = "32")]
    test_cast_float!(from_isize_to_f64_success_32bit, isize, f64, 42, Some(42.0));
    #[cfg(target_pointer_width = "32")]
    test_cast_float!(
        from_isize_to_f64_negative_32bit,
        isize,
        f64,
        -42,
        Some(-42.0)
    );

    // From floats
    test_cast_float!(from_f32_to_f64_success, f32, f64, 42.0, Some(42.0));
    test_cast_float!(from_f32_to_f64_negative, f32, f64, -42.0, Some(-42.0));
    test_cast_float!(from_f32_to_f64_nan, f32, f64, f32::NAN, Some(f64::NAN));
    test_cast_float!(
        from_f32_to_f64_infinity,
        f32,
        f64,
        f32::INFINITY,
        Some(f64::INFINITY)
    );
    test_cast_float!(
        from_f32_to_f64_neg_infinity,
        f32,
        f64,
        f32::NEG_INFINITY,
        Some(f64::NEG_INFINITY)
    );
    test_cast_float!(from_f64_identity, f64, f64, 42.0, Some(42.0));
    test_cast_float!(from_f64_identity_negative, f64, f64, -42.0, Some(-42.0));
    test_cast_float!(from_f64_identity_nan, f64, f64, f64::NAN, Some(f64::NAN));
    test_cast_float!(
        from_f64_identity_infinity,
        f64,
        f64,
        f64::INFINITY,
        Some(f64::INFINITY)
    );
    test_cast_float!(
        from_f64_identity_neg_infinity,
        f64,
        f64,
        f64::NEG_INFINITY,
        Some(f64::NEG_INFINITY)
    );
}
