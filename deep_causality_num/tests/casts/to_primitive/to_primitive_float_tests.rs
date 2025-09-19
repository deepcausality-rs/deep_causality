/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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

mod f32_to_tests {
    use super::*;

    test_to!(to_isize_ok, to_isize, f32, isize, 42.0, Some(42));
    test_to!(to_isize_fail, to_isize, f32, isize, f32::MAX, None);
    test_to!(
        to_isize_ok_min,
        to_isize,
        f32,
        isize,
        isize::MIN as f32,
        Some(isize::MIN)
    );
    test_to!(to_isize_fail_neg_inf, to_isize, f32, isize, f32::MIN, None);

    test_to!(to_i8_ok, to_i8, f32, i8, 42.0, Some(42));
    test_to!(to_i8_fail, to_i8, f32, i8, 128.0, None);
    test_to!(to_i8_ok_max, to_i8, f32, i8, 127.0, Some(127));
    test_to!(to_i8_ok_min, to_i8, f32, i8, -128.0, Some(-128));
    test_to!(to_i8_fail_neg, to_i8, f32, i8, -129.0, None);

    test_to!(to_i16_ok, to_i16, f32, i16, 42.0, Some(42));
    test_to!(to_i16_fail, to_i16, f32, i16, 32768.0, None);
    test_to!(to_i16_ok_max, to_i16, f32, i16, 32767.0, Some(32767));
    test_to!(to_i16_ok_min, to_i16, f32, i16, -32768.0, Some(-32768));
    test_to!(to_i16_fail_neg, to_i16, f32, i16, -32769.0, None);

    test_to!(to_i32_ok, to_i32, f32, i32, 42.0, Some(42));
    test_to!(to_i32_fail, to_i32, f32, i32, 2147483648.0, None);
    test_to!(
        to_i32_ok_min,
        to_i32,
        f32,
        i32,
        i32::MIN as f32,
        Some(i32::MIN)
    );
    test_to!(to_i32_fail_neg_inf, to_i32, f32, i32, f32::MIN, None);

    test_to!(to_i64_ok, to_i64, f32, i64, 42.0, Some(42));
    test_to!(to_i64_fail, to_i64, f32, i64, f32::MAX, None);
    test_to!(
        to_i64_ok_min,
        to_i64,
        f32,
        i64,
        i64::MIN as f32,
        Some(i64::MIN)
    );
    test_to!(to_i64_fail_neg_inf, to_i64, f32, i64, f32::MIN, None);

    test_to!(to_i128_ok, to_i128, f32, i128, 42.0, Some(42));
    test_to!(to_i128_fail, to_i128, f32, i128, f32::MAX, None);
    test_to!(
        to_i128_ok_min,
        to_i128,
        f32,
        i128,
        i128::MIN as f32,
        Some(i128::MIN)
    );
    test_to!(to_i128_fail_neg_inf, to_i128, f32, i128, f32::MIN, None);

    test_to!(to_usize_ok, to_usize, f32, usize, 42.0, Some(42));
    test_to!(to_usize_fail, to_usize, f32, usize, -1.0, None);
    test_to!(to_usize_ok_zero, to_usize, f32, usize, 0.0, Some(0));

    test_to!(to_u8_ok, to_u8, f32, u8, 42.0, Some(42));
    test_to!(to_u8_fail, to_u8, f32, u8, 256.0, None);
    test_to!(to_u8_ok_max, to_u8, f32, u8, 255.0, Some(255));
    test_to!(to_u8_ok_zero, to_u8, f32, u8, 0.0, Some(0));
    test_to!(to_u8_fail_neg, to_u8, f32, u8, -1.0, None);

    test_to!(to_u16_ok, to_u16, f32, u16, 42.0, Some(42));
    test_to!(to_u16_fail, to_u16, f32, u16, 65536.0, None);
    test_to!(to_u16_ok_max, to_u16, f32, u16, 65535.0, Some(65535));
    test_to!(to_u16_ok_zero, to_u16, f32, u16, 0.0, Some(0));
    test_to!(to_u16_fail_neg, to_u16, f32, u16, -1.0, None);

