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
// To usize
//================================================================================
mod to_usize_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, usize, 42, Some(42));
    test_from!(from_isize_fail_neg, from_isize, isize, usize, -1, None);

    test_from!(from_i8_ok, from_i8, i8, usize, 42, Some(42));
    test_from!(from_i8_fail_neg, from_i8, i8, usize, -1, None);

    test_from!(from_i16_ok, from_i16, i16, usize, 42, Some(42));
    test_from!(from_i16_fail_neg, from_i16, i16, usize, -1, None);

    test_from!(from_i32_ok, from_i32, i32, usize, 42, Some(42));
    test_from!(from_i32_fail_neg, from_i32, i32, usize, -1, None);

    test_from!(from_i64_ok, from_i64, i64, usize, 42, Some(42));
    test_from!(from_i64_fail_neg, from_i64, i64, usize, -1, None);
    #[cfg(target_pointer_width = "32")]
    test_from!(from_i64_fail_over, from_i64, i64, usize, i64::MAX, None);

    test_from!(from_i128_ok, from_i128, i128, usize, 42, Some(42));
    test_from!(from_i128_fail_neg, from_i128, i128, usize, -1, None);
    test_from!(from_i128_fail_over, from_i128, i128, usize, i128::MAX, None);

    test_from!(from_usize_ok, from_usize, usize, usize, 42, Some(42));

    test_from!(from_u8_ok, from_u8, u8, usize, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, usize, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, usize, 42, Some(42));

    test_from!(from_u64_ok, from_u64, u64, usize, 42, Some(42));
    #[cfg(target_pointer_width = "32")]
    test_from!(from_u64_fail_over, from_u64, u64, usize, u64::MAX, None);

    test_from!(from_u128_ok, from_u128, u128, usize, 42, Some(42));
    test_from!(from_u128_fail_over, from_u128, u128, usize, u128::MAX, None);

    test_from!(from_f32_ok, from_f32, f32, usize, 42.0, Some(42));
    test_from!(from_f32_fail_neg, from_f32, f32, usize, -1.0, None);

    test_from!(from_f64_ok, from_f64, f64, usize, 42.0, Some(42));
    test_from!(from_f64_fail_neg, from_f64, f64, usize, -1.0, None);
}

//================================================================================
// To u8
//================================================================================
mod to_u8_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, u8, 42, Some(42));
    test_from!(from_isize_fail_neg, from_isize, isize, u8, -1, None);
    test_from!(from_isize_fail_over, from_isize, isize, u8, 256, None);

    test_from!(from_i8_ok, from_i8, i8, u8, 42, Some(42));
    test_from!(from_i8_fail_neg, from_i8, i8, u8, -1, None);

    test_from!(from_i16_ok, from_i16, i16, u8, 42, Some(42));
    test_from!(from_i16_fail_neg, from_i16, i16, u8, -1, None);
    test_from!(from_i16_fail_over, from_i16, i16, u8, 256, None);

    test_from!(from_i32_ok, from_i32, i32, u8, 42, Some(42));
    test_from!(from_i32_fail_neg, from_i32, i32, u8, -1, None);
    test_from!(from_i32_fail_over, from_i32, i32, u8, 256, None);

    test_from!(from_i64_ok, from_i64, i64, u8, 42, Some(42));
    test_from!(from_i64_fail_neg, from_i64, i64, u8, -1, None);
    test_from!(from_i64_fail_over, from_i64, i64, u8, 256, None);

    test_from!(from_i128_ok, from_i128, i128, u8, 42, Some(42));
    test_from!(from_i128_fail_neg, from_i128, i128, u8, -1, None);
    test_from!(from_i128_fail_over, from_i128, i128, u8, 256, None);

    test_from!(from_usize_ok, from_usize, usize, u8, 42, Some(42));
    test_from!(from_usize_fail_over, from_usize, usize, u8, 256, None);

    test_from!(from_u8_ok, from_u8, u8, u8, 42, Some(42));

    test_from!(from_u16_ok, from_u16, u16, u8, 42, Some(42));
    test_from!(from_u16_fail_over, from_u16, u16, u8, 256, None);

    test_from!(from_u32_ok, from_u32, u32, u8, 42, Some(42));
    test_from!(from_u32_fail_over, from_u32, u32, u8, 256, None);

    test_from!(from_u64_ok, from_u64, u64, u8, 42, Some(42));
    test_from!(from_u64_fail_over, from_u64, u64, u8, 256, None);

    test_from!(from_u128_ok, from_u128, u128, u8, 42, Some(42));
    test_from!(from_u128_fail_over, from_u128, u128, u8, 256, None);

    test_from!(from_f32_ok, from_f32, f32, u8, 42.0, Some(42));
    test_from!(from_f32_fail_neg, from_f32, f32, u8, -1.0, None);
    test_from!(from_f32_fail_over, from_f32, f32, u8, 256.0, None);

    test_from!(from_f64_ok, from_f64, f64, u8, 42.0, Some(42));
    test_from!(from_f64_fail_neg, from_f64, f64, u8, -1.0, None);
    test_from!(from_f64_fail_over, from_f64, f64, u8, 256.0, None);
}

