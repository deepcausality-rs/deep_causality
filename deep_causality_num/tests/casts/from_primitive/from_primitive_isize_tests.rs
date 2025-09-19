/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::FromPrimitive;

macro_rules! test_from {
    ($name:ident, $from_method:ident, $from_ty:ty, $to_ty:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from_ty = $val;
            let expected: Option<$to_ty> = $expected;
            let actual = <$to_ty>::$from_method(v);
            assert_eq!(actual, expected);
        }
    };
}

//================================================================================
// To isize
//================================================================================
mod to_isize_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, isize, 42, Some(42));
    test_from!(from_i8_ok, from_i8, i8, isize, 42, Some(42));
    test_from!(from_i16_ok, from_i16, i16, isize, 42, Some(42));
    test_from!(from_i32_ok, from_i32, i32, isize, 42, Some(42));

    #[cfg(target_pointer_width = "32")]
    test_from!(from_i64_fail_over, from_i64, i64, isize, i64::MAX, None);
    #[cfg(target_pointer_width = "64")]
    test_from!(from_i64_ok, from_i64, i64, isize, 42, Some(42));

    test_from!(from_i128_fail_over, from_i128, i128, isize, i128::MAX, None);

    test_from!(from_usize_ok, from_usize, usize, isize, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_from!(
        from_usize_fail_over,
        from_usize,
        usize,
        isize,
        usize::MAX,
        None
    );

    test_from!(from_u8_ok, from_u8, u8, isize, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, isize, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, isize, 42, Some(42));

    #[cfg(target_pointer_width = "32")]
    test_from!(from_u64_fail_over, from_u64, u64, isize, u64::MAX, None);
    #[cfg(target_pointer_width = "64")]
    test_from!(from_u64_ok, from_u64, u64, isize, 42, Some(42));

    test_from!(from_u128_fail_over, from_u128, u128, isize, u128::MAX, None);

    test_from!(from_f32_ok, from_f32, f32, isize, 42.0, Some(42));
    test_from!(from_f64_ok, from_f64, f64, isize, 42.0, Some(42));
}

//================================================================================
// To i8
//================================================================================
mod to_i8_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, i8, 42, Some(42));
    test_from!(from_isize_fail_over, from_isize, isize, i8, 128, None);
    test_from!(from_isize_fail_under, from_isize, isize, i8, -129, None);

    test_from!(from_i8_ok, from_i8, i8, i8, 42, Some(42));

    test_from!(from_i16_ok, from_i16, i16, i8, 42, Some(42));
    test_from!(from_i16_fail_over, from_i16, i16, i8, 128, None);
    test_from!(from_i16_fail_under, from_i16, i16, i8, -129, None);

    test_from!(from_i32_ok, from_i32, i32, i8, 42, Some(42));
    test_from!(from_i32_fail_over, from_i32, i32, i8, 128, None);
    test_from!(from_i32_fail_under, from_i32, i32, i8, -129, None);

    test_from!(from_i64_ok, from_i64, i64, i8, 42, Some(42));
    test_from!(from_i64_fail_over, from_i64, i64, i8, 128, None);
    test_from!(from_i64_fail_under, from_i64, i64, i8, -129, None);

    test_from!(from_i128_ok, from_i128, i128, i8, 42, Some(42));
    test_from!(from_i128_fail_over, from_i128, i128, i8, 128, None);
    test_from!(from_i128_fail_under, from_i128, i128, i8, -129, None);

    test_from!(from_usize_ok, from_usize, usize, i8, 42, Some(42));
    test_from!(from_usize_fail_over, from_usize, usize, i8, 128, None);

    test_from!(from_u8_ok, from_u8, u8, i8, 42, Some(42));
    test_from!(from_u8_fail_over, from_u8, u8, i8, 128, None);

    test_from!(from_u16_ok, from_u16, u16, i8, 42, Some(42));
    test_from!(from_u16_fail_over, from_u16, u16, i8, 128, None);

    test_from!(from_u32_ok, from_u32, u32, i8, 42, Some(42));
    test_from!(from_u32_fail_over, from_u32, u32, i8, 128, None);

    test_from!(from_u64_ok, from_u64, u64, i8, 42, Some(42));
    test_from!(from_u64_fail_over, from_u64, u64, i8, 128, None);

    test_from!(from_u128_ok, from_u128, u128, i8, 42, Some(42));
    test_from!(from_u128_fail_over, from_u128, u128, i8, 128, None);

    test_from!(from_f32_ok, from_f32, f32, i8, 42.0, Some(42));
    test_from!(from_f32_fail_over, from_f32, f32, i8, 128.0, None);

    test_from!(from_f64_ok, from_f64, f64, i8, 42.0, Some(42));
    test_from!(from_f64_fail_over, from_f64, f64, i8, 128.0, None);
}

