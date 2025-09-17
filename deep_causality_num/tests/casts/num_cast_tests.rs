/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::NumCast;

macro_rules! test_cast {
    ($name:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let expected: Option<$to> = $expected;
            let actual: Option<$to> = NumCast::from(v);
            assert_eq!(actual, expected);
        }
    };
}

mod to_u8_tests {
    use super::*;
    test_cast!(from_u16, u16, u8, 42, Some(42));
    test_cast!(from_u16_fail, u16, u8, 300, None);
    test_cast!(from_i8, i8, u8, 42, Some(42));
    test_cast!(from_i8_fail, i8, u8, -42, None);
}

mod to_i8_tests {
    use super::*;
    test_cast!(from_u8, u8, i8, 42, Some(42));
    test_cast!(from_u8_fail, u8, i8, 200, None);
    test_cast!(from_i16, i16, i8, 42, Some(42));
    test_cast!(from_i16_fail, i16, i8, 300, None);
}

mod to_u16_tests {
    use super::*;
    test_cast!(from_u32, u32, u16, 42, Some(42));
    test_cast!(from_u32_fail, u32, u16, 70000, None);
    test_cast!(from_i16, i16, u16, 42, Some(42));
    test_cast!(from_i16_fail, i16, u16, -42, None);
}

mod to_i16_tests {
    use super::*;
    test_cast!(from_u16, u16, i16, 42, Some(42));
    test_cast!(from_u16_fail, u16, i16, 40000, None);
    test_cast!(from_i32, i32, i16, 42, Some(42));
    test_cast!(from_i32_fail, i32, i16, 70000, None);
}

mod to_f32_tests {
    use super::*;
    test_cast!(from_i32, i32, f32, 42, Some(42.0));
    test_cast!(from_f64, f64, f32, 3.0, Some(3.0f64 as f32));
}

mod to_f64_tests {
    use super::*;
    test_cast!(from_i32, i32, f64, 42, Some(42.0));
    test_cast!(from_f32, f32, f64, 3.0, Some(3.0f32 as f64));
}
