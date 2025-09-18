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

mod from_f32_tests {
    use super::*;
    test_from!(to_i32, from_f32, f32, i32, 3.0, Some(3i32));
    test_from!(to_i32_neg, from_f32, f32, i32, -3.0, Some(-3i32));
    test_from!(to_u32, from_f32, f32, u32, 3.0, Some(3u32));
    test_from!(to_u32_neg_fail, from_f32, f32, u32, -3.0, None);
    test_from!(to_i8_overflow, from_f32, f32, i8, 300.0, None);
    test_from!(to_f64, from_f32, f32, f64, 3.0, Some(3.0f32 as f64));
}

mod from_f64_tests {
    use super::*;
    test_from!(to_i64, from_f64, f64, i64, 3.0, Some(3i64));
    test_from!(to_i64_neg, from_f64, f64, i64, -3.0, Some(-3i64));
    test_from!(to_u64, from_f64, f64, u64, 3.0, Some(3u64));
    test_from!(to_u64_neg_fail, from_f64, f64, u64, -3.0, None);
    test_from!(to_i32_overflow, from_f64, f64, i32, 2147483648.0, None);
    test_from!(to_f32, from_f64, f64, f32, 3.0, Some(3.0f64 as f32));
}
