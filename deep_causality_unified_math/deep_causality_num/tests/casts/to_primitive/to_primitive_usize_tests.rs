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

mod usize_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, usize, isize, 42, Some(42));
    test_to!(
        to_isize_fail_pos,
        to_isize,
        usize,
        isize,
        isize::MAX as usize + 1,
        None
    );
    test_to!(to_isize_min, to_isize, usize, isize, 0, Some(0));
    test_to!(to_isize_max, to_isize, usize, isize, usize::MAX, None);

    test_to!(to_i8_ok, to_i8, usize, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, usize, i8, i8::MAX as usize + 1, None);
    test_to!(to_i8_min, to_i8, usize, i8, 0, Some(0));
    test_to!(to_i8_max, to_i8, usize, i8, usize::MAX, None);

    test_to!(to_i16_ok, to_i16, usize, i16, 42, Some(42));
    test_to!(
        to_i16_fail_pos,
        to_i16,
        usize,
        i16,
        i16::MAX as usize + 1,
        None
    );
    test_to!(to_i16_min, to_i16, usize, i16, 0, Some(0));
    test_to!(to_i16_max, to_i16, usize, i16, usize::MAX, None);

    test_to!(to_i32_ok, to_i32, usize, i32, 42, Some(42));
    test_to!(
        to_i32_fail_pos,
        to_i32,
        usize,
        i32,
        i32::MAX as usize + 1,
        None
    );
    test_to!(to_i32_min, to_i32, usize, i32, 0, Some(0));
    test_to!(to_i32_max, to_i32, usize, i32, usize::MAX, None);

    test_to!(to_i64_ok, to_i64, usize, i64, 42, Some(42));
    test_to!(
        to_i64_fail_pos,
        to_i64,
        usize,
        i64,
        i64::MAX as usize + 1,
        None
    );
    test_to!(to_i64_min, to_i64, usize, i64, 0, Some(0));
    test_to!(to_i64_max, to_i64, usize, i64, usize::MAX, None);

    test_to!(to_i128_min, to_i128, usize, i128, 0, Some(0));
    test_to!(
        to_i128_max,
        to_i128,
        usize,
        i128,
        usize::MAX,
        Some(usize::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, usize, usize, 42, Some(42));
    test_to!(
        to_usize_min,
        to_usize,
        usize,
        usize,
        usize::MIN,
        Some(usize::MIN)
    );
    test_to!(
        to_usize_max,
        to_usize,
        usize,
        usize,
        usize::MAX,
        Some(usize::MAX)
    );

    test_to!(to_u8_ok, to_u8, usize, u8, 42, Some(42));
    test_to!(to_u8_fail_pos, to_u8, usize, u8, u8::MAX as usize + 1, None);
    test_to!(to_u8_min, to_u8, usize, u8, 0, Some(0));
    test_to!(to_u8_max, to_u8, usize, u8, usize::MAX, None);

    test_to!(to_u16_ok, to_u16, usize, u16, 42, Some(42));
    test_to!(
        to_u16_fail_pos,
        to_u16,
        usize,
        u16,
        u16::MAX as usize + 1,
        None
    );
    test_to!(to_u16_min, to_u16, usize, u16, 0, Some(0));
    test_to!(to_u16_max, to_u16, usize, u16, usize::MAX, None);

    test_to!(to_u32_ok, to_u32, usize, u32, 42, Some(42));
    test_to!(
        to_u32_fail_pos,
        to_u32,
        usize,
        u32,
        u32::MAX as usize + 1,
        None
    );
    test_to!(to_u32_min, to_u32, usize, u32, 0, Some(0));
    test_to!(to_u32_max, to_u32, usize, u32, usize::MAX, None);

    test_to!(to_u64_min, to_u64, usize, u64, 0, Some(0));
    test_to!(
        to_u64_max,
        to_u64,
        usize,
        u64,
        usize::MAX,
        Some(usize::MAX as u64)
    );

    test_to!(to_u128_min, to_u128, usize, u128, 0, Some(0));
    test_to!(
        to_u128_max,
        to_u128,
        usize,
        u128,
        usize::MAX,
        Some(usize::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, usize, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, usize, f64, 42, Some(42.0f64));
}

mod u8_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, u8, isize, 42, Some(42));
    test_to!(
        to_isize_min,
        to_isize,
        u8,
        isize,
        u8::MIN,
        Some(u8::MIN as isize)
    );
    test_to!(
        to_isize_max,
        to_isize,
        u8,
        isize,
        u8::MAX,
        Some(u8::MAX as isize)
    );

    test_to!(to_i8_ok, to_i8, u8, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, u8, i8, i8::MAX as u8 + 1, None);
    test_to!(to_i8_min, to_i8, u8, i8, 0, Some(0));
    test_to!(to_i8_max, to_i8, u8, i8, u8::MAX, None);

    test_to!(to_i16_ok, to_i16, u8, i16, 42, Some(42));
    test_to!(to_i16_min, to_i16, u8, i16, u8::MIN, Some(u8::MIN as i16));
    test_to!(to_i16_max, to_i16, u8, i16, u8::MAX, Some(u8::MAX as i16));

    test_to!(to_i32_ok, to_i32, u8, i32, 42, Some(42));
    test_to!(to_i32_min, to_i32, u8, i32, u8::MIN, Some(u8::MIN as i32));
    test_to!(to_i32_max, to_i32, u8, i32, u8::MAX, Some(u8::MAX as i32));

    test_to!(to_i64_ok, to_i64, u8, i64, 42, Some(42));
    test_to!(to_i64_min, to_i64, u8, i64, u8::MIN, Some(u8::MIN as i64));
    test_to!(to_i64_max, to_i64, u8, i64, u8::MAX, Some(u8::MAX as i64));

    test_to!(to_i128_ok, to_i128, u8, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        u8,
        i128,
        u8::MIN,
        Some(u8::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        u8,
        i128,
        u8::MAX,
        Some(u8::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, u8, usize, 42, Some(42));
    test_to!(
        to_usize_min,
        to_usize,
        u8,
        usize,
        u8::MIN,
        Some(u8::MIN as usize)
    );
    test_to!(
        to_usize_max,
        to_usize,
        u8,
        usize,
        u8::MAX,
        Some(u8::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, u8, u8, 42, Some(42));
    test_to!(to_u8_min, to_u8, u8, u8, u8::MIN, Some(u8::MIN));
    test_to!(to_u8_max, to_u8, u8, u8, u8::MAX, Some(u8::MAX));

    test_to!(to_u16_ok, to_u16, u8, u16, 42, Some(42));
    test_to!(to_u16_min, to_u16, u8, u16, u8::MIN, Some(u8::MIN as u16));
    test_to!(to_u16_max, to_u16, u8, u16, u8::MAX, Some(u8::MAX as u16));

    test_to!(to_u32_ok, to_u32, u8, u32, 42, Some(42));
    test_to!(to_u32_min, to_u32, u8, u32, u8::MIN, Some(u8::MIN as u32));
    test_to!(to_u32_max, to_u32, u8, u32, u8::MAX, Some(u8::MAX as u32));

    test_to!(to_u64_ok, to_u64, u8, u64, 42, Some(42));
    test_to!(to_u64_min, to_u64, u8, u64, u8::MIN, Some(u8::MIN as u64));
    test_to!(to_u64_max, to_u64, u8, u64, u8::MAX, Some(u8::MAX as u64));

    test_to!(to_u128_ok, to_u128, u8, u128, 42, Some(42));
    test_to!(
        to_u128_min,
        to_u128,
        u8,
        u128,
        u8::MIN,
        Some(u8::MIN as u128)
    );
    test_to!(
        to_u128_max,
        to_u128,
        u8,
        u128,
        u8::MAX,
        Some(u8::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, u8, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, u8, f64, 42, Some(42.0f64));
}

mod u16_to_tests {
    use super::*;

    test_to!(
        to_isize_min,
        to_isize,
        u16,
        isize,
        u16::MIN,
        Some(u16::MIN as isize)
    );
    test_to!(
        to_isize_max,
        to_isize,
        u16,
        isize,
        u16::MAX,
        Some(u16::MAX as isize)
    );

    test_to!(to_i8_ok, to_i8, u16, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, u16, i8, i8::MAX as u16 + 1, None);
    test_to!(to_i8_min, to_i8, u16, i8, u16::MIN, Some(u16::MIN as i8));
    test_to!(to_i8_max, to_i8, u16, i8, u16::MAX, None);

    test_to!(to_i16_ok, to_i16, u16, i16, 42, Some(42));
    test_to!(to_i16_fail_pos, to_i16, u16, i16, i16::MAX as u16 + 1, None);
    test_to!(
        to_i16_min,
        to_i16,
        u16,
        i16,
        u16::MIN,
        Some(u16::MIN as i16)
    );
    test_to!(to_i16_max, to_i16, u16, i16, u16::MAX, None);

    test_to!(to_i32_ok, to_i32, u16, i32, 42, Some(42));
    test_to!(
        to_i32_min,
        to_i32,
        u16,
        i32,
        u16::MIN,
        Some(u16::MIN as i32)
    );
    test_to!(
        to_i32_max,
        to_i32,
        u16,
        i32,
        u16::MAX,
        Some(u16::MAX as i32)
    );

    test_to!(to_i64_ok, to_i64, u16, i64, 42, Some(42));
    test_to!(
        to_i64_min,
        to_i64,
        u16,
        i64,
        u16::MIN,
        Some(u16::MIN as i64)
    );
    test_to!(
        to_i64_max,
        to_i64,
        u16,
        i64,
        u16::MAX,
        Some(u16::MAX as i64)
    );

    test_to!(to_i128_ok, to_i128, u16, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        u16,
        i128,
        u16::MIN,
        Some(u16::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        u16,
        i128,
        u16::MAX,
        Some(u16::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, u16, usize, 42, Some(42));
    test_to!(
        to_usize_min,
        to_usize,
        u16,
        usize,
        u16::MIN,
        Some(u16::MIN as usize)
    );
    test_to!(
        to_usize_max,
        to_usize,
        u16,
        usize,
        u16::MAX,
        Some(u16::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, u16, u8, 42, Some(42));
    test_to!(to_u8_fail_pos, to_u8, u16, u8, u8::MAX as u16 + 1, None);
    test_to!(to_u8_min, to_u8, u16, u8, u16::MIN, Some(u16::MIN as u8));
    test_to!(to_u8_max, to_u8, u16, u8, u16::MAX, None);

    test_to!(to_u16_ok, to_u16, u16, u16, 42, Some(42));
    test_to!(to_u16_min, to_u16, u16, u16, u16::MIN, Some(u16::MIN));
    test_to!(to_u16_max, to_u16, u16, u16, u16::MAX, Some(u16::MAX));

    test_to!(to_u32_ok, to_u32, u16, u32, 42, Some(42));
    test_to!(
        to_u32_min,
        to_u32,
        u16,
        u32,
        u16::MIN,
        Some(u16::MIN as u32)
    );
    test_to!(
        to_u32_max,
        to_u32,
        u16,
        u32,
        u16::MAX,
        Some(u16::MAX as u32)
    );

    test_to!(to_u64_ok, to_u64, u16, u64, 42, Some(42));
    test_to!(
        to_u64_min,
        to_u64,
        u16,
        u64,
        u16::MIN,
        Some(u16::MIN as u64)
    );
    test_to!(
        to_u64_max,
        to_u64,
        u16,
        u64,
        u16::MAX,
        Some(u16::MAX as u64)
    );

    test_to!(to_u128_ok, to_u128, u16, u128, 42, Some(42));
    test_to!(
        to_u128_min,
        to_u128,
        u16,
        u128,
        u16::MIN,
        Some(u16::MIN as u128)
    );
    test_to!(
        to_u128_max,
        to_u128,
        u16,
        u128,
        u16::MAX,
        Some(u16::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, u16, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, u16, f64, 42, Some(42.0f64));
}

mod u32_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, u32, isize, 42, Some(42));
    test_to!(
        to_isize_min,
        to_isize,
        u32,
        isize,
        u32::MIN,
        Some(u32::MIN as isize)
    );

    #[cfg(target_pointer_width = "64")]
    test_to!(
        to_isize_max_ok,
        to_isize,
        u32,
        isize,
        u32::MAX,
        Some(u32::MAX as isize)
    );

    #[cfg(target_pointer_width = "32")]
    test_to!(
        to_isize_max_ok,
        to_isize,
        u32,
        isize,
        isize::MAX as u32,
        Some(isize::MAX)
    );

    #[cfg(target_pointer_width = "32")]
    test_to!(to_isize_fail_u32_max, to_isize, u32, isize, u32::MAX, None);

    test_to!(to_i8_ok, to_i8, u32, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, u32, i8, i8::MAX as u32 + 1, None);
    test_to!(to_i8_min, to_i8, u32, i8, u32::MIN, Some(u32::MIN as i8));
    test_to!(to_i8_max, to_i8, u32, i8, u32::MAX, None);

    test_to!(to_i16_ok, to_i16, u32, i16, 42, Some(42));
    test_to!(to_i16_fail_pos, to_i16, u32, i16, i16::MAX as u32 + 1, None);
    test_to!(
        to_i16_min,
        to_i16,
        u32,
        i16,
        u32::MIN,
        Some(u32::MIN as i16)
    );
    test_to!(to_i16_max, to_i16, u32, i16, u32::MAX, None);

    test_to!(to_i32_ok, to_i32, u32, i32, 42, Some(42));
    test_to!(to_i32_fail_pos, to_i32, u32, i32, i32::MAX as u32 + 1, None);
    test_to!(
        to_i32_min,
        to_i32,
        u32,
        i32,
        u32::MIN,
        Some(u32::MIN as i32)
    );
    test_to!(to_i32_max, to_i32, u32, i32, u32::MAX, None);

    test_to!(to_i64_ok, to_i64, u32, i64, 42, Some(42));
    test_to!(
        to_i64_min,
        to_i64,
        u32,
        i64,
        u32::MIN,
        Some(u32::MIN as i64)
    );
    test_to!(
        to_i64_max,
        to_i64,
        u32,
        i64,
        u32::MAX,
        Some(u32::MAX as i64)
    );

    test_to!(to_i128_ok, to_i128, u32, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        u32,
        i128,
        u32::MIN,
        Some(u32::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        u32,
        i128,
        u32::MAX,
        Some(u32::MAX as i128)
    );

    test_to!(to_usize_ok, to_usize, u32, usize, 42, Some(42));
    test_to!(
        to_usize_min,
        to_usize,
        u32,
        usize,
        u32::MIN,
        Some(u32::MIN as usize)
    );
    test_to!(
        to_usize_max,
        to_usize,
        u32,
        usize,
        u32::MAX,
        Some(u32::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, u32, u8, 42, Some(42));
    test_to!(to_u8_fail_pos, to_u8, u32, u8, u8::MAX as u32 + 1, None);
    test_to!(to_u8_min, to_u8, u32, u8, u32::MIN, Some(u32::MIN as u8));
    test_to!(to_u8_max, to_u8, u32, u8, u32::MAX, None);

    test_to!(to_u16_ok, to_u16, u32, u16, 42, Some(42));
    test_to!(to_u16_fail_pos, to_u16, u32, u16, u16::MAX as u32 + 1, None);
    test_to!(
        to_u16_min,
        to_u16,
        u32,
        u16,
        u32::MIN,
        Some(u32::MIN as u16)
    );
    test_to!(to_u16_max, to_u16, u32, u16, u32::MAX, None);

    test_to!(to_u32_ok, to_u32, u32, u32, 42, Some(42));
    test_to!(to_u32_min, to_u32, u32, u32, u32::MIN, Some(u32::MIN));
    test_to!(to_u32_max, to_u32, u32, u32, u32::MAX, Some(u32::MAX));

    test_to!(to_u64_ok, to_u64, u32, u64, 42, Some(42));
    test_to!(
        to_u64_min,
        to_u64,
        u32,
        u64,
        u32::MIN,
        Some(u32::MIN as u64)
    );
    test_to!(
        to_u64_max,
        to_u64,
        u32,
        u64,
        u32::MAX,
        Some(u32::MAX as u64)
    );

    test_to!(to_u128_ok, to_u128, u32, u128, 42, Some(42));
    test_to!(
        to_u128_min,
        to_u128,
        u32,
        u128,
        u32::MIN,
        Some(u32::MIN as u128)
    );
    test_to!(
        to_u128_max,
        to_u128,
        u32,
        u128,
        u32::MAX,
        Some(u32::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, u32, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, u32, f64, 42, Some(42.0f64));
}

mod u64_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, u64, isize, 42, Some(42));
    test_to!(
        to_isize_fail_pos,
        to_isize,
        u64,
        isize,
        isize::MAX as u64 + 1,
        None
    );
    test_to!(
        to_isize_min,
        to_isize,
        u64,
        isize,
        u64::MIN,
        Some(u64::MIN as isize)
    );
    test_to!(to_isize_max, to_isize, u64, isize, u64::MAX, None);

    test_to!(to_i8_ok, to_i8, u64, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, u64, i8, i8::MAX as u64 + 1, None);
    test_to!(to_i8_min, to_i8, u64, i8, u64::MIN, Some(u64::MIN as i8));
    test_to!(to_i8_max, to_i8, u64, i8, u64::MAX, None);

    test_to!(to_i16_ok, to_i16, u64, i16, 42, Some(42));
    test_to!(to_i16_fail_pos, to_i16, u64, i16, i16::MAX as u64 + 1, None);
    test_to!(
        to_i16_min,
        to_i16,
        u64,
        i16,
        u64::MIN,
        Some(u64::MIN as i16)
    );
    test_to!(to_i16_max, to_i16, u64, i16, u64::MAX, None);

    test_to!(to_i32_ok, to_i32, u64, i32, 42, Some(42));
    test_to!(to_i32_fail_pos, to_i32, u64, i32, i32::MAX as u64 + 1, None);
    test_to!(
        to_i32_min,
        to_i32,
        u64,
        i32,
        u64::MIN,
        Some(u64::MIN as i32)
    );
    test_to!(to_i32_max, to_i32, u64, i32, u64::MAX, None);

    test_to!(to_i64_ok, to_i64, u64, i64, 42, Some(42));
    test_to!(to_i64_fail_pos, to_i64, u64, i64, i64::MAX as u64 + 1, None);
    test_to!(
        to_i64_min,
        to_i64,
        u64,
        i64,
        u64::MIN,
        Some(u64::MIN as i64)
    );
    test_to!(to_i64_max, to_i64, u64, i64, u64::MAX, None);

    test_to!(to_i128_ok, to_i128, u64, i128, 42, Some(42));
    test_to!(
        to_i128_min,
        to_i128,
        u64,
        i128,
        u64::MIN,
        Some(u64::MIN as i128)
    );
    test_to!(
        to_i128_max,
        to_i128,
        u64,
        i128,
        u64::MAX,
        Some(u64::MAX as i128)
    );

    test_to!(
        to_usize_min,
        to_usize,
        u64,
        usize,
        u64::MIN,
        Some(u64::MIN as usize)
    );
    test_to!(
        to_usize_max,
        to_usize,
        u64,
        usize,
        u64::MAX,
        Some(u64::MAX as usize)
    );

    test_to!(to_u8_ok, to_u8, u64, u8, 42, Some(42));
    test_to!(to_u8_fail_pos, to_u8, u64, u8, u8::MAX as u64 + 1, None);
    test_to!(to_u8_min, to_u8, u64, u8, u64::MIN, Some(u64::MIN as u8));
    test_to!(to_u8_max, to_u8, u64, u8, u64::MAX, None);

    test_to!(to_u16_ok, to_u16, u64, u16, 42, Some(42));
    test_to!(to_u16_fail_pos, to_u16, u64, u16, u16::MAX as u64 + 1, None);
    test_to!(
        to_u16_min,
        to_u16,
        u64,
        u16,
        u64::MIN,
        Some(u64::MIN as u16)
    );
    test_to!(to_u16_max, to_u16, u64, u16, u64::MAX, None);

    test_to!(to_u32_ok, to_u32, u64, u32, 42, Some(42));
    test_to!(to_u32_fail_pos, to_u32, u64, u32, u32::MAX as u64 + 1, None);
    test_to!(
        to_u32_min,
        to_u32,
        u64,
        u32,
        u64::MIN,
        Some(u64::MIN as u32)
    );
    test_to!(to_u32_max, to_u32, u64, u32, u64::MAX, None);

    test_to!(to_u64_ok, to_u64, u64, u64, 42, Some(42u64));
    test_to!(to_u64_min, to_u64, u64, u64, u64::MIN, Some(u64::MIN));
    test_to!(to_u64_max, to_u64, u64, u64, u64::MAX, Some(u64::MAX));

    test_to!(to_u128_ok, to_u128, u64, u128, 42, Some(42));
    test_to!(
        to_u128_min,
        to_u128,
        u64,
        u128,
        u64::MIN,
        Some(u64::MIN as u128)
    );
    test_to!(
        to_u128_max,
        to_u128,
        u64,
        u128,
        u64::MAX,
        Some(u64::MAX as u128)
    );

    test_to!(to_f32_ok, to_f32, u64, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, u64, f64, 42, Some(42.0f64));
}

mod u128_to_tests {
    use super::*;
    test_to!(to_isize_ok, to_isize, u128, isize, 42, Some(42));
    test_to!(
        to_isize_fail_pos,
        to_isize,
        u128,
        isize,
        isize::MAX as u128 + 1,
        None
    );
    test_to!(
        to_isize_min,
        to_isize,
        u128,
        isize,
        u128::MIN,
        Some(u128::MIN as isize)
    );
    test_to!(to_isize_max, to_isize, u128, isize, u128::MAX, None);

    test_to!(to_i8_ok, to_i8, u128, i8, 42, Some(42));
    test_to!(to_i8_fail_pos, to_i8, u128, i8, i8::MAX as u128 + 1, None);
    test_to!(to_i8_min, to_i8, u128, i8, u128::MIN, Some(u128::MIN as i8));
    test_to!(to_i8_max, to_i8, u128, i8, u128::MAX, None);

    test_to!(to_i16_ok, to_i16, u128, i16, 42, Some(42));
    test_to!(
        to_i16_fail_pos,
        to_i16,
        u128,
        i16,
        i16::MAX as u128 + 1,
        None
    );
    test_to!(
        to_i16_min,
        to_i16,
        u128,
        i16,
        u128::MIN,
        Some(u128::MIN as i16)
    );
    test_to!(to_i16_max, to_i16, u128, i16, u16::MAX as u128 + 1, None);

    test_to!(to_i32_ok, to_i32, u128, i32, 42, Some(42));
    test_to!(
        to_i32_fail_pos,
        to_i32,
        u128,
        i32,
        i32::MAX as u128 + 1,
        None
    );
    test_to!(
        to_i32_min,
        to_i32,
        u128,
        i32,
        u128::MIN,
        Some(u128::MIN as i32)
    );
    test_to!(to_i32_max, to_i32, u128, i32, u128::MAX, None);

    test_to!(to_i64_ok, to_i64, u128, i64, 42, Some(42));
    test_to!(
        to_i64_fail_pos,
        to_i64,
        u128,
        i64,
        i64::MAX as u128 + 1,
        None
    );
    test_to!(
        to_i64_min,
        to_i64,
        u128,
        i64,
        u128::MIN,
        Some(u128::MIN as i64)
    );
    test_to!(to_i64_max, to_i64, u128, i64, u128::MAX, None);

    test_to!(to_i128_ok, to_i128, u128, i128, 42, Some(42));
    test_to!(
        to_i128_fail_pos,
        to_i128,
        u128,
        i128,
        i128::MAX as u128 + 1,
        None
    );
    test_to!(
        to_i128_min,
        to_i128,
        u128,
        i128,
        u128::MIN,
        Some(u128::MIN as i128)
    );
    test_to!(to_i128_max, to_i128, u128, i128, u128::MAX, None);

    test_to!(to_usize_ok, to_usize, u128, usize, 42, Some(42));
    test_to!(
        to_usize_fail_pos,
        to_usize,
        u128,
        usize,
        usize::MAX as u128 + 1,
        None
    );
    test_to!(
        to_usize_min,
        to_usize,
        u128,
        usize,
        u128::MIN,
        Some(u128::MIN as usize)
    );
    test_to!(to_usize_max, to_usize, u128, usize, u128::MAX, None);

    test_to!(to_u8_ok, to_u8, u128, u8, 42, Some(42));
    test_to!(to_u8_fail_pos, to_u8, u128, u8, u8::MAX as u128 + 1, None);
    test_to!(to_u8_min, to_u8, u128, u8, u128::MIN, Some(u128::MIN as u8));
    test_to!(to_u8_max, to_u8, u128, u8, u128::MAX, None);

    test_to!(to_u16_ok, to_u16, u128, u16, 42, Some(42));
    test_to!(
        to_u16_fail_pos,
        to_u16,
        u128,
        u16,
        u16::MAX as u128 + 1,
        None
    );
    test_to!(
        to_u16_min,
        to_u16,
        u128,
        u16,
        u128::MIN,
        Some(u128::MIN as u16)
    );
    test_to!(to_u16_max, to_u16, u128, u16, u128::MAX, None);

    test_to!(to_u32_ok, to_u32, u128, u32, 42, Some(42));
    test_to!(
        to_u32_fail_pos,
        to_u32,
        u128,
        u32,
        u32::MAX as u128 + 1,
        None
    );
    test_to!(
        to_u32_min,
        to_u32,
        u128,
        u32,
        u128::MIN,
        Some(u128::MIN as u32)
    );
    test_to!(to_u32_max, to_u32, u128, u32, u128::MAX, None);

    test_to!(to_u64_ok, to_u64, u128, u64, 42, Some(42));
    test_to!(
        to_u64_fail_pos,
        to_u64,
        u128,
        u64,
        u64::MAX as u128 + 1,
        None
    );
    test_to!(
        to_u64_min,
        to_u64,
        u128,
        u64,
        u128::MIN,
        Some(u128::MIN as u64)
    );
    test_to!(to_u64_max, to_u64, u128, u64, u128::MAX, None);

    test_to!(to_u128_ok, to_u128, u128, u128, 42, Some(42));
    test_to!(to_u128_min, to_u128, u128, u128, u128::MIN, Some(u128::MIN));
    test_to!(to_u128_max, to_u128, u128, u128, u128::MAX, Some(u128::MAX));

    test_to!(to_f32_ok, to_f32, u128, f32, 42, Some(42.0f32));
    test_to!(to_f64_ok, to_f64, u128, f64, 42, Some(42.0f64));
}
