/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Float, Num, One, Zero};

macro_rules! test_num_integer {
    ($ty:ty, $name:ident) => {
        mod $name {
            use super::*;

            #[test]
            fn test_is_num() {
                fn is_num<T: Num>() {}
                is_num::<$ty>();
            }

            #[test]
            fn test_zero() {
                assert_eq!(<$ty as Zero>::zero(), 0 as $ty);
            }

            #[test]
            fn test_one() {
                assert_eq!(<$ty as One>::one(), 1 as $ty);
            }

            #[test]
            fn test_ops() {
                let a = 2 as $ty;
                let b = 2 as $ty;
                assert_eq!(a + b, 4 as $ty);
                assert_eq!(a - <$ty as One>::one(), 1 as $ty);
                assert_eq!(a * b, 4 as $ty);
                if b != <$ty as Zero>::zero() {
                    assert_eq!(a / b, 1 as $ty);
                    assert_eq!(a % b, 0 as $ty);
                }
            }

            #[test]
            fn test_assign_ops() {
                let mut a = 2 as $ty;
                let b = 2 as $ty;
                a += b;
                assert_eq!(a, 4 as $ty);
                a -= <$ty as One>::one();
                assert_eq!(a, 3 as $ty);
                a *= b;
                assert_eq!(a, 6 as $ty);
                if b != <$ty as Zero>::zero() {
                    a /= b;
                    assert_eq!(a, 3 as $ty);
                    a %= b;
                    assert_eq!(a, 1 as $ty);
                }
            }

            #[test]
            fn test_ref_ops() {
                let a = 2 as $ty;
                let b = 2 as $ty;
                assert_eq!(a + &b, 4 as $ty);
            }

            #[test]
            fn test_ref_assign_ops() {
                let mut a = 2 as $ty;
                let b = 2 as $ty;
                a += &b;
                assert_eq!(a, 4 as $ty);
            }
        }
    };
}

macro_rules! test_num_float {
    ($ty:ty, $name:ident) => {
        mod $name {
            use super::*;

            #[test]
            fn test_is_num() {
                fn is_num<T: Num>() {}
                is_num::<$ty>();
            }

            #[test]
            fn test_zero() {
                assert_eq!(<$ty as Zero>::zero(), 0.0 as $ty);
            }

            #[test]
            fn test_one() {
                assert_eq!(<$ty as One>::one(), 1.0 as $ty);
            }

            #[test]
            fn test_ops() {
                let a = 2.0 as $ty;
                let b = 2.0 as $ty;
                assert!((a + b - 4.0 as $ty).abs() < <$ty as Float>::epsilon());
                assert!((a - <$ty as One>::one() - 1.0 as $ty).abs() < <$ty as Float>::epsilon());
                assert!((a * b - 4.0 as $ty).abs() < <$ty as Float>::epsilon());
                if b != <$ty as Zero>::zero() {
                    assert!((a / b - 1.0 as $ty).abs() < <$ty as Float>::epsilon());
                    assert!((a % b - 0.0 as $ty).abs() < <$ty as Float>::epsilon());
                }
            }

            #[test]
            fn test_assign_ops() {
                let mut a = 2.0 as $ty;
                let b = 2.0 as $ty;
                a += b;
                assert!((a - 4.0 as $ty).abs() < <$ty as Float>::epsilon());
                a -= <$ty as One>::one();
                assert!((a - 3.0 as $ty).abs() < <$ty as Float>::epsilon());
                a *= b;
                assert!((a - 6.0 as $ty).abs() < <$ty as Float>::epsilon());
                if b != <$ty as Zero>::zero() {
                    a /= b;
                    assert!((a - 3.0 as $ty).abs() < <$ty as Float>::epsilon());
                    a %= b;
                    assert!((a - 1.0 as $ty).abs() < <$ty as Float>::epsilon());
                }
            }

            #[test]
            fn test_ref_ops() {
                let a = 2.0 as $ty;
                let b = 2.0 as $ty;
                assert!((a + &b - 4.0 as $ty).abs() < <$ty as Float>::epsilon());
            }

            #[test]
            fn test_ref_assign_ops() {
                let mut a = 2.0 as $ty;
                let b = 2.0 as $ty;
                a += &b;
                assert!((a - 4.0 as $ty).abs() < <$ty as Float>::epsilon());
            }
        }
    };
}

test_num_integer!(usize, usize_tests);
test_num_integer!(u8, u8_tests);
test_num_integer!(u16, u16_tests);
test_num_integer!(u32, u32_tests);
test_num_integer!(u128, u128_tests);

test_num_integer!(isize, isize_tests);
test_num_integer!(i8, i8_tests);
test_num_integer!(i16, i16_tests);
test_num_integer!(i32, i32_tests);
test_num_integer!(i128, i128_tests);

test_num_float!(f32, f32_tests);
test_num_float!(f64, f64_tests);
