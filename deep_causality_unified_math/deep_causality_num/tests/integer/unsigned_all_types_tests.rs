/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive tests for UnsignedInt trait across all unsigned integer types.

use deep_causality_num::UnsignedInt;

// =============================================================================
// Macro to generate tests for each unsigned integer type
// =============================================================================

macro_rules! test_unsigned_int {
    ($ty:ty, $mod_name:ident) => {
        mod $mod_name {
            use super::*;

            // -----------------------------------------------------------------
            // is_power_of_two tests
            // -----------------------------------------------------------------
            #[test]
            fn test_is_power_of_two_one() {
                let x: $ty = 1;
                assert!(UnsignedInt::is_power_of_two(x));
            }

            #[test]
            fn test_is_power_of_two_two() {
                let x: $ty = 2;
                assert!(UnsignedInt::is_power_of_two(x));
            }

            #[test]
            fn test_is_power_of_two_four() {
                let x: $ty = 4;
                assert!(UnsignedInt::is_power_of_two(x));
            }

            #[test]
            fn test_is_power_of_two_eight() {
                let x: $ty = 8;
                assert!(UnsignedInt::is_power_of_two(x));
            }

            #[test]
            fn test_is_power_of_two_zero() {
                let x: $ty = 0;
                assert!(!UnsignedInt::is_power_of_two(x));
            }

            #[test]
            fn test_is_power_of_two_three() {
                let x: $ty = 3;
                assert!(!UnsignedInt::is_power_of_two(x));
            }

            #[test]
            fn test_is_power_of_two_five() {
                let x: $ty = 5;
                assert!(!UnsignedInt::is_power_of_two(x));
            }

            // -----------------------------------------------------------------
            // next_power_of_two tests
            // -----------------------------------------------------------------
            #[test]
            fn test_next_power_of_two_zero() {
                let x: $ty = 0;
                assert_eq!(UnsignedInt::next_power_of_two(x), 1);
            }

            #[test]
            fn test_next_power_of_two_one() {
                let x: $ty = 1;
                assert_eq!(UnsignedInt::next_power_of_two(x), 1);
            }

            #[test]
            fn test_next_power_of_two_two() {
                let x: $ty = 2;
                assert_eq!(UnsignedInt::next_power_of_two(x), 2);
            }

            #[test]
            fn test_next_power_of_two_three() {
                let x: $ty = 3;
                assert_eq!(UnsignedInt::next_power_of_two(x), 4);
            }

            #[test]
            fn test_next_power_of_two_five() {
                let x: $ty = 5;
                assert_eq!(UnsignedInt::next_power_of_two(x), 8);
            }

            #[test]
            fn test_next_power_of_two_seven() {
                let x: $ty = 7;
                assert_eq!(UnsignedInt::next_power_of_two(x), 8);
            }

            #[test]
            fn test_next_power_of_two_eight() {
                let x: $ty = 8;
                assert_eq!(UnsignedInt::next_power_of_two(x), 8);
            }

            #[test]
            fn test_next_power_of_two_nine() {
                let x: $ty = 9;
                assert_eq!(UnsignedInt::next_power_of_two(x), 16);
            }

            // -----------------------------------------------------------------
            // checked_next_power_of_two tests
            // -----------------------------------------------------------------
            #[test]
            fn test_checked_next_power_of_two_zero() {
                let x: $ty = 0;
                assert_eq!(UnsignedInt::checked_next_power_of_two(x), Some(1));
            }

            #[test]
            fn test_checked_next_power_of_two_one() {
                let x: $ty = 1;
                assert_eq!(UnsignedInt::checked_next_power_of_two(x), Some(1));
            }

            #[test]
            fn test_checked_next_power_of_two_three() {
                let x: $ty = 3;
                assert_eq!(UnsignedInt::checked_next_power_of_two(x), Some(4));
            }

            #[test]
            fn test_checked_next_power_of_two_max_overflow() {
                // Value that would overflow: for u8, 129 would need 256
                let x: $ty = <$ty>::MAX;
                assert_eq!(UnsignedInt::checked_next_power_of_two(x), None);
            }

            #[test]
            fn test_checked_next_power_of_two_high_value_overflow() {
                // Half max + 2 should overflow for all types
                let half_max = <$ty>::MAX / 2;
                let x: $ty = half_max + 2;
                assert_eq!(UnsignedInt::checked_next_power_of_two(x), None);
            }
        }
    };
}

// Generate tests for all unsigned integer types
test_unsigned_int!(u8, u8_tests);
test_unsigned_int!(u16, u16_tests);
test_unsigned_int!(u32, u32_tests);
test_unsigned_int!(u64, u64_tests);
test_unsigned_int!(u128, u128_tests);
test_unsigned_int!(usize, usize_tests);
