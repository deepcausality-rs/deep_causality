/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::{ConstOne, One};

// Macro to generate tests for a specific numeric type
macro_rules! test_one_for_type {
    ($ty:ty, $test_name:ident) => {
        mod $test_name {
            use super::*;

            #[test]
            fn test_one() {
                let one_val: $ty = One::one();
                assert_eq!(one_val, 1 as $ty);
            }

            #[test]
            fn test_const_one() {
                let const_one: $ty = ConstOne::ONE;
                assert_eq!(const_one, 1 as $ty);
            }

            #[test]
            fn test_is_one() {
                let one_val: $ty = 1 as $ty;
                let zero_val: $ty = 0 as $ty;
                let two_val: $ty = 2 as $ty;

                assert!(one_val.is_one());
                assert!(!zero_val.is_one());
                assert!(!two_val.is_one());
            }

            #[test]
            fn test_set_one() {
                let mut val: $ty = 0 as $ty;
                assert!(!val.is_one());
                val.set_one();
                assert!(val.is_one());
            }
        }
    };
}

mod test_uint {
    use super::*;

    // Generate tests for all implemented types
    test_one_for_type!(usize, test_usize);
    test_one_for_type!(u8, test_u8);
    test_one_for_type!(u16, test_u16);
    test_one_for_type!(u32, test_u32);
    test_one_for_type!(u64, test_u64);
    test_one_for_type!(u128, test_u128);
}

mod test_int {
    use super::*;

    test_one_for_type!(isize, test_isize);
    test_one_for_type!(i8, test_i8);
    test_one_for_type!(i16, test_i16);
    test_one_for_type!(i32, test_i32);
    test_one_for_type!(i64, test_i64);
    test_one_for_type!(i128, test_i128);
}
// Special handling for floats due to precision
mod test_f32 {
    use super::*;

    #[test]
    fn test_one() {
        let one_val: f32 = One::one();
        assert_eq!(one_val, 1.0f32);
    }

    #[test]
    fn test_const_one() {
        let const_one: f32 = ConstOne::ONE;
        assert_eq!(const_one, 1.0f32);
    }

    #[test]
    fn test_is_one() {
        let one_val: f32 = 1.0f32;
        let zero_val: f32 = 0.0f32;
        // Test a value that is not one
        let two_val: f32 = 2.0f32;
        // Test a value very close to one
        let almost_one: f32 = 1.0 + f32::EPSILON;

        assert!(one_val.is_one());
        assert!(!zero_val.is_one());
        assert!(!two_val.is_one());
        assert!(!almost_one.is_one());
    }

    #[test]
    fn test_set_one() {
        let mut val: f32 = 0.0f32;
        assert!(!val.is_one());
        val.set_one();
        assert!(val.is_one());
    }
}

mod test_f64 {
    use super::*;

    #[test]
    fn test_one() {
        let one_val: f64 = One::one();
        assert_eq!(one_val, 1.0f64);
    }

    #[test]
    fn test_const_one() {
        let const_one: f64 = ConstOne::ONE;
        assert_eq!(const_one, 1.0f64);
    }

    #[test]
    fn test_is_one() {
        let one_val: f64 = 1.0f64;
        let zero_val: f64 = 0.0f64;
        let two_val: f64 = 2.0f64;
        let almost_one: f64 = 1.0 + f64::EPSILON;

        assert!(one_val.is_one());
        assert!(!zero_val.is_one());
        assert!(!two_val.is_one());
        assert!(!almost_one.is_one());
    }

    #[test]
    fn test_set_one() {
        let mut val: f64 = 0.0f64;
        assert!(!val.is_one());
        val.set_one();
        assert!(val.is_one());
    }
}