//================================================================================
// To u16
//================================================================================
mod to_u16_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, u16, 42, Some(42));
    test_from!(from_isize_fail_neg, from_isize, isize, u16, -1, None);
    test_from!(from_isize_fail_over, from_isize, isize, u16, 65536, None);

    test_from!(from_i8_ok, from_i8, i8, u16, 42, Some(42));
    test_from!(from_i8_fail_neg, from_i8, i8, u16, -1, None);

    test_from!(from_i16_ok, from_i16, i16, u16, 42, Some(42));
    test_from!(from_i16_fail_neg, from_i16, i16, u16, -1, None);

    test_from!(from_i32_ok, from_i32, i32, u16, 42, Some(42));
    test_from!(from_i32_fail_neg, from_i32, i32, u16, -1, None);
    test_from!(from_i32_fail_over, from_i32, i32, u16, 65536, None);

    test_from!(from_i64_ok, from_i64, i64, u16, 42, Some(42));
    test_from!(from_i64_fail_neg, from_i64, i64, u16, -1, None);
    test_from!(from_i64_fail_over, from_i64, i64, u16, 65536, None);

    test_from!(from_i128_ok, from_i128, i128, u16, 42, Some(42));
    test_from!(from_i128_fail_neg, from_i128, i128, u16, -1, None);
    test_from!(from_i128_fail_over, from_i128, i128, u16, 65536, None);

    test_from!(from_usize_ok, from_usize, usize, u16, 42, Some(42));
    test_from!(from_usize_fail_over, from_usize, usize, u16, 65536, None);

    test_from!(from_u8_ok, from_u8, u8, u16, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, u16, 42, Some(42));

    test_from!(from_u32_ok, from_u32, u32, u16, 42, Some(42));
    test_from!(from_u32_fail_over, from_u32, u32, u16, 65536, None);

    test_from!(from_u64_ok, from_u64, u64, u16, 42, Some(42));
    test_from!(from_u64_fail_over, from_u64, u64, u16, 65536, None);

    test_from!(from_u128_ok, from_u128, u128, u16, 42, Some(42));
    test_from!(from_u128_fail_over, from_u128, u128, u16, 65536, None);

    test_from!(from_f32_ok, from_f32, f32, u16, 42.0, Some(42));
    test_from!(from_f32_fail_neg, from_f32, f32, u16, -1.0, None);
    test_from!(from_f32_fail_over, from_f32, f32, u16, 65536.0, None);

    test_from!(from_f64_ok, from_f64, f64, u16, 42.0, Some(42));
    test_from!(from_f64_fail_neg, from_f64, f64, u16, -1.0, None);
    test_from!(from_f64_fail_over, from_f64, f64, u16, 65536.0, None);
}

//================================================================================
// To u32
//================================================================================
mod to_u32_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, u32, 42, Some(42));
    test_from!(from_isize_fail_neg, from_isize, isize, u32, -1, None);
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_isize_fail_over,
        from_isize,
        isize,
        u32,
        4294967296,
        None
    );

    test_from!(from_i8_ok, from_i8, i8, u32, 42, Some(42));
    test_from!(from_i8_fail_neg, from_i8, i8, u32, -1, None);

    test_from!(from_i16_ok, from_i16, i16, u32, 42, Some(42));
    test_from!(from_i16_fail_neg, from_i16, i16, u32, -1, None);

    test_from!(from_i32_ok, from_i32, i32, u32, 42, Some(42));
    test_from!(from_i32_fail_neg, from_i32, i32, u32, -1, None);

    test_from!(from_i64_ok, from_i64, i64, u32, 42, Some(42));
    test_from!(from_i64_fail_neg, from_i64, i64, u32, -1, None);
    test_from!(from_i64_fail_over, from_i64, i64, u32, 4294967296, None);

    test_from!(from_i128_ok, from_i128, i128, u32, 42, Some(42));
    test_from!(from_i128_fail_neg, from_i128, i128, u32, -1, None);
    test_from!(from_i128_fail_over, from_i128, i128, u32, 4294967296, None);

    test_from!(from_usize_ok, from_usize, usize, u32, 42, Some(42));
    #[cfg(target_pointer_width = "64")]
    test_from!(
        from_usize_fail_over,
        from_usize,
        usize,
        u32,
        4294967296,
        None
    );

    test_from!(from_u8_ok, from_u8, u8, u32, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, u32, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, u32, 42, Some(42));

    test_from!(from_u64_ok, from_u64, u64, u32, 42, Some(42));
    test_from!(from_u64_fail_over, from_u64, u64, u32, 4294967296, None);

    test_from!(from_u128_ok, from_u128, u128, u32, 42, Some(42));
    test_from!(from_u128_fail_over, from_u128, u128, u32, 4294967296, None);

    test_from!(from_f32_ok, from_f32, f32, u32, 42.0, Some(42));
    test_from!(from_f32_fail_neg, from_f32, f32, u32, -1.0, None);
    test_from!(from_f32_fail_over, from_f32, f32, u32, 4294967296.0, None);

    test_from!(from_f64_ok, from_f64, f64, u32, 42.0, Some(42));
    test_from!(from_f64_fail_neg, from_f64, f64, u32, -1.0, None);
    test_from!(from_f64_fail_over, from_f64, f64, u32, 4294967296.0, None);
}

