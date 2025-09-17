/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::FromPrimitive;

macro_rules! test_from_wrapping {
    ($name:ident, $from_method:ident, $from_ty:ty, $to_ty:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from_ty = $val;
            let expected: Option<Wrapping<$to_ty>> = $expected.map(Wrapping);
            let actual = <Wrapping<$to_ty>>::$from_method(v);
            assert_eq!(actual, expected);
        }
    };
}

mod wrapping_from_i8_tests {
    use std::num::Wrapping;

    use super::*;
    test_from_wrapping!(to_i16, from_i8, i8, i16, 42, Some(42i16));
    test_from_wrapping!(to_u8, from_i8, i8, u8, 42, Some(42u8));
    test_from_wrapping!(to_u8_fail, from_i8, i8, u8, -42, None);
}

mod wrapping_from_u8_tests {
    use std::num::Wrapping;

    use super::*;
    test_from_wrapping!(to_i8, from_u8, u8, i8, 42, Some(42i8));
    test_from_wrapping!(to_i8_fail, from_u8, u8, i8, 200, None);
    test_from_wrapping!(to_u16, from_u8, u8, u16, 42, Some(42u16));
}

mod wrapping_from_f32_tests {
    use std::num::Wrapping;

    use super::*;
    test_from_wrapping!(to_i32, from_f32, f32, i32, 3.0, Some(3i32));
    test_from_wrapping!(to_u32_neg_fail, from_f32, f32, u32, -3.0, None);
}
