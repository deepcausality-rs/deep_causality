/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

macro_rules! test_as_primitive_bool {
    ($to_type:ty) => {
        use deep_causality_num::AsPrimitive;

        #[test]
        fn test_true() {
            let val: bool = true;
            let expected: $to_type = 1 as $to_type;
            assert_eq!(AsPrimitive::<$to_type>::as_(val), expected);
        }

        #[test]
        fn test_false() {
            let val: bool = false;
            let expected: $to_type = 0 as $to_type;
            assert_eq!(AsPrimitive::<$to_type>::as_(val), expected);
        }
    };
}

mod to_u8 {
    test_as_primitive_bool!(u8);
}
mod to_u16 {
    test_as_primitive_bool!(u16);
}
mod to_u32 {
    test_as_primitive_bool!(u32);
}
mod to_u64 {
    test_as_primitive_bool!(u64);
}
mod to_u128 {
    test_as_primitive_bool!(u128);
}
mod to_usize {
    test_as_primitive_bool!(usize);
}

mod to_i8 {
    test_as_primitive_bool!(i8);
}
mod to_i16 {
    test_as_primitive_bool!(i16);
}
mod to_i32 {
    test_as_primitive_bool!(i32);
}
mod to_i64 {
    test_as_primitive_bool!(i64);
}
mod to_i128 {
    test_as_primitive_bool!(i128);
}
mod to_isize {
    test_as_primitive_bool!(isize);
}
