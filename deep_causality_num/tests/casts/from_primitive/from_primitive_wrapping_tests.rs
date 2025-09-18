/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::FromPrimitive;
use std::num::Wrapping;

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

mod wrapping_from_i8_tests {
    use super::*;
    test_from!(
        to_i16,
        from_i8,
        i8,
        Wrapping<i16>,
        42,
        Some(Wrapping(42i16))
    );
    test_from!(to_u8, from_i8, i8, Wrapping<u8>, 42, Some(Wrapping(42u8)));
    test_from!(to_u8_fail, from_i8, i8, Wrapping<u8>, -42, None);
}

mod wrapping_from_u8_tests {
    use super::*;
    test_from!(
        to_u16,
        from_u8,
        u8,
        Wrapping<u16>,
        42,
        Some(Wrapping(42u16))
    );
    test_from!(to_i8, from_u8, u8, Wrapping<i8>, 42, Some(Wrapping(42i8)));
    test_from!(to_i8_fail, from_u8, u8, Wrapping<i8>, 200, None);
}

mod wrapping_from_f32_tests {
    use super::*;
    test_from!(
        to_i32,
        from_f32,
        f32,
        Wrapping<i32>,
        42.1,
        Some(Wrapping(42i32))
    );
    test_from!(to_u32_neg_fail, from_f32, f32, Wrapping<u32>, -42.1, None);
}
