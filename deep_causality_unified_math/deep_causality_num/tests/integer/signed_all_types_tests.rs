/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive tests for SignedInt trait across all signed integer types.

use deep_causality_num::SignedInt;

// =============================================================================
// Macro to generate tests for each signed integer type
// =============================================================================

macro_rules! test_signed_int {
    ($ty:ty, $mod_name:ident) => {
        mod $mod_name {
            use super::*;

            // -----------------------------------------------------------------
            // abs tests
            // -----------------------------------------------------------------
            #[test]
            fn test_abs_positive() {
                let x: $ty = 42;
                assert_eq!(SignedInt::abs(x), 42);
            }

            #[test]
            fn test_abs_negative() {
                let x: $ty = -42;
                assert_eq!(SignedInt::abs(x), 42);
            }

            #[test]
            fn test_abs_zero() {
                let x: $ty = 0;
                assert_eq!(SignedInt::abs(x), 0);
            }

            // -----------------------------------------------------------------
            // signum tests
            // -----------------------------------------------------------------
            #[test]
            fn test_signum_positive() {
                let x: $ty = 42;
                assert_eq!(SignedInt::signum(x), 1);
            }

            #[test]
            fn test_signum_negative() {
                let x: $ty = -42;
                assert_eq!(SignedInt::signum(x), -1);
            }

            #[test]
            fn test_signum_zero() {
                let x: $ty = 0;
                assert_eq!(SignedInt::signum(x), 0);
            }

            // -----------------------------------------------------------------
            // is_negative tests
            // -----------------------------------------------------------------
            #[test]
            fn test_is_negative_true() {
                let x: $ty = -1;
                assert!(SignedInt::is_negative(x));
            }

            #[test]
            fn test_is_negative_false() {
                let x: $ty = 1;
                assert!(!SignedInt::is_negative(x));
            }

            #[test]
            fn test_is_negative_zero() {
                let x: $ty = 0;
                assert!(!SignedInt::is_negative(x));
            }

            // -----------------------------------------------------------------
            // is_positive tests
            // -----------------------------------------------------------------
            #[test]
            fn test_is_positive_true() {
                let x: $ty = 1;
                assert!(SignedInt::is_positive(x));
            }

            #[test]
            fn test_is_positive_false() {
                let x: $ty = -1;
                assert!(!SignedInt::is_positive(x));
            }

            #[test]
            fn test_is_positive_zero() {
                let x: $ty = 0;
                assert!(!SignedInt::is_positive(x));
            }

            // -----------------------------------------------------------------
            // checked_abs tests
            // -----------------------------------------------------------------
            #[test]
            fn test_checked_abs_normal() {
                let x: $ty = -42;
                assert_eq!(SignedInt::checked_abs(x), Some(42));
            }

            #[test]
            fn test_checked_abs_min_overflow() {
                let x: $ty = <$ty>::MIN;
                assert_eq!(SignedInt::checked_abs(x), None);
            }

            #[test]
            fn test_checked_abs_zero() {
                let x: $ty = 0;
                assert_eq!(SignedInt::checked_abs(x), Some(0));
            }

            // -----------------------------------------------------------------
            // checked_neg tests
            // -----------------------------------------------------------------
            #[test]
            fn test_checked_neg_normal() {
                let x: $ty = 42;
                assert_eq!(SignedInt::checked_neg(x), Some(-42));
            }

            #[test]
            fn test_checked_neg_min_overflow() {
                let x: $ty = <$ty>::MIN;
                assert_eq!(SignedInt::checked_neg(x), None);
            }

            #[test]
            fn test_checked_neg_zero() {
                let x: $ty = 0;
                assert_eq!(SignedInt::checked_neg(x), Some(0));
            }

            // -----------------------------------------------------------------
            // saturating_abs tests
            // -----------------------------------------------------------------
            #[test]
            fn test_saturating_abs_normal() {
                let x: $ty = -42;
                assert_eq!(SignedInt::saturating_abs(x), 42);
            }

            #[test]
            fn test_saturating_abs_min_saturates() {
                let x: $ty = <$ty>::MIN;
                assert_eq!(SignedInt::saturating_abs(x), <$ty>::MAX);
            }

            // -----------------------------------------------------------------
            // wrapping_abs tests
            // -----------------------------------------------------------------
            #[test]
            fn test_wrapping_abs_normal() {
                let x: $ty = -42;
                assert_eq!(SignedInt::wrapping_abs(x), 42);
            }

            #[test]
            fn test_wrapping_abs_min_wraps() {
                let x: $ty = <$ty>::MIN;
                assert_eq!(SignedInt::wrapping_abs(x), <$ty>::MIN);
            }

            // -----------------------------------------------------------------
            // wrapping_neg tests
            // -----------------------------------------------------------------
            #[test]
            fn test_wrapping_neg_normal() {
                let x: $ty = 42;
                assert_eq!(SignedInt::wrapping_neg(x), -42);
            }

            #[test]
            fn test_wrapping_neg_min_wraps() {
                let x: $ty = <$ty>::MIN;
                assert_eq!(SignedInt::wrapping_neg(x), <$ty>::MIN);
            }

            #[test]
            fn test_wrapping_neg_zero() {
                let x: $ty = 0;
                assert_eq!(SignedInt::wrapping_neg(x), 0);
            }
        }
    };
}

// Generate tests for all signed integer types
test_signed_int!(i8, i8_tests);
test_signed_int!(i16, i16_tests);
test_signed_int!(i32, i32_tests);
test_signed_int!(i64, i64_tests);
test_signed_int!(i128, i128_tests);
test_signed_int!(isize, isize_tests);
