/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::ToPrimitive;

macro_rules! test_to {
    ($name:ident, $method:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let expected: Option<$to> = $expected;
            let actual = v.$method();
            assert_eq!(actual, expected);
        }
    };
}

mod f32_to_tests {
    use super::*;
    test_to!(to_i8, to_i8, f32, i8, 42.0, Some(42));
    test_to!(to_i8_fail, to_i8, f32, i8, 300.0, None);
    test_to!(to_u8, to_u8, f32, u8, 42.0, Some(42));
    test_to!(to_u8_fail, to_u8, f32, u8, -42.0, None);
    test_to!(to_f64, to_f64, f32, f64, 3.0, Some(3.0f32 as f64));
}

mod f64_to_tests {
    use super::*;
    test_to!(to_i32, to_i32, f64, i32, 42.0, Some(42));
    test_to!(to_i32_fail, to_i32, f64, i32, 2147483648.0, None);
    test_to!(to_u64, to_u64, f64, u64, 42.0, Some(42));
    test_to!(to_u64_fail, to_u64, f64, u64, -42.0, None);
    test_to!(to_f32, to_f32, f64, f32, 3.0, Some(3.0f64 as f32));
}
