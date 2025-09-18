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

test_as!(to_u8_true, bool, u8, true, 1);
test_as!(to_u8_false, bool, u8, false, 0);
test_as!(to_i8_true, bool, i8, true, 1);
test_as!(to_i8_false, bool, i8, false, 0);