//================================================================================
// To i16
//================================================================================
mod to_i16_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, i16, 42, Some(42));
    test_from!(from_isize_fail_over, from_isize, isize, i16, 32768, None);
    test_from!(from_isize_fail_under, from_isize, isize, i16, -32769, None);

    test_from!(from_i8_ok, from_i8, i8, i16, 42, Some(42));
    test_from!(from_i16_ok, from_i16, i16, i16, 42, Some(42));

    test_from!(from_i32_ok, from_i32, i32, i16, 42, Some(42));
    test_from!(from_i32_fail_over, from_i32, i32, i16, 32768, None);
    test_from!(from_i32_fail_under, from_i32, i32, i16, -32769, None);

    test_from!(from_i64_ok, from_i64, i64, i16, 42, Some(42));
    test_from!(from_i64_fail_over, from_i64, i64, i16, 32768, None);
    test_from!(from_i64_fail_under, from_i64, i64, i16, -32769, None);

    test_from!(from_i128_ok, from_i128, i128, i16, 42, Some(42));
    test_from!(from_i128_fail_over, from_i128, i128, i16, 32768, None);
    test_from!(from_i128_fail_under, from_i128, i128, i16, -32769, None);

    test_from!(from_usize_ok, from_usize, usize, i16, 42, Some(42));
    test_from!(from_usize_fail_over, from_usize, usize, i16, 32768, None);

    test_from!(from_u8_ok, from_u8, u8, i16, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, i16, 42, Some(42));
    test_from!(from_u16_fail_over, from_u16, u16, i16, 32768, None);

    test_from!(from_u32_ok, from_u32, u32, i16, 42, Some(42));
    test_from!(from_u32_fail_over, from_u32, u32, i16, 32768, None);

    test_from!(from_u64_ok, from_u64, u64, i16, 42, Some(42));
    test_from!(from_u64_fail_over, from_u64, u64, i16, 32768, None);

    test_from!(from_u128_ok, from_u128, u128, i16, 42, Some(42));
    test_from!(from_u128_fail_over, from_u128, u128, i16, 32768, None);

    test_from!(from_f32_ok, from_f32, f32, i16, 42.0, Some(42));
    test_from!(from_f32_fail_over, from_f32, f32, i16, 32768.0, None);

    test_from!(from_f64_ok, from_f64, f64, i16, 42.0, Some(42));
    test_from!(from_f64_fail_over, from_f64, f64, i16, 32768.0, None);
}

//================================================================================
// To i32
//================================================================================
mod to_i32_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, i32, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_isize_fail_over,
        from_isize,
        isize,
        i32,
        i32::MAX as isize + 1,
        None
    );

    test_from!(from_i8_ok, from_i8, i8, i32, 42, Some(42));
    test_from!(from_i16_ok, from_i16, i16, i32, 42, Some(42));
    test_from!(from_i32_ok, from_i32, i32, i32, 42, Some(42));

    test_from!(from_i64_ok, from_i64, i64, i32, 42, Some(42));
    test_from!(
        from_i64_fail_over,
        from_i64,
        i64,
        i32,
        i32::MAX as i64 + 1,
        None
    );
    test_from!(
        from_i64_fail_under,
        from_i64,
        i64,
        i32,
        i32::MIN as i64 - 1,
        None
    );

    test_from!(from_i128_ok, from_i128, i128, i32, 42, Some(42));
    test_from!(
        from_i128_fail_over,
        from_i128,
        i128,
        i32,
        i32::MAX as i128 + 1,
        None
    );
    test_from!(
        from_i128_fail_under,
        from_i128,
        i128,
        i32,
        i32::MIN as i128 - 1,
        None
    );

    test_from!(from_usize_ok, from_usize, usize, i32, 42, Some(42));
    test_from!(
        from_usize_fail_over,
        from_usize,
        usize,
        i32,
        i32::MAX as usize + 1,
        None
    );

    test_from!(from_u8_ok, from_u8, u8, i32, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, i32, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, i32, 42, Some(42));
    test_from!(
        from_u32_fail_over,
        from_u32,
        u32,
        i32,
        i32::MAX as u32 + 1,
        None
    );

    test_from!(from_u64_ok, from_u64, u64, i32, 42, Some(42));
    test_from!(
        from_u64_fail_over,
        from_u64,
        u64,
        i32,
        i32::MAX as u64 + 1,
        None
    );

    test_from!(from_u128_ok, from_u128, u128, i32, 42, Some(42));
    test_from!(
        from_u128_fail_over,
        from_u128,
        u128,
        i32,
        i32::MAX as u128 + 1,
        None
    );

    test_from!(from_f32_ok, from_f32, f32, i32, 42.0, Some(42));
    test_from!(
        from_f32_fail_over,
        from_f32,
        f32,
        i32,
        i32::MAX as f32,
        None
    );

    test_from!(from_f64_ok, from_f64, f64, i32, 42.0, Some(42));
    test_from!(
        from_f64_fail_over,
        from_f64,
        f64,
        i32,
        i32::MAX as f64 + 1.0,
        None
    );
}