//================================================================================
// To u64
//================================================================================
mod to_u64_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, u64, 42, Some(42));
    test_from!(from_isize_fail_neg, from_isize, isize, u64, -1, None);

    test_from!(from_i8_ok, from_i8, i8, u64, 42, Some(42));
    test_from!(from_i8_fail_neg, from_i8, i8, u64, -1, None);

    test_from!(from_i16_ok, from_i16, i16, u64, 42, Some(42));
    test_from!(from_i16_fail_neg, from_i16, i16, u64, -1, None);

    test_from!(from_i32_ok, from_i32, i32, u64, 42, Some(42));
    test_from!(from_i32_fail_neg, from_i32, i32, u64, -1, None);

    test_from!(from_i64_ok, from_i64, i64, u64, 42, Some(42));
    test_from!(from_i64_fail_neg, from_i64, i64, u64, -1, None);

    test_from!(from_i128_ok, from_i128, i128, u64, 42, Some(42));
    test_from!(from_i128_fail_neg, from_i128, i128, u64, -1, None);
    test_from!(
        from_i128_fail_over,
        from_i128,
        i128,
        u64,
        u64::MAX as i128 + 1,
        None
    );

    test_from!(from_usize_ok, from_usize, usize, u64, 42, Some(42));

    test_from!(from_u8_ok, from_u8, u8, u64, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, u64, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, u64, 42, Some(42));
    test_from!(from_u64_ok, from_u64, u64, u64, 42, Some(42));

    test_from!(from_u128_ok, from_u128, u128, u64, 42, Some(42));
    test_from!(
        from_u128_fail_over,
        from_u128,
        u128,
        u64,
        u64::MAX as u128 + 1,
        None
    );

    test_from!(from_f32_ok, from_f32, f32, u64, 42.0, Some(42));
    test_from!(from_f32_fail_neg, from_f32, f32, u64, -1.0, None);

    test_from!(from_f64_ok, from_f64, f64, u64, 42.0, Some(42));
    test_from!(from_f64_fail_neg, from_f64, f64, u64, -1.0, None);
}

//================================================================================
// To u128
//================================================================================
mod to_u128_tests {
    use super::*;

    test_from!(from_isize_ok, from_isize, isize, u128, 42, Some(42));
    test_from!(from_isize_fail_neg, from_isize, isize, u128, -1, None);

    test_from!(from_i8_ok, from_i8, i8, u128, 42, Some(42));
    test_from!(from_i8_fail_neg, from_i8, i8, u128, -1, None);

    test_from!(from_i16_ok, from_i16, i16, u128, 42, Some(42));
    test_from!(from_i16_fail_neg, from_i16, i16, u128, -1, None);

    test_from!(from_i32_ok, from_i32, i32, u128, 42, Some(42));
    test_from!(from_i32_fail_neg, from_i32, i32, u128, -1, None);

    test_from!(from_i64_ok, from_i64, i64, u128, 42, Some(42));
    test_from!(from_i64_fail_neg, from_i64, i64, u128, -1, None);

    test_from!(from_i128_ok, from_i128, i128, u128, 42, Some(42));
    test_from!(from_i128_fail_neg, from_i128, i128, u128, -1, None);

    test_from!(from_usize_ok, from_usize, usize, u128, 42, Some(42));

    test_from!(from_u8_ok, from_u8, u8, u128, 42, Some(42));
    test_from!(from_u16_ok, from_u16, u16, u128, 42, Some(42));
    test_from!(from_u32_ok, from_u32, u32, u128, 42, Some(42));
    test_from!(from_u64_ok, from_u64, u64, u128, 42, Some(42));
    test_from!(from_u128_ok, from_u128, u128, u128, 42, Some(42));

    test_from!(from_f32_ok, from_f32, f32, u128, 42.0, Some(42));
    test_from!(from_f32_fail_neg, from_f32, f32, u128, -1.0, None);

    test_from!(from_f64_ok, from_f64, f64, u128, 42.0, Some(42));
    test_from!(from_f64_fail_neg, from_f64, f64, u128, -1.0, None);
}
