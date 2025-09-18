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
mod f32_tests {
    use super::*;
    test_as!(to_i32_truncate, f32, i32, 3.0, 3);
    test_as!(to_i32_truncate_neg, f32, i32, -3.0, -3);
    test_as!(to_u32_truncate, f32, u32, 3.0, 3);
    test_as!(to_f64, f32, f64, 3.0, 3.0f32 as f64);
}

mod f64_tests {
    use super::*;
    test_as!(to_i64_truncate, f64, i64, 3.0, 3);
    test_as!(to_i64_truncate_neg, f64, i64, -3.0, -3);
    test_as!(to_u64_truncate, f64, u64, 3.0, 3);
    test_as!(to_f32, f64, f32, 3.0, 3.0f64 as f32);
}
