/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::num::Wrapping;
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

mod from_isize_tests {
    use super::*;
    test_from!(
        to_i32,
        from_isize,
        isize,
        Wrapping<i32>,
        42,
        Some(Wrapping(42))
    );
    test_from!(to_u32_fail, from_isize, isize, Wrapping<u32>, -1, None);
}

mod from_i8_tests {
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

mod from_i16_tests {
    use super::*;
    test_from!(to_i32, from_i16, i16, Wrapping<i32>, 42, Some(Wrapping(42)));
    test_from!(to_u16_fail, from_i16, i16, Wrapping<u16>, -1, None);
    test_from!(to_i8_fail, from_i16, i16, Wrapping<i8>, 300, None);
}

mod from_i32_tests {
    use super::*;
    test_from!(to_i64, from_i32, i32, Wrapping<i64>, 42, Some(Wrapping(42)));
    test_from!(to_u32_fail, from_i32, i32, Wrapping<u32>, -1, None);
    test_from!(to_i16_fail, from_i32, i32, Wrapping<i16>, 40_000, None);
}

mod from_i64_tests {
    use super::*;
    test_from!(
        to_i128,
        from_i64,
        i64,
        Wrapping<i128>,
        42,
        Some(Wrapping(42))
    );
    test_from!(to_u64_fail, from_i64, i64, Wrapping<u64>, -1, None);
    test_from!(to_i32_fail, from_i64, i64, Wrapping<i32>, i64::MAX, None);
}

mod from_i128_tests {
    use super::*;
    test_from!(
        to_i128,
        from_i128,
        i128,
        Wrapping<i128>,
        42,
        Some(Wrapping(42))
    );
    test_from!(to_u128_fail, from_i128, i128, Wrapping<u128>, -1, None);
    test_from!(to_i64_fail, from_i128, i128, Wrapping<i64>, i128::MAX, None);
}

mod from_usize_tests {
    use super::*;
    test_from!(
        to_u64,
        from_usize,
        usize,
        Wrapping<u64>,
        42,
        Some(Wrapping(42))
    );
    test_from!(
        to_i32_fail,
        from_usize,
        usize,
        Wrapping<i32>,
        usize::MAX,
        None
    );
}

mod from_u8_tests {
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

mod from_u16_tests {
    use super::*;
    test_from!(to_u32, from_u16, u16, Wrapping<u32>, 42, Some(Wrapping(42)));
    test_from!(to_i16_fail, from_u16, u16, Wrapping<i16>, 40_000, None);
}

mod from_u32_tests {
    use super::*;
    test_from!(to_u64, from_u32, u32, Wrapping<u64>, 42, Some(Wrapping(42)));
    test_from!(to_i32_fail, from_u32, u32, Wrapping<i32>, u32::MAX, None);
}

mod from_u64_tests {
    use super::*;
    test_from!(
        to_u128,
        from_u64,
        u64,
        Wrapping<u128>,
        42,
        Some(Wrapping(42))
    );
    test_from!(to_i64_fail, from_u64, u64, Wrapping<i64>, u64::MAX, None);
}

mod from_u128_tests {
    use super::*;
    test_from!(
        to_u128,
        from_u128,
        u128,
        Wrapping<u128>,
        42,
        Some(Wrapping(42))
    );
    test_from!(
        to_i128_fail,
        from_u128,
        u128,
        Wrapping<i128>,
        u128::MAX,
        None
    );
}

mod from_f32_tests {
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
    test_from!(to_i8_fail, from_f32, f32, Wrapping<i8>, 300.0, None);
}

mod from_f64_tests {
    use super::*;
    test_from!(
        to_i64,
        from_f64,
        f64,
        Wrapping<i64>,
        42.1,
        Some(Wrapping(42i64))
    );
    test_from!(to_u64_neg_fail, from_f64, f64, Wrapping<u64>, -42.1, None);
    test_from!(to_i32_fail, from_f64, f64, Wrapping<i32>, f64::MAX, None);
}