//================================================================================
// To i64
//================================================================================
mod to_i64_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, i64, 42, Some(42));

    test_from!(from_i8_ok, from_i8, i8, i64, 42, Some(42));
    test_from!(from_i16_ok, from_i16, i16, i64, 42, Some(42));
    test_from!(from_i32_ok, from_i32, i32, i64, 42, Some(42));
    test_from!(from_i64_ok, from_i64, i64, i64, 42, Some(42));

    test_from!(from_i128_ok, from_i128, i128, i64, 42, Some(42));
    test_from!(
        from_i128_fail_over,
        from_i128,
        i128,
        i64,
        i64::MAX as i128 + 1,
        None
    );
    test_from!(
        from_i128_fail_under,
        from_i128,
        i128,
        i64,
        i64::MIN as i128 - 1,
        None
    );

    test_from!(from_usize_ok, from_usize, usize, i64, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_usize_fail_over,
        from_usize,
        usize,
        i64,
        i64::MAX as usize + 1,
        None
    );

    test_from!(from_u8_ok, from_u8, u8, i64, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, i64, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, i64, 42, Some(42));
    test_from!(from_u64_ok, from_u64, u64, i64, 42, Some(42));
    test_from!(
        from_u64_fail_over,
        from_u64,
        u64,
        i64,
        i64::MAX as u64 + 1,
        None
    );

    test_from!(from_u128_ok, from_u128, u128, i64, 42, Some(42));
    test_from!(
        from_u128_fail_over,
        from_u128,
        u128,
        i64,
        i64::MAX as u128 + 1,
        None
    );

    test_from!(from_f64_ok, from_f64, f64, i64, 42.0, Some(42));
    test_from!(
        from_f64_fail_over,
        from_f64,
        f64,
        i64,
        i64::MAX as f64 * 2.0,
        None
    );
}

//================================================================================
// To i128
//================================================================================
mod to_i128_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, i128, 42, Some(42));
    test_from!(from_i8_ok, from_i8, i8, i128, 42, Some(42));
    test_from!(from_i16_ok, from_i16, i16, i128, 42, Some(42));
    test_from!(from_i32_ok, from_i32, i32, i128, 42, Some(42));
    test_from!(from_i64_ok, from_i64, i64, i128, 42, Some(42));
    test_from!(from_i128_ok, from_i128, i128, i128, 42, Some(42));

    test_from!(from_usize_ok, from_usize, usize, i128, 42, Some(42));

    test_from!(from_u8_ok, from_u8, u8, i128, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, i128, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, i128, 42, Some(42));
    test_from!(from_u64_ok, from_u64, u64, i128, 42, Some(42));
    test_from!(from_u128_ok, from_u128, u128, i128, 42, Some(42));
    test_from!(
        from_u128_fail_over,
        from_u128,
        u128,
        i128,
        i128::MAX as u128 + 1,
        None
    );

    test_from!(from_f32_ok, from_f32, f32, i128, 42.0, Some(42));
    test_from!(from_f32_fail_over, from_f32, f32, i128, f32::MAX, None);

    test_from!(from_f64_ok, from_f64, f64, i128, 42.0, Some(42));
    test_from!(from_f64_fail_over, from_f64, f64, i128, f64::MAX, None);
}
