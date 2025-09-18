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

mod from_u8_tests {
    use super::*;
    test_from!(to_i8, from_u8, u8, i8, 42, Some(42i8));
    test_from!(to_i8_fail, from_u8, u8, i8, 200, None);
    test_from!(to_u16, from_u8, u8, u16, 42, Some(42u16));
    test_from!(to_u8, from_u8, u8, u8, 42, Some(42u8));
}

mod from_u16_tests {
    use super::*;
    test_from!(to_i16, from_u16, u16, i16, 42, Some(42i16));
    test_from!(to_i16_fail, from_u16, u16, i16, 40000, None);
    test_from!(to_u32, from_u16, u16, u32, 42, Some(42u32));
    test_from!(to_u8_fail, from_u16, u16, u8, 300, None);
}
