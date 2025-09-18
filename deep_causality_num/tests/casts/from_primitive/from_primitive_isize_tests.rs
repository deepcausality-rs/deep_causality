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
// To i8
//================================================================================
mod to_i8_tests {
    use super::*;

    // from isize
    test_from!(from_isize_fit, from_isize, isize, i8, 42, Some(42));
    test_from!(
        from_isize_max,
        from_isize,
        isize,
        i8,
        i8::MAX as isize,
        Some(i8::MAX)
    );
    test_from!(
        from_isize_min,
        from_isize,
        isize,
        i8,
        i8::MIN as isize,
        Some(i8::MIN)
    );
    test_from!(
        from_isize_over,
        from_isize,
        isize,
        i8,
        i8::MAX as isize + 1,
        None
    );
    test_from!(
        from_isize_under,
        from_isize,
        isize,
        i8,
        i8::MIN as isize - 1,
        None
    );

    // from i8
    test_from!(from_i8_id, from_i8, i8, i8, 42, Some(42));

    // from i16
    test_from!(from_i16_fit, from_i16, i16, i8, 42, Some(42));
    test_from!(from_i16_over, from_i16, i16, i8, i8::MAX as i16 + 1, None);
    test_from!(from_i16_under, from_i16, i16, i8, i8::MIN as i16 - 1, None);

    // from i32
    test_from!(from_i32_over, from_i32, i32, i8, i8::MAX as i32 + 1, None);
    test_from!(from_i32_under, from_i32, i32, i8, i8::MIN as i32 - 1, None);

    // from i64
    test_from!(from_i64_over, from_i64, i64, i8, i8::MAX as i64 + 1, None);
    test_from!(from_i64_under, from_i64, i64, i8, i8::MIN as i64 - 1, None);

    // from i128
    test_from!(
        from_i128_over,
        from_i128,
        i128,
        i8,
        i8::MAX as i128 + 1,
        None
    );
    test_from!(
        from_i128_under,
        from_i128,
        i128,
        i8,
        i8::MIN as i128 - 1,
        None
    );

    // from usize
    test_from!(from_usize_fit, from_usize, usize, i8, 42, Some(42));
    test_from!(
        from_usize_max,
        from_usize,
        usize,
        i8,
        i8::MAX as usize,
        Some(i8::MAX)
    );
    test_from!(
        from_usize_over,
        from_usize,
        usize,
        i8,
        i8::MAX as usize + 1,
        None
    );

    // from u8
    test_from!(from_u8_fit, from_u8, u8, i8, 42, Some(42));
    test_from!(from_u8_max, from_u8, u8, i8, i8::MAX as u8, Some(i8::MAX));
    test_from!(from_u8_over, from_u8, u8, i8, i8::MAX as u8 + 1, None);

    // from u16
    test_from!(from_u16_over, from_u16, u16, i8, i8::MAX as u16 + 1, None);

    // from u32
    test_from!(from_u32_over, from_u32, u32, i8, i8::MAX as u32 + 1, None);

    // from u64
    test_from!(from_u64_over, from_u64, u64, i8, i8::MAX as u64 + 1, None);

    // from u128
    test_from!(
        from_u128_over,
        from_u128,
        u128,
        i8,
        i8::MAX as u128 + 1,
        None
    );

    // from f32
    test_from!(from_f32_fit, from_f32, f32, i8, 42.1, Some(42));
    test_from!(from_f32_neg_fit, from_f32, f32, i8, -42.9, Some(-42));
    test_from!(from_f32_over, from_f32, f32, i8, 128.0, None);
    test_from!(from_f32_under, from_f32, f32, i8, -129.0, None);
    test_from!(from_f32_nan, from_f32, f32, i8, f32::NAN, None);
    test_from!(from_f32_inf, from_f32, f32, i8, f32::INFINITY, None);

    // from f64
    test_from!(from_f64_fit, from_f64, f64, i8, 42.1, Some(42));
    test_from!(from_f64_over, from_f64, f64, i8, 128.0, None);
    test_from!(from_f64_under, from_f64, f64, i8, -129.0, None);
}

//================================================================================
// To i16
//================================================================================
mod to_i16_tests {
    use super::*;

