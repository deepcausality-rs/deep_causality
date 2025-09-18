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

test_as!(to_u8, char, u8, 'A', 65);
test_as!(to_i8, char, i8, 'A', 65);
test_as!(to_u8_overflow, char, u8, '\u{FF}', 255);
test_as!(to_i8_overflow, char, i8, '\u{FF}', -1);
