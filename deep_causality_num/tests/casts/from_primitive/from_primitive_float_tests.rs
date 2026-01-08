/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::FromPrimitive;

// Macro to test conversion from signed integers to float
macro_rules! test_from_signed_primitive_to_float {
    ($test_name:ident, $from_type:ty, $to_type:ty, $from_method:ident) => {
        #[test]
        fn $test_name() {
            let zero: $from_type = 0;
            assert_eq!(<$to_type>::$from_method(zero), Some(zero as $to_type));

            let positive: $from_type = 42;
            assert_eq!(
                <$to_type>::$from_method(positive),
                Some(positive as $to_type)
            );

            let negative: $from_type = -42;
            assert_eq!(
                <$to_type>::$from_method(negative),
                Some(negative as $to_type)
            );

            let max_val: $from_type = <$from_type>::MAX;
            assert_eq!(<$to_type>::$from_method(max_val), Some(max_val as $to_type));

            let min_val: $from_type = <$from_type>::MIN;
            assert_eq!(<$to_type>::$from_method(min_val), Some(min_val as $to_type));
        }
    };
}

// Macro to test conversion from unsigned integers to float
macro_rules! test_from_unsigned_primitive_to_float {
    ($test_name:ident, $from_type:ty, $to_type:ty, $from_method:ident) => {
        #[test]
        fn $test_name() {
            let zero: $from_type = 0;
            assert_eq!(<$to_type>::$from_method(zero), Some(zero as $to_type));

            let positive: $from_type = 42;
            assert_eq!(
                <$to_type>::$from_method(positive),
                Some(positive as $to_type)
            );

            let max_val: $from_type = <$from_type>::MAX;
            assert_eq!(<$to_type>::$from_method(max_val), Some(max_val as $to_type));
        }
    };
}

// Macro to test conversion from float to float
macro_rules! test_from_float_to_float {
    ($test_name:ident, $from_type:ty, $to_type:ty, $from_method:ident) => {
        #[test]
        fn $test_name() {
            let zero: $from_type = 0.0;
            assert_eq!(<$to_type>::$from_method(zero), Some(zero as $to_type));

            let positive: $from_type = 42.42;
            assert_eq!(
                <$to_type>::$from_method(positive),
                Some(positive as $to_type)
            );

            let negative: $from_type = -42.42;
            assert_eq!(
                <$to_type>::$from_method(negative),
                Some(negative as $to_type)
            );

            let max_val: $from_type = <$from_type>::MAX;
            assert_eq!(<$to_type>::$from_method(max_val), Some(max_val as $to_type));

            let min_val: $from_type = <$from_type>::MIN;
            assert_eq!(<$to_type>::$from_method(min_val), Some(min_val as $to_type));

            let infinity: $from_type = <$from_type>::INFINITY;
            assert_eq!(
                <$to_type>::$from_method(infinity),
                Some(<$to_type>::INFINITY)
            );

            let neg_infinity: $from_type = <$from_type>::NEG_INFINITY;
            assert_eq!(
                <$to_type>::$from_method(neg_infinity),
                Some(<$to_type>::NEG_INFINITY)
            );

            let nan: $from_type = <$from_type>::NAN;
            let res = <$to_type>::$from_method(nan);
            assert!(res.is_some());
            assert!(res.unwrap().is_nan());
        }
    };
}

mod tests_for_f32 {
    use super::*;

    test_from_signed_primitive_to_float!(test_f32_from_isize, isize, f32, from_isize);
    test_from_signed_primitive_to_float!(test_f32_from_i8, i8, f32, from_i8);
    test_from_signed_primitive_to_float!(test_f32_from_i16, i16, f32, from_i16);
    test_from_signed_primitive_to_float!(test_f32_from_i32, i32, f32, from_i32);
    test_from_signed_primitive_to_float!(test_f32_from_i64, i64, f32, from_i64);
    test_from_signed_primitive_to_float!(test_f32_from_i128, i128, f32, from_i128);

    test_from_unsigned_primitive_to_float!(test_f32_from_usize, usize, f32, from_usize);
    test_from_unsigned_primitive_to_float!(test_f32_from_u8, u8, f32, from_u8);
    test_from_unsigned_primitive_to_float!(test_f32_from_u16, u16, f32, from_u16);
    test_from_unsigned_primitive_to_float!(test_f32_from_u32, u32, f32, from_u32);
    test_from_unsigned_primitive_to_float!(test_f32_from_u64, u64, f32, from_u64);
    test_from_unsigned_primitive_to_float!(test_f32_from_u128, u128, f32, from_u128);

    test_from_float_to_float!(test_f32_from_f32, f32, f32, from_f32);
    test_from_float_to_float!(test_f32_from_f64, f64, f32, from_f64);
}

mod tests_for_f64 {
    use super::*;

    test_from_signed_primitive_to_float!(test_f64_from_isize, isize, f64, from_isize);
    test_from_signed_primitive_to_float!(test_f64_from_i8, i8, f64, from_i8);
    test_from_signed_primitive_to_float!(test_f64_from_i16, i16, f64, from_i16);
    test_from_signed_primitive_to_float!(test_f64_from_i32, i32, f64, from_i32);
    test_from_signed_primitive_to_float!(test_f64_from_i64, i64, f64, from_i64);
    test_from_signed_primitive_to_float!(test_f64_from_i128, i128, f64, from_i128);

    test_from_unsigned_primitive_to_float!(test_f64_from_usize, usize, f64, from_usize);
    test_from_unsigned_primitive_to_float!(test_f64_from_u8, u8, f64, from_u8);
    test_from_unsigned_primitive_to_float!(test_f64_from_u16, u16, f64, from_u16);
    test_from_unsigned_primitive_to_float!(test_f64_from_u32, u32, f64, from_u32);
    test_from_unsigned_primitive_to_float!(test_f64_from_u64, u64, f64, from_u64);
    test_from_unsigned_primitive_to_float!(test_f64_from_u128, u128, f64, from_u128);

    test_from_float_to_float!(test_f64_from_f32, f32, f64, from_f32);
    test_from_float_to_float!(test_f64_from_f64, f64, f64, from_f64);
}
