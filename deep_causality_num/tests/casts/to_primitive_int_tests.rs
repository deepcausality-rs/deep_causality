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

mod isize_to_tests {
    use super::*;
    test_to!(to_i8, to_i8, isize, i8, 42, Some(42));
    test_to!(to_i8_fail, to_i8, isize, i8, 300, None);
    test_to!(to_u8, to_u8, isize, u8, 42, Some(42));
    test_to!(to_u8_fail, to_u8, isize, u8, -42, None);
    test_to!(to_f32, to_f32, isize, f32, 42, Some(42.0f32));
    test_to!(to_f64, to_f64, isize, f64, 42, Some(42.0f64));
}

mod i8_to_tests {
    use super::*;
    test_to!(to_isize, to_isize, i8, isize, 42, Some(42));
    test_to!(to_u8, to_u8, i8, u8, 42, Some(42));
    test_to!(to_u8_fail, to_u8, i8, u8, -42, None);
}

mod i16_to_tests {
    use super::*;
    test_to!(to_i8, to_i8, i16, i8, 42, Some(42));
    test_to!(to_i8_fail, to_i8, i16, i8, 300, None);
    test_to!(to_u16, to_u16, i16, u16, 42, Some(42));
    test_to!(to_u16_fail, to_u16, i16, u16, -42, None);
}

mod i32_to_tests {
    use super::*;
    test_to!(to_i16, to_i16, i32, i16, 42, Some(42));
    test_to!(to_i16_fail, to_i16, i32, i16, 70000, None);
    test_to!(to_u32, to_u32, i32, u32, 42, Some(42));
    test_to!(to_u32_fail, to_u32, i32, u32, -42, None);
}

mod i64_to_tests {
    use super::*;
    test_to!(to_i32, to_i32, i64, i32, 42, Some(42));
    test_to!(to_i32_fail, to_i32, i64, i32, 2147483648i64, None);
    test_to!(to_u64, to_u64, i64, u64, 42, Some(42));
    test_to!(to_u64_fail, to_u64, i64, u64, -42, None);
}

mod i128_to_tests {
    use super::*;
    test_to!(to_i64, to_i64, i128, i64, 42, Some(42));
    test_to!(
        to_i64_fail,
        to_i64,
        i128,
        i64,
        9223372036854775808i128,
        None
    );
    test_to!(to_u128, to_u128, i128, u128, 42, Some(42));
    test_to!(to_u128_fail, to_u128, i128, u128, -42, None);
}

mod usize_to_tests {
    use super::*;
    test_to!(to_isize, to_isize, usize, isize, 42, Some(42));
    test_to!(to_i8, to_i8, usize, i8, 42, Some(42));
    test_to!(to_i8_fail, to_i8, usize, i8, 300, None);
}

mod u8_to_tests {
    use super::*;
    test_to!(to_isize, to_isize, u8, isize, 42, Some(42));
    test_to!(to_i8, to_i8, u8, i8, 42, Some(42));
    test_to!(to_i8_fail, to_i8, u8, i8, 200, None);
}

mod u16_to_tests {
    use super::*;
    test_to!(to_i16, to_i16, u16, i16, 42, Some(42));
    test_to!(to_i16_fail, to_i16, u16, i16, 40000, None);
    test_to!(to_isize, to_isize, u16, isize, 42, Some(42));
}

mod u32_to_tests {
    use super::*;
    test_to!(to_i32, to_i32, u32, i32, 42, Some(42));
    test_to!(to_i32_fail, to_i32, u32, i32, 2147483648u32, None);
    test_to!(to_isize, to_isize, u32, isize, 42, Some(42));
}

mod u64_to_tests {
    use super::*;
    test_to!(to_i64, to_i64, u64, i64, 42, Some(42));
    test_to!(to_i64_fail, to_i64, u64, i64, 9223372036854775808u64, None);
    test_to!(to_isize, to_isize, u64, isize, 42, Some(42));
}

mod u128_to_tests {
    use super::*;
    test_to!(to_i128, to_i128, u128, i128, 42, Some(42));
    test_to!(
        to_i128_fail,
        to_i128,
        u128,
        i128,
        170141183460469231731687303715884105728u128,
        None
    );
    test_to!(to_isize, to_isize, u128, isize, 42, Some(42));
}
