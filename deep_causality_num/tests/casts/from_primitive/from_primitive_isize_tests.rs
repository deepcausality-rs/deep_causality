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

mod from_i8_tests {
    use super::*;
    test_from!(to_i16, from_i8, i8, i16, 42, Some(42i16));
    test_from!(to_u8, from_i8, i8, u8, 42, Some(42u8));
    test_from!(to_u8_fail, from_i8, i8, u8, -42, None);
    test_from!(to_i8, from_i8, i8, i8, 42, Some(42i8));
}

mod from_i16_tests {
    use super::*;
    test_from!(to_i32, from_i16, i16, i32, 42, Some(42i32));
    test_from!(to_u16, from_i16, i16, u16, 42, Some(42u16));
    test_from!(to_u16_fail, from_i16, i16, u16, -42, None);
    test_from!(to_i8_fail, from_i16, i16, i8, 300, None);
}