    test_to!(to_u32_ok, to_u32, f32, u32, 42.0, Some(42));
    test_to!(to_u32_fail, to_u32, f32, u32, 4294967296.0, None);
    test_to!(
        to_u32_ok_almost_max,
        to_u32,
        f32,
        u32,
        4294967040.0,
        Some(4294967040)
    );
    test_to!(to_u32_ok_zero, to_u32, f32, u32, 0.0, Some(0));
    test_to!(to_u32_fail_neg, to_u32, f32, u32, -1.0, None);

    test_to!(to_u64_ok, to_u64, f32, u64, 42.0, Some(42));
    test_to!(to_u64_fail, to_u64, f32, u64, f32::MAX, None);
    test_to!(
        to_u64_ok_large,
        to_u64,
        f32,
        u64,
        1152921504606846976.0,
        Some(1152921504606846976)
    );
    test_to!(to_u64_ok_zero, to_u64, f32, u64, 0.0, Some(0));
    test_to!(to_u64_fail_neg, to_u64, f32, u64, -1.0, None);

    test_to!(to_u128_ok, to_u128, f32, u128, 42.0, Some(42));
    test_to!(
        to_u128_max,
        to_u128,
        f32,
        u128,
        f32::MAX,
        Some(340282346638528859811704183484516925440u128)
    );
    test_to!(to_u128_fail_neg, to_u128, f32, u128, -1.0, None);
    test_to!(to_u128_ok_zero, to_u128, f32, u128, 0.0, Some(0));

    test_to!(to_f32_ok, to_f32, f32, f32, 42.0, Some(42.0));

    test_to!(to_f64_ok, to_f64, f32, f64, 42.0, Some(42.0));
}

mod f64_to_tests {
    use super::*;

    test_to!(to_isize_ok, to_isize, f64, isize, 42.0, Some(42));
    test_to!(to_isize_fail, to_isize, f64, isize, f64::MAX, None);
    test_to!(to_isize_ok_max, to_isize, f64, isize, 1000.0, Some(1000));
    test_to!(to_isize_ok_min, to_isize, f64, isize, -1000.0, Some(-1000));
    test_to!(
        to_isize_fail_max_plus_one,
        to_isize,
        f64,
        isize,
        isize::MAX as f64 + 1.0,
        None
    );
    test_to!(
        to_isize_fail_min_minus_one,
        to_isize,
        f64,
        isize,
        f64::MIN,
        None
    );

    test_to!(to_i8_ok, to_i8, f64, i8, 42.0, Some(42));
    test_to!(to_i8_fail, to_i8, f64, i8, 128.0, None);
    test_to!(to_i8_ok_max, to_i8, f64, i8, 127.0, Some(127));
    test_to!(to_i8_ok_min, to_i8, f64, i8, -128.0, Some(-128));
    test_to!(to_i8_fail_neg, to_i8, f64, i8, -129.0, None);

    test_to!(to_i16_ok, to_i16, f64, i16, 42.0, Some(42));
    test_to!(to_i16_fail, to_i16, f64, i16, 32768.0, None);
    test_to!(to_i16_ok_max, to_i16, f64, i16, 32767.0, Some(32767));
    test_to!(to_i16_ok_min, to_i16, f64, i16, -32768.0, Some(-32768));
    test_to!(to_i16_fail_neg, to_i16, f64, i16, -32769.0, None);

    test_to!(to_i32_ok, to_i32, f64, i32, 42.0, Some(42));
    test_to!(to_i32_fail, to_i32, f64, i32, 2147483648.0, None);
    test_to!(
        to_i32_ok_max,
        to_i32,
        f64,
        i32,
        2147483647.0,
        Some(2147483647)
    );
    test_to!(
        to_i32_ok_min,
        to_i32,
        f64,
        i32,
        -2147483648.0,
        Some(-2147483648)
    );
    test_to!(to_i32_fail_neg, to_i32, f64, i32, -2147483649.0, None);

