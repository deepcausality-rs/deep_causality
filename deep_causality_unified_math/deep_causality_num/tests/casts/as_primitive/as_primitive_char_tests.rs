/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::AsPrimitive;

macro_rules! test_char_as {
    ($test_name:ident, $to_type:ty, $char_val:expr, $expected:expr) => {
        #[test]
        fn $test_name() {
            let val: char = $char_val;
            let expected: $to_type = $expected;
            assert_eq!(AsPrimitive::<$to_type>::as_(val), expected);
        }
    };
}

mod to_char {
    use super::*;
    test_char_as!(identity, char, 'a', 'a');
}

mod to_u8 {
    use super::*;
    test_char_as!(ascii, u8, 'a', 97u8);
    test_char_as!(null, u8, '\0', 0u8);
    test_char_as!(extended_ascii, u8, 'ÿ', 255u8); // '\u{FF}'
    test_char_as!(truncate, u8, '\u{100}', 0u8);
    test_char_as!(truncate_max, u8, '\u{10FFFF}', 255u8);
}

mod to_i8 {
    use super::*;
    test_char_as!(ascii, i8, 'a', 97i8);
    test_char_as!(max_i8, i8, '\u{7F}', 127i8);
    test_char_as!(overflow_to_min, i8, '\u{80}', -128i8);
    test_char_as!(overflow_to_neg_one, i8, '\u{FF}', -1i8);
}

mod to_u16 {
    use super::*;
    test_char_as!(ascii, u16, 'a', 97u16);
    test_char_as!(bmp_max, u16, '\u{FFFF}', 65535u16);
    test_char_as!(truncate, u16, '\u{10000}', 0u16);
}

mod to_i16 {
    use super::*;
    test_char_as!(ascii, i16, 'a', 97i16);
    test_char_as!(max_i16, i16, '\u{7FFF}', 32767i16);
    test_char_as!(overflow_to_min, i16, '\u{8000}', -32768i16);
}

mod to_u32 {
    use super::*;
    test_char_as!(ascii, u32, 'a', 97u32);
    test_char_as!(max_char, u32, '\u{10FFFF}', 0x10FFFF);
}

mod to_i32 {
    use super::*;
    test_char_as!(ascii, i32, 'a', 97i32);
    test_char_as!(max_char, i32, '\u{10FFFF}', 0x10FFFF);
}

mod to_u64 {
    use super::*;
    test_char_as!(max_char, u64, '\u{10FFFF}', 0x10FFFF);
}

mod to_i64 {
    use super::*;
    test_char_as!(max_char, i64, '\u{10FFFF}', 0x10FFFF);
}

mod to_u128 {
    use super::*;
    test_char_as!(max_char, u128, '\u{10FFFF}', 0x10FFFF);
}

mod to_i128 {
    use super::*;
    test_char_as!(max_char, i128, '\u{10FFFF}', 0x10FFFF);
}

mod to_usize {
    use super::*;
    test_char_as!(max_char, usize, '\u{10FFFF}', 0x10FFFF);
}

mod to_isize {
    use super::*;
    test_char_as!(max_char, isize, '\u{10FFFF}', 0x10FFFF);
}
