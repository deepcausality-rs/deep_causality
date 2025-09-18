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
// To u8
//================================================================================
mod to_u8_tests {
    use super::*;

    // from isize
    test_from!(from_isize_fit, from_isize, isize, u8, 42, Some(42));
    test_from!(
        from_isize_max,
        from_isize,
        isize,
        u8,
        u8::MAX as isize,
        Some(u8::MAX)
    );
    test_from!(
        from_isize_over,
        from_isize,
        isize,
        u8,
        u8::MAX as isize + 1,
        None
    );
    test_from!(from_isize_neg, from_isize, isize, u8, -1, None);

    // from i8
    test_from!(from_i8_fit, from_i8, i8, u8, 42, Some(42));
    test_from!(from_i8_neg, from_i8, i8, u8, -1, None);

    // from i16
    test_from!(from_i16_over, from_i16, i16, u8, u8::MAX as i16 + 1, None);
    test_from!(from_i16_neg, from_i16, i16, u8, -1, None);

    // from u8
    test_from!(from_u8_id, from_u8, u8, u8, 42, Some(42));

    // from u16
    test_from!(from_u16_fit, from_u16, u16, u8, 42, Some(42));
    test_from!(
        from_u16_max,
        from_u16,
        u16,
        u8,
        u8::MAX as u16,
        Some(u8::MAX)
    );
    test_from!(from_u16_over, from_u16, u16, u8, u8::MAX as u16 + 1, None);

    // from f32
    test_from!(from_f32_fit, from_f32, f32, u8, 42.1, Some(42));
    test_from!(from_f32_over, from_f32, f32, u8, 256.0, None);
    test_from!(from_f32_neg, from_f32, f32, u8, -1.0, None);
    test_from!(from_f32_nan, from_f32, f32, u8, f32::NAN, None);
    test_from!(from_f32_inf, from_f32, f32, u8, f32::INFINITY, None);
}

//================================================================================
// To u16
//================================================================================
mod to_u16_tests {
    use super::*;

    // from i8
    test_from!(from_i8_fit, from_i8, i8, u16, 42, Some(42));
    test_from!(from_i8_neg, from_i8, i8, u16, -1, None);

    // from i16
    test_from!(from_i16_fit, from_i16, i16, u16, 42, Some(42));
    test_from!(from_i16_neg, from_i16, i16, u16, -1, None);

    // from u8
    test_from!(
        from_u8_widen,
        from_u8,
        u8,
        u16,
        u8::MAX,
        Some(u8::MAX as u16)
    );

    // from u32
    test_from!(from_u32_over, from_u32, u32, u16, u16::MAX as u32 + 1, None);

    // from f32
    test_from!(
        from_f32_over,
        from_f32,
        f32,
        u16,
        u16::MAX as f32 + 1.0,
        None
    );
    test_from!(from_f32_neg, from_f32, f32, u16, -1.0, None);
}

//================================================================================
// To u32
//================================================================================
mod to_u32_tests {
    use super::*;

    // from i16
    test_from!(from_i16_fit, from_i16, i16, u32, 42, Some(42));
    test_from!(from_i16_neg, from_i16, i16, u32, -1, None);

    // from i32
    test_from!(from_i32_fit, from_i32, i32, u32, 42, Some(42));
    test_from!(from_i32_neg, from_i32, i32, u32, -1, None);

    // from u16
    test_from!(
        from_u16_widen,
        from_u16,
        u16,
        u32,
        u16::MAX,
        Some(u16::MAX as u32)
    );

    // from u64
    test_from!(from_u64_over, from_u64, u64, u32, u32::MAX as u64 + 1, None);

    // from f32
    test_from!(from_f32_fit, from_f32, f32, u32, 16777216.0, Some(16777216));
    test_from!(from_f32_over, from_f32, f32, u32, 4294967296.0, None);
    test_from!(from_f32_neg, from_f32, f32, u32, -1.0, None);
}

//================================================================================
// To u64
//================================================================================
mod to_u64_tests {
    use super::*;

    // from i32
    test_from!(from_i32_fit, from_i32, i32, u64, 42, Some(42));
    test_from!(from_i32_neg, from_i32, i32, u64, -1, None);

    // from i64
    test_from!(from_i64_fit, from_i64, i64, u64, 42, Some(42));
    test_from!(from_i64_neg, from_i64, i64, u64, -1, None);

    // from u32
    test_from!(
        from_u32_widen,
        from_u32,
        u32,
        u64,
        u32::MAX,
        Some(u32::MAX as u64)
    );

    // from u128
    test_from!(
        from_u128_over,
        from_u128,
        u128,
        u64,
        u64::MAX as u128 + 1,
        None
    );

    // from f64
    test_from!(
        from_f64_fit,
        from_f64,
        f64,
        u64,
        9007199254740992.0,
        Some(9007199254740992)
    );
    test_from!(
        from_f64_over,
        from_f64,
        f64,
        u64,
        18446744073709551616.0,
        None
    );
    test_from!(from_f64_neg, from_f64, f64, u64, -1.0, None);
}

//================================================================================
// To u128
//================================================================================
mod to_u128_tests {
    use super::*;

    // from i64
    test_from!(from_i64_fit, from_i64, i64, u128, 42, Some(42));
    test_from!(from_i64_neg, from_i64, i64, u128, -1, None);

    // from i128
    test_from!(from_i128_fit, from_i128, i128, u128, 42, Some(42));
    test_from!(from_i128_neg, from_i128, i128, u128, -1, None);

    // from u64
    test_from!(
        from_u64_widen,
        from_u64,
        u64,
        u128,
        u64::MAX,
        Some(u64::MAX as u128)
    );
}

//================================================================================
// To usize
//================================================================================
mod to_usize_tests {
    use super::*;

    // from isize
    test_from!(from_isize_fit, from_isize, isize, usize, 42, Some(42));
    test_from!(from_isize_neg, from_isize, isize, usize, -1, None);

    // from i64
    #[cfg(target_pointer_width = "32")]
    test_from!(
        from_i64_over_32,
        from_i64,
        i64,
        usize,
        usize::MAX as i64 + 1,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_i64_fit_64,
        from_i64,
        i64,
        usize,
        i64::MAX,
        Some(i64::MAX as usize)
    );
    test_from!(from_i64_neg, from_i64, i64, usize, -1, None);

    // from u64
    #[cfg(target_pointer_width = "32")]
    test_from!(
        from_u64_over_32,
        from_u64,
        u64,
        usize,
        usize::MAX as u64 + 1,
        None
    );
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_u64_fit_64,
        from_u64,
        u64,
        usize,
        u64::MAX,
        Some(u64::MAX as usize)
    );
}
