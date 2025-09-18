/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::AsPrimitive;

macro_rules! test_as {
    ($name:ident, $from:ty, $to:ty, $val:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let v: $from = $val;
            let a: $to = v.as_();
            assert_eq!(a, $expected);
        }
    };
}

mod i8_tests {
    use super::*;
    test_as!(to_u8, i8, u8, 42, 42);
    test_as!(to_u8_neg, i8, u8, -1, 255);
    test_as!(to_i8, i8, i8, 42, 42);
    test_as!(to_i16, i8, i16, 42, 42);
    test_as!(to_f32, i8, f32, 42, 42.0f32);
    test_as!(to_f32_neg, i8, f32, -42, -42.0f32);
    test_as!(to_f64, i8, f64, 42, 42.0f64);
    test_as!(to_f64_neg, i8, f64, -42, -42.0f64);
}

mod i16_tests {
    use super::*;
    test_as!(to_i8_truncation, i16, i8, 300, 44);
    test_as!(to_u16_negative, i16, u16, -1, 65535);
    test_as!(to_i16, i16, i16, 300, 300);
}

mod i32_tests {
    use super::*;
    test_as!(to_i16_truncation, i32, i16, 70000, 4464);
    test_as!(to_i32, i32, i32, 70000, 70000);
}

mod i64_tests {
    use super::*;
    test_as!(to_i32_truncation, i64, i32, i32::MAX as i64 + 1, i32::MIN);
    test_as!(to_i64, i64, i64, i32::MAX as i64 + 1, i32::MAX as i64 + 1);
}

mod i128_tests {
    use super::*;
    test_as!(to_i64_truncation, i128, i64, i64::MAX as i128 + 1, i64::MIN);
    test_as!(
        to_i128,
        i128,
        i128,
        i64::MAX as i128 + 1,
        i64::MAX as i128 + 1
    );
}