    // from i8
    test_from!(
        from_i8_widen,
        from_i8,
        i8,
        i16,
        i8::MIN,
        Some(i8::MIN as i16)
    );
    // from i16
    test_from!(from_i16_id, from_i16, i16, i16, 42, Some(42));
    // from i32
    test_from!(
        from_i32_narrow_over,
        from_i32,
        i32,
        i16,
        i16::MAX as i32 + 1,
        None
    );
    test_from!(
        from_i32_narrow_under,
        from_i32,
        i32,
        i16,
        i16::MIN as i32 - 1,
        None
    );
    // from u16
    test_from!(
        from_u16_narrow_over,
        from_u16,
        u16,
        i16,
        i16::MAX as u16 + 1,
        None
    );
    // from f32
    test_from!(
        from_f32_narrow_over,
        from_f32,
        f32,
        i16,
        i16::MAX as f32 + 1.0,
        None
    );
    test_from!(
        from_f32_narrow_under,
        from_f32,
        f32,
        i16,
        i16::MIN as f32 - 1.0,
        None
    );
}

//================================================================================
// To i32
//================================================================================
mod to_i32_tests {
    use super::*;
    // from i16
    test_from!(
        from_i16_widen,
        from_i16,
        i16,
        i32,
        i16::MIN,
        Some(i16::MIN as i32)
    );
    // from i32
    test_from!(from_i32_id, from_i32, i32, i32, 42, Some(42));
    // from i64
    test_from!(
        from_i64_narrow_over,
        from_i64,
        i64,
        i32,
        i32::MAX as i64 + 1,
        None
    );
    test_from!(
        from_i64_narrow_under,
        from_i64,
        i64,
        i32,
        i32::MIN as i64 - 1,
        None
    );
    // from u32
    test_from!(
        from_u32_narrow_over,
        from_u32,
        u32,
        i32,
        i32::MAX as u32 + 1,
        None
    );
    // from f32
    test_from!(from_f32_fit, from_f32, f32, i32, 16777216.0, Some(16777216));
    test_from!(
        from_f32_fit_min,
        from_f32,
        f32,
        i32,
        -16777216.0,
        Some(-16777216)
    );
}

//================================================================================
// To i64
//================================================================================
mod to_i64_tests {
    use super::*;
    // from i32
    test_from!(
        from_i32_widen,
        from_i32,
        i32,
        i64,
        i32::MIN,
        Some(i32::MIN as i64)
    );
    // from i64
    test_from!(from_i64_id, from_i64, i64, i64, 42, Some(42));
    // from i128
    test_from!(
        from_i128_narrow_over,
        from_i128,
        i128,
        i64,
        i64::MAX as i128 + 1,
        None
    );
    test_from!(
        from_i128_narrow_under,
        from_i128,
        i128,
        i64,
        i64::MIN as i128 - 1,
        None
    );
    // from u64
    test_from!(
        from_u64_narrow_over,
        from_u64,
        u64,
        i64,
        i64::MAX as u64 + 1,
        None
    );
    // from f64
    test_from!(
        from_f64_fit,
        from_f64,
        f64,
        i64,
        9007199254740992.0,
        Some(9007199254740992)
    );
    test_from!(
        from_f64_fit_min,
        from_f64,
        f64,
        i64,
        -9007199254740992.0,
        Some(-9007199254740992)
    );
}

//================================================================================
// To i128
//================================================================================
mod to_i128_tests {
    use super::*;
    // from i64
    test_from!(
        from_i64_widen,
        from_i64,
        i64,
        i128,
        i64::MIN,
        Some(i64::MIN as i128)
    );
    // from i128
    test_from!(from_i128_id, from_i128, i128, i128, 42, Some(42));
    // from u128
    test_from!(
        from_u128_narrow_over,
        from_u128,
        u128,
        i128,
        i128::MAX as u128 + 1,
        None
    );
}

//================================================================================
// To isize
//================================================================================
mod to_isize_tests {
    use super::*;
    // from isize
    test_from!(from_isize_id, from_isize, isize, isize, 42, Some(42));
    // from i64
    #[cfg(target_pointer_width = "32")]
    test_from!(
        from_i64_narrow_over_32,
        from_i64,
        i64,
        isize,
        isize::MAX as i64 + 1,
        None
    );
    #[cfg(target_pointer_width = "32")]
    test_from!(
        from_i64_narrow_under_32,
        from_i64,
        i64,
        isize,
        isize::MIN as i64 - 1,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_i64_widen_64,
        from_i64,
        i64,
        isize,
        i64::MAX,
        Some(i64::MAX as isize)
    );

    // from u64
    #[cfg(target_pointer_width = "32")]
    test_from!(
        from_u64_narrow_over_32,
        from_u64,
        u64,
        isize,
        isize::MAX as u64 + 1,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_u64_narrow_over_64,
        from_u64,
        u64,
        isize,
        isize::MAX as u64 + 1,
        None
    );
}
