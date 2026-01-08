/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::{ConstZero, Zero};

// Macro to generate tests for a specific numeric type
macro_rules! test_zero_for_type {
    ($ty:ty, $test_name:ident) => {
        mod $test_name {
            use super::*;

            #[test]
            fn test_zero() {
                let zero_val: $ty = Zero::zero();
                assert_eq!(zero_val, 0 as $ty);
            }

            #[test]
            fn test_const_zero() {
                let const_zero: $ty = ConstZero::ZERO;
                assert_eq!(const_zero, 0 as $ty);
            }

            #[test]
            fn test_is_zero() {
                let zero_val: $ty = 0 as $ty;
                let one_val: $ty = 1 as $ty;

                assert!(zero_val.is_zero());
                assert!(!one_val.is_zero());
            }

            #[test]
            fn test_set_zero() {
                let mut val: $ty = 1 as $ty;
                assert!(!val.is_zero());
                val.set_zero();
                assert!(val.is_zero());
            }
        }
    };
}

// Generate tests for all implemented types
test_zero_for_type!(usize, test_usize);
test_zero_for_type!(u8, test_u8);
test_zero_for_type!(u16, test_u16);
test_zero_for_type!(u32, test_u32);
test_zero_for_type!(u64, test_u64);
test_zero_for_type!(u128, test_u128);

test_zero_for_type!(isize, test_isize);
test_zero_for_type!(i8, test_i8);
test_zero_for_type!(i16, test_i16);
test_zero_for_type!(i32, test_i32);
test_zero_for_type!(i64, test_i64);
test_zero_for_type!(i128, test_i128);

// Special handling for floats
mod test_f32 {
    use super::*;

    #[test]
    fn test_zero() {
        let zero_val: f32 = Zero::zero();
        assert_eq!(zero_val, 0.0f32);
    }

    #[test]
    fn test_const_zero() {
        let const_zero: f32 = ConstZero::ZERO;
        assert_eq!(const_zero, 0.0f32);
    }

    #[test]
    fn test_is_zero() {
        let zero_val: f32 = 0.0f32;
        let one_val: f32 = 1.0f32;
        // Test with negative zero
        let neg_zero_val: f32 = -0.0f32;

        assert!(zero_val.is_zero());
        assert!(neg_zero_val.is_zero()); // -0.0 should also be considered zero
        assert!(!one_val.is_zero());
    }

    #[test]
    fn test_set_zero() {
        let mut val: f32 = 1.0f32;
        assert!(!val.is_zero());
        val.set_zero();
        assert!(val.is_zero());
    }
}

mod test_f64 {
    use super::*;

    #[test]
    fn test_zero() {
        let zero_val: f64 = Zero::zero();
        assert_eq!(zero_val, 0.0f64);
    }

    #[test]
    fn test_const_zero() {
        let const_zero: f64 = ConstZero::ZERO;
        assert_eq!(const_zero, 0.0f64);
    }

    #[test]
    fn test_is_zero() {
        let zero_val: f64 = 0.0f64;
        let one_val: f64 = 1.0f64;
        let neg_zero_val: f64 = -0.0f64;

        assert!(zero_val.is_zero());
        assert!(neg_zero_val.is_zero()); // -0.0 should also be considered zero
        assert!(!one_val.is_zero());
    }

    #[test]
    fn test_set_zero() {
        let mut val: f64 = 1.0f64;
        assert!(!val.is_zero());
        val.set_zero();
        assert!(val.is_zero());
    }
}