    test_to!(to_i64_ok, to_i64, f64, i64, 42.0, Some(42));
    test_to!(to_i64_fail, to_i64, f64, i64, i64::MAX as f64 + 1.0, None);
    test_to!(
        to_i64_fail_max_as_f64,
        to_i64,
        f64,
        i64,
        i64::MAX as f64,
        None
    );
    test_to!(
        to_i64_ok_min_as_f64,
        to_i64,
        f64,
        i64,
        i64::MIN as f64,
        Some(i64::MIN)
    );

    test_to!(to_i128_ok, to_i128, f64, i128, 42.0, Some(42));
    test_to!(to_i128_fail, to_i128, f64, i128, f64::MAX, None);
    test_to!(
        to_i128_ok_min,
        to_i128,
        f64,
        i128,
        i128::MIN as f64,
        Some(i128::MIN)
    );
    test_to!(to_i128_fail_neg_inf, to_i128, f64, i128, f64::MIN, None);

    test_to!(to_usize_ok, to_usize, f64, usize, 42.0, Some(42));
    test_to!(to_usize_fail, to_usize, f64, usize, -1.0, None);
    test_to!(to_usize_ok_zero, to_usize, f64, usize, 0.0, Some(0));

    test_to!(to_u8_ok, to_u8, f64, u8, 42.0, Some(42));
    test_to!(to_u8_fail, to_u8, f64, u8, 256.0, None);
    test_to!(to_u8_ok_max, to_u8, f64, u8, 255.0, Some(255));
    test_to!(to_u8_ok_zero, to_u8, f64, u8, 0.0, Some(0));
    test_to!(to_u8_fail_neg, to_u8, f64, u8, -1.0, None);

    test_to!(to_u16_ok, to_u16, f64, u16, 42.0, Some(42));
    test_to!(to_u16_fail, to_u16, f64, u16, 65536.0, None);
    test_to!(to_u16_ok_max, to_u16, f64, u16, 65535.0, Some(65535));
    test_to!(to_u16_ok_zero, to_u16, f64, u16, 0.0, Some(0));
    test_to!(to_u16_fail_neg, to_u16, f64, u16, -1.0, None);

    test_to!(to_u32_ok, to_u32, f64, u32, 42.0, Some(42));
    test_to!(to_u32_fail, to_u32, f64, u32, 4294967296.0, None);
    test_to!(
        to_u32_ok_max,
        to_u32,
        f64,
        u32,
        4294967295.0,
        Some(4294967295)
    );
    test_to!(to_u32_ok_zero, to_u32, f64, u32, 0.0, Some(0));
    test_to!(to_u32_fail_neg, to_u32, f64, u32, -1.0, None);

    test_to!(to_u64_ok, to_u64, f64, u64, 42.0, Some(42));
    test_to!(to_u64_fail, to_u64, f64, u64, u64::MAX as f64 + 1.0, None);
    test_to!(
        to_u64_fail_max_as_f64,
        to_u64,
        f64,
        u64,
        u64::MAX as f64,
        None
    );
    test_to!(
        to_u64_ok_max_minus_a_bit,
        to_u64,
        f64,
        u64,
        1152921504606846976.0,
        Some(1152921504606846976)
    );
    test_to!(to_u64_ok_zero, to_u64, f64, u64, 0.0, Some(0));
    test_to!(to_u64_fail_neg, to_u64, f64, u64, -1.0, None);

    test_to!(to_u128_ok, to_u128, f64, u128, 42.0, Some(42));
    test_to!(to_u128_fail, to_u128, f64, u128, f64::MAX, None);
    test_to!(
        to_u128_ok_large,
        to_u128,
        f64,
        u128,
        1329227995784915872903807060280344576.0,
        Some(1329227995784915872903807060280344576)
    );
    test_to!(to_u128_ok_zero, to_u128, f64, u128, 0.0, Some(0));
    test_to!(to_u128_fail_neg, to_u128, f64, u128, -1.0, None);

    test_to!(to_f32_ok, to_f32, f64, f32, 42.0, Some(42.0));

    test_to!(to_f64_ok, to_f64, f64, f64, 42.0, Some(42.0));
}
