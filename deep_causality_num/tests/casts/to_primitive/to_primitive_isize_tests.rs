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

mod isize_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, isize, isize, 42, Some(42));
    test_to!(
        to_isize_min,
        to_isize,
        isize,
        isize,
        isize::MIN,
        Some(isize::MIN)
    );
    test_to!(
        to_isize_max,
        to_isize,
        isize,
        isize,
        isize::MAX,
        Some(isize::MAX)
    );

    test_to!(to_i8_ok, to_i8, isize, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, isize, i8, i8::MAX as isize + 1, None);
    test_to!(to_i8_fail_neg, to_i8, isize, i8, i8::MIN as isize - 1, None);
    test_to!(to_i8_min, to_i8, isize, i8, i8::MIN as isize, Some(i8::MIN));
    test_to!(to_i8_max, to_i8, isize, i8, i8::MAX as isize, Some(i8::MAX));

    test_to!(to_i16_ok, to_i16, isize, i16, 42, Some(42));
    test_to!(
        to_i16_fail_pos,
        to_i16,
        isize,
        i16,
        i16::MAX as isize + 1,
        None
    );
    test_to!(
        to_i16_fail_neg,
        to_i16,
        isize,
        i16,
        i16::MIN as isize - 1,
        None
    );
    test_to!(
        to_i16_min,
        to_i16,
        isize,
        i16,
        i16::MIN as isize,
        Some(i16::MIN)
    );
    test_to!(
        to_i16_max,
        to_i16,
        isize,
        i16,
        i16::MAX as isize,
        Some(i16::MAX)
    );

    test_to!(to_i32_ok, to_i32, isize, i32, 42, Some(42));
    test_to!(
        to_i32_fail_pos,
        to_i32,
        isize,
        i32,
        i32::MAX as isize + 1,
        None
    );
    test_to!(
        to_i32_fail_neg,
        to_i32,
        isize,
        i32,
        i32::MIN as isize - 1,
        None
    );
    test_to!(
        to_i32_min,
        to_i32,
        isize,
        i32,
        i32::MIN as isize,
        Some(i32::MIN)
    );
    test_to!(
        to_i32_max,
        to_i32,
        isize,
        i32,
        i32::MAX as isize,
        Some(i32::MAX)
    );

    test_to!(to_i64_ok, to_i64, isize, i64, 42, Some(42));
    test_to!(
        to_i64_min,
        to_i64,
        isize,
        i64,
        isize::MIN,
        Some(isize::MIN as i64)
    );
    test_to!(
        to_i64_max,
        to_i64,
        isize,
        i64,
        isize::MAX,
        Some(isize::MAX as i64)
    );

    test_to!(to_i128_ok, to_i128, isize, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        isize,
        i128,
        isize::MIN,
        Some(isize::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        isize,
        i128,
        isize::MAX,
        Some(isize::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, isize, usize, 42, Some(42));
    test_to!(to_usize_fail_neg, to_usize, isize, usize, -1, None);
    test_to!(to_usize_min, to_usize, isize, usize, 0, Some(0));
    test_to!(
        to_usize_max,
        to_usize,
        isize,
        usize,
        isize::MAX,
        Some(isize::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, isize, u8, 42, Some(42));
    test_to!(to_u8_fail_neg, to_u8, isize, u8, -1, None);
    test_to!(to_u8_fail_pos, to_u8, isize, u8, u8::MAX as isize + 1, None);
    test_to!(to_u8_min, to_u8, isize, u8, 0, Some(0));
    test_to!(to_u8_max, to_u8, isize, u8, u8::MAX as isize, Some(u8::MAX));

    test_to!(to_u16_ok, to_u16, isize, u16, 42, Some(42));
    test_to!(to_u16_fail_neg, to_u16, isize, u16, -1, None);
    test_to!(
        to_u16_fail_pos,
        to_u16,
        isize,
        u16,
        u16::MAX as isize + 1,
        None
    );
    test_to!(to_u16_min, to_u16, isize, u16, 0, Some(0));
    test_to!(
        to_u16_max,
        to_u16,
        isize,
        u16,
        u16::MAX as isize,
        Some(u16::MAX)
    );

    test_to!(to_u32_ok, to_u32, isize, u32, 42, Some(42));
    test_to!(to_u32_fail_neg, to_u32, isize, u32, -1, None);
    test_to!(
        to_u32_fail_pos,
        to_u32,
        isize,
        u32,
        u32::MAX as isize + 1,
        None
    );
    test_to!(to_u32_min, to_u32, isize, u32, 0, Some(0));
    test_to!(
        to_u32_max,
        to_u32,
        isize,
        u32,
        u32::MAX as isize,
        Some(u32::MAX)
    );

    test_to!(to_u64_ok, to_u64, isize, u64, 42, Some(42));
    test_to!(to_u64_fail_neg, to_u64, isize, u64, -1, None);
    test_to!(to_u64_min, to_u64, isize, u64, 0, Some(0));
    test_to!(
        to_u64_max,
        to_u64,
        isize,
        u64,
        isize::MAX,
        Some(isize::MAX as u64)
    );

    test_to!(to_u128_ok, to_u128, isize, u128, 42, Some(42));
    test_to!(to_u128_fail_neg, to_u128, isize, u128, -1, None);
    test_to!(to_u128_min, to_u128, isize, u128, 0, Some(0));
    test_to!(
        to_u128_max,
        to_u128,
        isize,
        u128,
        isize::MAX,
        Some(isize::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, isize, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, isize, f64, 42, Some(42.0f64));
}

mod i8_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, i8, isize, 42, Some(42));
    test_to!(
        to_isize_min,
        to_isize,
        i8,
        isize,
        i8::MIN,
        Some(i8::MIN as isize)
    );
    test_to!(
        to_isize_max,
        to_isize,
        i8,
        isize,
        i8::MAX,
        Some(i8::MAX as isize)
    );

    test_to!(to_i8_ok, to_i8, i8, i8, 42, Some(42));
    test_to!(to_i8_min, to_i8, i8, i8, i8::MIN, Some(i8::MIN));
    test_to!(to_i8_max, to_i8, i8, i8, i8::MAX, Some(i8::MAX));

    test_to!(to_i16_ok, to_i16, i8, i16, 42, Some(42));
    test_to!(to_i16_min, to_i16, i8, i16, i8::MIN, Some(i8::MIN as i16));
    test_to!(to_i16_max, to_i16, i8, i16, i8::MAX, Some(i8::MAX as i16));

    test_to!(to_i32_ok, to_i32, i8, i32, 42, Some(42));
    test_to!(to_i32_min, to_i32, i8, i32, i8::MIN, Some(i8::MIN as i32));
    test_to!(to_i32_max, to_i32, i8, i32, i8::MAX, Some(i8::MAX as i32));

    test_to!(to_i64_ok, to_i64, i8, i64, 42, Some(42));
    test_to!(to_i64_min, to_i64, i8, i64, i8::MIN, Some(i8::MIN as i64));
    test_to!(to_i64_max, to_i64, i8, i64, i8::MAX, Some(i8::MAX as i64));

    test_to!(to_i128_ok, to_i128, i8, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        i8,
        i128,
        i8::MIN,
        Some(i8::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        i8,
        i128,
        i8::MAX,
        Some(i8::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, i8, usize, 42, Some(42));
    test_to!(to_usize_fail_neg, to_usize, i8, usize, -1, None);
    test_to!(to_usize_min, to_usize, i8, usize, 0, Some(0));
    test_to!(
        to_usize_max,
        to_usize,
        i8,
        usize,
        i8::MAX,
        Some(i8::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, i8, u8, 42, Some(42));
    test_to!(to_u8_fail_neg, to_u8, i8, u8, -1, None);
    test_to!(to_u8_min, to_u8, i8, u8, 0, Some(0));
    test_to!(to_u8_max, to_u8, i8, u8, i8::MAX, Some(i8::MAX as u8));

    test_to!(to_u16_ok, to_u16, i8, u16, 42, Some(42));
    test_to!(to_u16_fail_neg, to_u16, i8, u16, -1, None);
    test_to!(to_u16_min, to_u16, i8, u16, 0, Some(0));
    test_to!(to_u16_max, to_u16, i8, u16, i8::MAX, Some(i8::MAX as u16));

    test_to!(to_u32_ok, to_u32, i8, u32, 42, Some(42));
    test_to!(to_u32_fail_neg, to_u32, i8, u32, -1, None);
    test_to!(to_u32_min, to_u32, i8, u32, 0, Some(0));
    test_to!(to_u32_max, to_u32, i8, u32, i8::MAX, Some(i8::MAX as u32));

    test_to!(to_u64_ok, to_u64, i8, u64, 42, Some(42));
    test_to!(to_u64_fail_neg, to_u64, i8, u64, -1, None);
    test_to!(to_u64_min, to_u64, i8, u64, 0, Some(0));
    test_to!(to_u64_max, to_u64, i8, u64, i8::MAX, Some(i8::MAX as u64));

    test_to!(to_u128_ok, to_u128, i8, u128, 42, Some(42));
    test_to!(to_u128_fail_neg, to_u128, i8, u128, -1, None);
    test_to!(to_u128_min, to_u128, i8, u128, 0, Some(0));
    test_to!(
        to_u128_max,
        to_u128,
        i8,
        u128,
        i8::MAX,
        Some(i8::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, i8, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, i8, f64, 42, Some(42.0f64));
}

mod i16_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, i16, isize, 42, Some(42));
    test_to!(
        to_isize_min,
        to_isize,
        i16,
        isize,
        i16::MIN,
        Some(i16::MIN as isize)
    );
    test_to!(
        to_isize_max,
        to_isize,
        i16,
        isize,
        i16::MAX,
        Some(i16::MAX as isize)
    );

    test_to!(to_i8_ok, to_i8, i16, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, i16, i8, i8::MAX as i16 + 1, None);
    test_to!(to_i8_fail_neg, to_i8, i16, i8, i8::MIN as i16 - 1, None);
    test_to!(to_i8_min, to_i8, i16, i8, i8::MIN as i16, Some(i8::MIN));
    test_to!(to_i8_max, to_i8, i16, i8, i8::MAX as i16, Some(i8::MAX));

    test_to!(to_i16_ok, to_i16, i16, i16, 42, Some(42));
    test_to!(to_i16_min, to_i16, i16, i16, i16::MIN, Some(i16::MIN));
    test_to!(to_i16_max, to_i16, i16, i16, i16::MAX, Some(i16::MAX));

    test_to!(to_i32_ok, to_i32, i16, i32, 42, Some(42));
    test_to!(
        to_i32_min,
        to_i32,
        i16,
        i32,
        i16::MIN,
        Some(i16::MIN as i32)
    );
    test_to!(
        to_i32_max,
        to_i32,
        i16,
        i32,
        i16::MAX,
        Some(i16::MAX as i32)
    );

    test_to!(to_i64_ok, to_i64, i16, i64, 42, Some(42));
    test_to!(
        to_i64_min,
        to_i64,
        i16,
        i64,
        i16::MIN,
        Some(i16::MIN as i64)
    );
    test_to!(
        to_i64_max,
        to_i64,
        i16,
        i64,
        i16::MAX,
        Some(i16::MAX as i64)
    );

    test_to!(to_i128_ok, to_i128, i16, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        i16,
        i128,
        i16::MIN,
        Some(i16::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        i16,
        i128,
        i16::MAX,
        Some(i16::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, i16, usize, 42, Some(42));
    test_to!(to_usize_fail_neg, to_usize, i16, usize, -1, None);
    test_to!(to_usize_min, to_usize, i16, usize, 0, Some(0));
    test_to!(
        to_usize_max,
        to_usize,
        i16,
        usize,
        i16::MAX,
        Some(i16::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, i16, u8, 42, Some(42));
    test_to!(to_u8_fail_neg, to_u8, i16, u8, -1, None);
    test_to!(to_u8_fail_pos, to_u8, i16, u8, u8::MAX as i16 + 1, None);
    test_to!(to_u8_min, to_u8, i16, u8, 0, Some(0));
    test_to!(to_u8_max, to_u8, i16, u8, u8::MAX as i16, Some(u8::MAX));

    test_to!(to_u16_ok, to_u16, i16, u16, 42, Some(42));
    test_to!(to_u16_fail_neg, to_u16, i16, u16, -1, None);
    test_to!(to_u16_min, to_u16, i16, u16, 0, Some(0));
    test_to!(
        to_u16_max,
        to_u16,
        i16,
        u16,
        i16::MAX,
        Some(i16::MAX as u16)
    );

    test_to!(to_u32_ok, to_u32, i16, u32, 42, Some(42));
    test_to!(to_u32_fail_neg, to_u32, i16, u32, -1, None);
    test_to!(to_u32_min, to_u32, i16, u32, 0, Some(0));
    test_to!(
        to_u32_max,
        to_u32,
        i16,
        u32,
        i16::MAX,
        Some(i16::MAX as u32)
    );

    test_to!(to_u64_ok, to_u64, i16, u64, 42, Some(42));
    test_to!(to_u64_fail_neg, to_u64, i16, u64, -1, None);
    test_to!(to_u64_min, to_u64, i16, u64, 0, Some(0));
    test_to!(
        to_u64_max,
        to_u64,
        i16,
        u64,
        i16::MAX,
        Some(i16::MAX as u64)
    );

    test_to!(to_u128_ok, to_u128, i16, u128, 42, Some(42));
    test_to!(to_u128_fail_neg, to_u128, i16, u128, -1, None);
    test_to!(to_u128_min, to_u128, i16, u128, 0, Some(0));
    test_to!(
        to_u128_max,
        to_u128,
        i16,
        u128,
        i16::MAX,
        Some(i16::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, i16, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, i16, f64, 42, Some(42.0f64));
}

mod i32_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, i32, isize, 42, Some(42));

    test_to!(
        to_isize_min,
        to_isize,
        i32,
        isize,
        i32::MIN,
        Some(i32::MIN as isize)
    );
    test_to!(
        to_isize_max,
        to_isize,
        i32,
        isize,
        i32::MAX,
        Some(i32::MAX as isize)
    );

    test_to!(to_i8_ok, to_i8, i32, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, i32, i8, i8::MAX as i32 + 1, None);
    test_to!(to_i8_fail_neg, to_i8, i32, i8, i8::MIN as i32 - 1, None);
    test_to!(to_i8_min, to_i8, i32, i8, i8::MIN as i32, Some(i8::MIN));
    test_to!(to_i8_max, to_i8, i32, i8, i8::MAX as i32, Some(i8::MAX));

    test_to!(to_i16_ok, to_i16, i32, i16, 42, Some(42));
    test_to!(to_i16_fail_pos, to_i16, i32, i16, i16::MAX as i32 + 1, None);
    test_to!(to_i16_fail_neg, to_i16, i32, i16, i16::MIN as i32 - 1, None);
    test_to!(
        to_i16_min,
        to_i16,
        i32,
        i16,
        i16::MIN as i32,
        Some(i16::MIN)
    );
    test_to!(
        to_i16_max,
        to_i16,
        i32,
        i16,
        i16::MAX as i32,
        Some(i16::MAX)
    );

    test_to!(to_i32_ok, to_i32, i32, i32, 42, Some(42));
    test_to!(to_i32_min, to_i32, i32, i32, i32::MIN, Some(i32::MIN));
    test_to!(to_i32_max, to_i32, i32, i32, i32::MAX, Some(i32::MAX));

    test_to!(to_i64_ok, to_i64, i32, i64, 42, Some(42));
    test_to!(
        to_i64_min,
        to_i64,
        i32,
        i64,
        i32::MIN,
        Some(i32::MIN as i64)
    );
    test_to!(
        to_i64_max,
        to_i64,
        i32,
        i64,
        i32::MAX,
        Some(i32::MAX as i64)
    );

    test_to!(to_i128_ok, to_i128, i32, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        i32,
        i128,
        i32::MIN,
        Some(i32::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        i32,
        i128,
        i32::MAX,
        Some(i32::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, i32, usize, 42, Some(42));
    test_to!(to_usize_fail_neg, to_usize, i32, usize, -1, None);
    test_to!(to_usize_min, to_usize, i32, usize, 0, Some(0));
    test_to!(
        to_usize_max,
        to_usize,
        i32,
        usize,
        i32::MAX,
        Some(i32::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, i32, u8, 42, Some(42));
    test_to!(to_u8_fail_neg, to_u8, i32, u8, -1, None);
    test_to!(to_u8_fail_pos, to_u8, i32, u8, u8::MAX as i32 + 1, None);
    test_to!(to_u8_min, to_u8, i32, u8, 0, Some(0));
    test_to!(to_u8_max, to_u8, i32, u8, u8::MAX as i32, Some(u8::MAX));

    test_to!(to_u16_ok, to_u16, i32, u16, 42, Some(42));
    test_to!(to_u16_fail_neg, to_u16, i32, u16, -1, None);
    test_to!(to_u16_fail_pos, to_u16, i32, u16, u16::MAX as i32 + 1, None);
    test_to!(to_u16_min, to_u16, i32, u16, 0, Some(0));
    test_to!(
        to_u16_max,
        to_u16,
        i32,
        u16,
        u16::MAX as i32,
        Some(u16::MAX)
    );

    test_to!(to_u32_ok, to_u32, i32, u32, 42, Some(42));
    test_to!(to_u32_fail_neg, to_u32, i32, u32, -1, None);
    test_to!(to_u32_min, to_u32, i32, u32, 0, Some(0));
    test_to!(
        to_u32_max,
        to_u32,
        i32,
        u32,
        i32::MAX,
        Some(i32::MAX as u32)
    );

    test_to!(to_u64_ok, to_u64, i32, u64, 42, Some(42));
    test_to!(to_u64_fail_neg, to_u64, i32, u64, -1, None);
    test_to!(to_u64_min, to_u64, i32, u64, 0, Some(0));
    test_to!(
        to_u64_max,
        to_u64,
        i32,
        u64,
        i32::MAX,
        Some(i32::MAX as u64)
    );

    test_to!(to_u128_ok, to_u128, i32, u128, 42, Some(42));
    test_to!(to_u128_fail_neg, to_u128, i32, u128, -1, None);
    test_to!(to_u128_min, to_u128, i32, u128, 0, Some(0));
    test_to!(
        to_u128_max,
        to_u128,
        i32,
        u128,
        i32::MAX,
        Some(i32::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, i32, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, i32, f64, 42, Some(42.0f64));
}

mod i64_to_tests {
    use super::*;

    test_to!(
        to_isize_min,
        to_isize,
        i64,
        isize,
        i64::MIN,
        Some(i64::MIN as isize)
    );
    test_to!(
        to_isize_max,
        to_isize,
        i64,
        isize,
        i64::MAX,
        Some(i64::MAX as isize)
    );

    test_to!(to_i8_ok, to_i8, i64, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, i64, i8, i8::MAX as i64 + 1, None);
    test_to!(to_i8_fail_neg, to_i8, i64, i8, i8::MIN as i64 - 1, None);
    test_to!(to_i8_min, to_i8, i64, i8, i64::MIN, None);
    test_to!(to_i8_max, to_i8, i64, i8, i64::MAX, None);

    test_to!(to_i16_ok, to_i16, i64, i16, 42, Some(42));
    test_to!(to_i16_fail_pos, to_i16, i64, i16, i16::MAX as i64 + 1, None);
    test_to!(to_i16_fail_neg, to_i16, i64, i16, i16::MIN as i64 - 1, None);
    test_to!(to_i16_min, to_i16, i64, i16, i64::MIN, None);
    test_to!(to_i16_max, to_i16, i64, i16, i64::MAX, None);

    test_to!(to_i32_ok, to_i32, i64, i32, 42, Some(42));
    test_to!(to_i32_fail_pos, to_i32, i64, i32, i32::MAX as i64 + 1, None);
    test_to!(to_i32_fail_neg, to_i32, i64, i32, i32::MIN as i64 - 1, None);
    test_to!(to_i32_min, to_i32, i64, i32, i64::MIN, None);
    test_to!(to_i32_max, to_i32, i64, i32, i64::MAX, None);

    test_to!(to_i64_ok, to_i64, i64, i64, 42, Some(42));
    test_to!(to_i64_min, to_i64, i64, i64, i64::MIN, Some(i64::MIN));
    test_to!(to_i64_max, to_i64, i64, i64, i64::MAX, Some(i64::MAX));

    test_to!(to_i128_ok, to_i128, i64, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        i64,
        i128,
        i64::MIN,
        Some(i64::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        i64,
        i128,
        i64::MAX,
        Some(i64::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, i64, usize, 42, Some(42));
    test_to!(to_usize_fail_neg, to_usize, i64, usize, -1, None);
    test_to!(to_usize_min, to_usize, i64, usize, 0, Some(0));
    test_to!(
        to_usize_max,
        to_usize,
        i64,
        usize,
        i64::MAX,
        Some(i64::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, i64, u8, 42, Some(42));
    test_to!(to_u8_fail_neg, to_u8, i64, u8, -1, None);
    test_to!(to_u8_fail_pos, to_u8, i64, u8, u8::MAX as i64 + 1, None);
    test_to!(to_u8_min, to_u8, i64, u8, 0, Some(0));
    test_to!(to_u8_max, to_u8, i64, u8, u8::MAX as i64, Some(u8::MAX));

    test_to!(to_u16_ok, to_u16, i64, u16, 42, Some(42));
    test_to!(to_u16_fail_neg, to_u16, i64, u16, -1, None);
    test_to!(to_u16_fail_pos, to_u16, i64, u16, u16::MAX as i64 + 1, None);
    test_to!(to_u16_min, to_u16, i64, u16, 0, Some(0));
    test_to!(
        to_u16_max,
        to_u16,
        i64,
        u16,
        u16::MAX as i64,
        Some(u16::MAX)
    );

    test_to!(to_u32_ok, to_u32, i64, u32, 42, Some(42));
    test_to!(to_u32_fail_neg, to_u32, i64, u32, -1, None);
    test_to!(to_u32_fail_pos, to_u32, i64, u32, u32::MAX as i64 + 1, None);
    test_to!(to_u32_min, to_u32, i64, u32, 0, Some(0));
    test_to!(
        to_u32_max,
        to_u32,
        i64,
        u32,
        u32::MAX as i64,
        Some(u32::MAX)
    );

    test_to!(to_u64_ok, to_u64, i64, u64, 42, Some(42));
    test_to!(to_u64_fail_neg, to_u64, i64, u64, -1, None);
    test_to!(to_u64_min, to_u64, i64, u64, 0, Some(0));
    test_to!(
        to_u64_max,
        to_u64,
        i64,
        u64,
        i64::MAX,
        Some(i64::MAX as u64)
    );

    test_to!(to_u128_ok, to_u128, i64, u128, 42, Some(42));
    test_to!(to_u128_fail_neg, to_u128, i64, u128, -1, None);
    test_to!(to_u128_min, to_u128, i64, u128, 0, Some(0));
    test_to!(
        to_u128_max,
        to_u128,
        i64,
        u128,
        i64::MAX,
        Some(i64::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, i64, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, i64, f64, 42, Some(42.0f64));
}

mod i128_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, i128, isize, 42, Some(42));
    test_to!(
        to_isize_fail_pos,
        to_isize,
        i128,
        isize,
        isize::MAX as i128 + 1,
        None
    );
    test_to!(
        to_isize_fail_neg,
        to_isize,
        i128,
        isize,
        isize::MIN as i128 - 1,
        None
    );
    test_to!(to_isize_min, to_isize, i128, isize, i128::MIN, None);
    test_to!(to_isize_max, to_isize, i128, isize, i128::MAX, None);

    test_to!(to_i8_ok, to_i8, i128, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, i128, i8, i8::MAX as i128 + 1, None);
    test_to!(to_i8_fail_neg, to_i8, i128, i8, i8::MIN as i128 - 1, None);
    test_to!(to_i8_min, to_i8, i128, i8, i128::MIN, None);
    test_to!(to_i8_max, to_i8, i128, i8, i128::MAX, None);

    test_to!(to_i16_ok, to_i16, i128, i16, 42, Some(42));
    test_to!(
        to_i16_fail_pos,
        to_i16,
        i128,
        i16,
        i16::MAX as i128 + 1,
        None
    );
    test_to!(
        to_i16_fail_neg,
        to_i16,
        i128,
        i16,
        i16::MIN as i128 - 1,
        None
    );
    test_to!(to_i16_min, to_i16, i128, i16, i128::MIN, None);
    test_to!(to_i16_max, to_i16, i128, i16, i128::MAX, None);

    test_to!(to_i32_ok, to_i32, i128, i32, 42, Some(42));
    test_to!(
        to_i32_fail_pos,
        to_i32,
        i128,
        i32,
        i32::MAX as i128 + 1,
        None
    );
    test_to!(
        to_i32_fail_neg,
        to_i32,
        i128,
        i32,
        i32::MIN as i128 - 1,
        None
    );
    test_to!(to_i32_min, to_i32, i128, i32, i128::MIN, None);
    test_to!(to_i32_max, to_i32, i128, i32, i128::MAX, None);

    test_to!(to_i64_ok, to_i64, i128, i64, 42, Some(42));
    test_to!(
        to_i64_fail_pos,
        to_i64,
        i128,
        i64,
        i64::MAX as i128 + 1,
        None
    );
    test_to!(
        to_i64_fail_neg,
        to_i64,
        i128,
        i64,
        i64::MIN as i128 - 1,
        None
    );
    test_to!(to_i64_min, to_i64, i128, i64, i128::MIN, None);
    test_to!(to_i64_max, to_i64, i128, i64, i128::MAX, None);

    test_to!(to_i128_ok, to_i128, i128, i128, 42, Some(42));
    test_to!(to_i128_min, to_i128, i128, i128, i128::MIN, Some(i128::MIN));
    test_to!(to_i128_max, to_i128, i128, i128, i128::MAX, Some(i128::MAX));

    test_to!(to_usize_ok, to_usize, i128, usize, 42, Some(42));
    test_to!(to_usize_fail_neg, to_usize, i128, usize, -1, None);
    test_to!(
        to_usize_fail_pos,
        to_usize,
        i128,
        usize,
        usize::MAX as i128 + 1,
        None
    );
    test_to!(to_usize_min, to_usize, i128, usize, 0, Some(0));
    test_to!(to_usize_max, to_usize, i128, usize, i128::MAX, None);

    test_to!(to_u8_ok, to_u8, i128, u8, 42, Some(42));
    test_to!(to_u8_fail_neg, to_u8, i128, u8, -1, None);
    test_to!(to_u8_fail_pos, to_u8, i128, u8, u8::MAX as i128 + 1, None);
    test_to!(to_u8_min, to_u8, i128, u8, 0, Some(0));
    test_to!(to_u8_max, to_u8, i128, u8, i128::MAX, None);

    test_to!(to_u16_ok, to_u16, i128, u16, 42, Some(42));
    test_to!(to_u16_fail_neg, to_u16, i128, u16, -1, None);
    test_to!(
        to_u16_fail_pos,
        to_u16,
        i128,
        u16,
        u16::MAX as i128 + 1,
        None
    );
    test_to!(to_u16_min, to_u16, i128, u16, 0, Some(0));
    test_to!(to_u16_max, to_u16, i128, u16, i128::MAX, None);

    test_to!(to_u32_ok, to_u32, i128, u32, 42, Some(42));
    test_to!(to_u32_fail_neg, to_u32, i128, u32, -1, None);
    test_to!(
        to_u32_fail_pos,
        to_u32,
        i128,
        u32,
        u32::MAX as i128 + 1,
        None
    );
    test_to!(to_u32_min, to_u32, i128, u32, 0, Some(0));
    test_to!(to_u32_max, to_u32, i128, u32, i128::MAX, None);

    test_to!(to_u64_ok, to_u64, i128, u64, 42, Some(42));
    test_to!(to_u64_fail_neg, to_u64, i128, u64, -1, None);
    test_to!(
        to_u64_fail_pos,
        to_u64,
        i128,
        u64,
        u64::MAX as i128 + 1,
        None
    );
    test_to!(to_u64_min, to_u64, i128, u64, 0, Some(0));
    test_to!(to_u64_max, to_u64, i128, u64, i128::MAX, None);

    test_to!(to_u128_ok, to_u128, i128, u128, 42, Some(42));
    test_to!(to_u128_fail_neg, to_u128, i128, u128, -1, None);
    test_to!(to_u128_min, to_u128, i128, u128, 0, Some(0));
    test_to!(
        to_u128_max,
        to_u128,
        i128,
        u128,
        i128::MAX,
        Some(i128::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, i128, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, i128, f64, 42, Some(42.0f64));
}
