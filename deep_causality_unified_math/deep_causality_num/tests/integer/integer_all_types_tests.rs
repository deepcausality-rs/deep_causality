/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive tests for Integer trait across all integer types.

use deep_causality_num::Integer;

// =============================================================================
// Macro to generate tests for each integer type
// =============================================================================

macro_rules! test_integer {
    ($ty:ty, $mod_name:ident) => {
        mod $mod_name {
            use super::*;

            // -----------------------------------------------------------------
            // Constants tests
            // -----------------------------------------------------------------
            #[test]
            fn test_min_constant() {
                assert_eq!(<$ty as Integer>::MIN, <$ty>::MIN);
            }

            #[test]
            fn test_max_constant() {
                assert_eq!(<$ty as Integer>::MAX, <$ty>::MAX);
            }

            #[test]
            fn test_bits_constant() {
                assert_eq!(<$ty as Integer>::BITS, <$ty>::BITS);
            }

            // -----------------------------------------------------------------
            // Bit counting tests
            // -----------------------------------------------------------------
            // Expected values are hand-derived from the bit pattern and the type's width, not
            // read back from the inherent method the trait forwards to.
            #[test]
            fn test_count_ones() {
                // 0b1010_1010 has four set bits, and widening or reinterpreting the byte as a
                // signed type preserves the pattern, so the count is four at every width.
                assert_eq!(Integer::count_ones(0b10101010u8 as $ty), 4);
                assert_eq!(Integer::count_ones(0 as $ty), 0);
                assert_eq!(Integer::count_ones(1 as $ty), 1);
                // All bits set: -1 for the signed types, MAX for the unsigned ones.
                let all_ones = !(0 as $ty);
                assert_eq!(Integer::count_ones(all_ones), <$ty>::BITS);
                // A single bit, walked across every position.
                for shift in 0..<$ty>::BITS {
                    assert_eq!(
                        Integer::count_ones((1 as $ty).wrapping_shl(shift)),
                        1,
                        "bit {}",
                        shift
                    );
                }
            }

            #[test]
            fn test_count_zeros() {
                assert_eq!(Integer::count_zeros(0b10101010u8 as $ty), <$ty>::BITS - 4);
                assert_eq!(Integer::count_zeros(0 as $ty), <$ty>::BITS);
                assert_eq!(Integer::count_zeros(!(0 as $ty)), 0);
                // Ones and zeros partition the word at every width.
                for x in [0 as $ty, 1, 0b10101010u8 as $ty, !(0 as $ty)] {
                    assert_eq!(
                        Integer::count_ones(x) + Integer::count_zeros(x),
                        <$ty>::BITS
                    );
                }
            }

            #[test]
            fn test_leading_zeros() {
                // A single bit at position k leaves BITS - 1 - k leading zeros.
                for shift in 0..<$ty>::BITS {
                    let x = (1 as $ty).wrapping_shl(shift);
                    assert_eq!(
                        Integer::leading_zeros(x),
                        <$ty>::BITS - 1 - shift,
                        "single bit at {}",
                        shift
                    );
                }
                assert_eq!(Integer::leading_zeros(0 as $ty), <$ty>::BITS);
                assert_eq!(Integer::leading_zeros(!(0 as $ty)), 0);
            }

            #[test]
            fn test_trailing_zeros() {
                // A single bit at position k leaves exactly k trailing zeros.
                for shift in 0..<$ty>::BITS {
                    let x = (1 as $ty).wrapping_shl(shift);
                    assert_eq!(Integer::trailing_zeros(x), shift, "single bit at {}", shift);
                }
                assert_eq!(Integer::trailing_zeros(0 as $ty), <$ty>::BITS);
                assert_eq!(Integer::trailing_zeros(1 as $ty), 0);
                assert_eq!(Integer::trailing_zeros(8 as $ty), 3);
            }

            // -----------------------------------------------------------------
            // Byte/Endianness tests
            // -----------------------------------------------------------------
            #[test]
            fn test_swap_bytes() {
                // Reversing the bytes of 0x01 moves the set bit to the top byte, which is
                // position BITS - 8. At a width of one byte that is the identity.
                assert_eq!(
                    Integer::swap_bytes(1 as $ty),
                    (1 as $ty).wrapping_shl(<$ty>::BITS - 8)
                );
                // Byte reversal is an involution, and fixes any byte-wise palindrome.
                for x in [
                    0 as $ty,
                    1,
                    0x12,
                    0b10101010u8 as $ty,
                    !(0 as $ty),
                    <$ty>::MAX,
                    <$ty>::MIN,
                ] {
                    assert_eq!(Integer::swap_bytes(Integer::swap_bytes(x)), x);
                }
                assert_eq!(Integer::swap_bytes(0 as $ty), 0);
                assert_eq!(Integer::swap_bytes(!(0 as $ty)), !(0 as $ty));
            }

            #[test]
            fn test_from_be_roundtrip() {
                let x: $ty = 42;
                assert_eq!(Integer::from_be(x.to_be()), x);
            }

            #[test]
            fn test_from_le_roundtrip() {
                let x: $ty = 42;
                assert_eq!(Integer::from_le(x.to_le()), x);
            }

            #[test]
            fn test_to_be() {
                // On a little-endian target the big-endian form is the byte reversal; on a
                // big-endian one it is the value itself. The target decides which, so the
                // expectation is stated against the target rather than against `to_be` itself.
                for x in [0 as $ty, 1, 42, 0x12, !(0 as $ty), <$ty>::MAX, <$ty>::MIN] {
                    let expected = if cfg!(target_endian = "little") {
                        Integer::swap_bytes(x)
                    } else {
                        x
                    };
                    assert_eq!(Integer::to_be(x), expected, "to_be({})", x);
                }
            }

            #[test]
            fn test_to_le() {
                for x in [0 as $ty, 1, 42, 0x12, !(0 as $ty), <$ty>::MAX, <$ty>::MIN] {
                    let expected = if cfg!(target_endian = "little") {
                        x
                    } else {
                        Integer::swap_bytes(x)
                    };
                    assert_eq!(Integer::to_le(x), expected, "to_le({})", x);
                }
                // Exactly one of the two conversions is the identity on any target.
                let sample = 0x12 as $ty;
                assert!(Integer::to_le(sample) == sample || Integer::to_be(sample) == sample);
            }

            // -----------------------------------------------------------------
            // Checked arithmetic tests
            // -----------------------------------------------------------------
            #[test]
            fn test_checked_add_normal() {
                let x: $ty = 10;
                let y: $ty = 20;
                assert_eq!(Integer::checked_add(x, y), Some(30));
            }

            #[test]
            fn test_checked_add_overflow() {
                let x: $ty = <$ty>::MAX;
                let y: $ty = 1;
                assert_eq!(Integer::checked_add(x, y), None);
            }

            #[test]
            fn test_checked_sub_normal() {
                let x: $ty = 30;
                let y: $ty = 10;
                assert_eq!(Integer::checked_sub(x, y), Some(20));
            }

            #[test]
            fn test_checked_mul_normal() {
                let x: $ty = 5;
                let y: $ty = 6;
                assert_eq!(Integer::checked_mul(x, y), Some(30));
            }

            #[test]
            fn test_checked_div_normal() {
                let x: $ty = 30;
                let y: $ty = 5;
                assert_eq!(Integer::checked_div(x, y), Some(6));
            }

            #[test]
            fn test_checked_div_by_zero() {
                let x: $ty = 30;
                let y: $ty = 0;
                assert_eq!(Integer::checked_div(x, y), None);
            }

            #[test]
            fn test_checked_rem_normal() {
                let x: $ty = 17;
                let y: $ty = 5;
                assert_eq!(Integer::checked_rem(x, y), Some(2));
            }

            #[test]
            fn test_checked_rem_by_zero() {
                let x: $ty = 17;
                let y: $ty = 0;
                assert_eq!(Integer::checked_rem(x, y), None);
            }

            // -----------------------------------------------------------------
            // Saturating arithmetic tests
            // -----------------------------------------------------------------
            #[test]
            fn test_saturating_add_normal() {
                let x: $ty = 10;
                let y: $ty = 20;
                assert_eq!(Integer::saturating_add(x, y), 30);
            }

            #[test]
            fn test_saturating_add_saturates() {
                let x: $ty = <$ty>::MAX;
                let y: $ty = 1;
                assert_eq!(Integer::saturating_add(x, y), <$ty>::MAX);
            }

            #[test]
            fn test_saturating_sub_normal() {
                let x: $ty = 30;
                let y: $ty = 10;
                assert_eq!(Integer::saturating_sub(x, y), 20);
            }

            #[test]
            fn test_saturating_mul_normal() {
                let x: $ty = 5;
                let y: $ty = 6;
                assert_eq!(Integer::saturating_mul(x, y), 30);
            }

            // -----------------------------------------------------------------
            // Wrapping arithmetic tests
            // -----------------------------------------------------------------
            #[test]
            fn test_wrapping_add_normal() {
                let x: $ty = 10;
                let y: $ty = 20;
                assert_eq!(Integer::wrapping_add(x, y), 30);
            }

            #[test]
            fn test_wrapping_sub_normal() {
                let x: $ty = 30;
                let y: $ty = 10;
                assert_eq!(Integer::wrapping_sub(x, y), 20);
            }

            #[test]
            fn test_wrapping_mul_normal() {
                let x: $ty = 5;
                let y: $ty = 6;
                assert_eq!(Integer::wrapping_mul(x, y), 30);
            }

            // -----------------------------------------------------------------
            // Power and Euclidean tests
            // -----------------------------------------------------------------
            #[test]
            fn test_pow() {
                let x: $ty = 2;
                assert_eq!(Integer::pow(x, 4), 16);
            }

            #[test]
            fn test_pow_zero() {
                let x: $ty = 5;
                assert_eq!(Integer::pow(x, 0), 1);
            }

            #[test]
            fn test_div_euclid() {
                let x: $ty = 7;
                let y: $ty = 3;
                assert_eq!(Integer::div_euclid(x, y), 2);
            }

            #[test]
            fn test_rem_euclid() {
                let x: $ty = 7;
                let y: $ty = 3;
                assert_eq!(Integer::rem_euclid(x, y), 1);
            }
        }
    };
}

// Generate tests for all integer types
test_integer!(i8, i8_tests);
test_integer!(i16, i16_tests);
test_integer!(i32, i32_tests);
test_integer!(i64, i64_tests);
test_integer!(i128, i128_tests);
test_integer!(isize, isize_tests);
test_integer!(u8, u8_tests);
test_integer!(u16, u16_tests);
test_integer!(u32, u32_tests);
test_integer!(u64, u64_tests);
test_integer!(u128, u128_tests);
test_integer!(usize, usize_tests);
