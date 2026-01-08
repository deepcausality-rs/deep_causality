/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Integer, One, Ring, Zero};

/// Test that constants are correct for various integer types.
#[test]
fn test_integer_constants() {
    // i32
    assert_eq!(<i32 as Integer>::MIN, i32::MIN);
    assert_eq!(<i32 as Integer>::MAX, i32::MAX);
    assert_eq!(<i32 as Integer>::BITS, 32);

    // u64
    assert_eq!(<u64 as Integer>::MIN, u64::MIN);
    assert_eq!(<u64 as Integer>::MAX, u64::MAX);
    assert_eq!(<u64 as Integer>::BITS, 64);

    // isize
    assert_eq!(<isize as Integer>::MIN, isize::MIN);
    assert_eq!(<isize as Integer>::MAX, isize::MAX);

    // usize
    assert_eq!(<usize as Integer>::MIN, usize::MIN);
    assert_eq!(<usize as Integer>::MAX, usize::MAX);
}

/// Test bit counting operations.
#[test]
fn test_bit_operations() {
    let x: u32 = 0b10101010;
    assert_eq!(x.count_ones(), 4);
    assert_eq!(x.count_zeros(), 28);
    assert_eq!(x.leading_zeros(), 24);
    assert_eq!(x.trailing_zeros(), 1);

    let y: i64 = 0x0F00;
    assert_eq!(y.count_ones(), 4);
    assert_eq!(y.trailing_zeros(), 8);
}

/// Test endianness conversions.
#[test]
fn test_endianness() {
    let x: u32 = 0x12345678;
    let swapped = x.swap_bytes();
    assert_eq!(swapped, 0x78563412);
    assert_eq!(swapped.swap_bytes(), x);

    // Round-trip through be/le
    assert_eq!(Integer::from_be(x.to_be()), x);
    assert_eq!(Integer::from_le(x.to_le()), x);
}

/// Test checked arithmetic operations.
#[test]
fn test_checked_arithmetic() {
    // Addition overflow
    assert_eq!(i8::MAX.checked_add(1), None);
    assert_eq!(100i32.checked_add(50), Some(150));

    // Subtraction underflow
    assert_eq!(0u32.checked_sub(1), None);
    assert_eq!(100i32.checked_sub(50), Some(50));

    // Multiplication overflow
    assert_eq!(i8::MAX.checked_mul(2), None);
    assert_eq!(10i32.checked_mul(5), Some(50));

    // Division by zero
    assert_eq!(10i32.checked_div(0), None);
    assert_eq!(10i32.checked_div(2), Some(5));

    // Remainder by zero
    assert_eq!(10i32.checked_rem(0), None);
    assert_eq!(10i32.checked_rem(3), Some(1));
}

/// Test saturating arithmetic.
#[test]
fn test_saturating_arithmetic() {
    assert_eq!(i8::MAX.saturating_add(10), i8::MAX);
    assert_eq!(i8::MIN.saturating_sub(10), i8::MIN);
    assert_eq!(100i16.saturating_mul(1000), i16::MAX); // 100000 > 32767

    assert_eq!(u8::MAX.saturating_add(10), u8::MAX);
    assert_eq!(0u8.saturating_sub(10), 0u8);
}

/// Test wrapping arithmetic.
#[test]
fn test_wrapping_arithmetic() {
    assert_eq!(u8::MAX.wrapping_add(1), 0u8);
    assert_eq!(0u8.wrapping_sub(1), u8::MAX);
    assert_eq!(128u8.wrapping_mul(2), 0u8);

    assert_eq!(i8::MAX.wrapping_add(1), i8::MIN);
    assert_eq!(i8::MIN.wrapping_sub(1), i8::MAX);
}

/// Test power function.
#[test]
fn test_pow() {
    assert_eq!(2i32.pow(10), 1024);
    assert_eq!(3u64.pow(5), 243);
    assert_eq!((-2i32).pow(3), -8);
    assert_eq!((-2i32).pow(4), 16);
}

/// Test Euclidean division and remainder.
#[test]
fn test_euclidean_div_rem() {
    // For positive numbers, same as regular div/rem
    assert_eq!(7i32.div_euclid(3), 2);
    assert_eq!(7i32.rem_euclid(3), 1);

    // For negative dividend, result differs from truncated division
    assert_eq!((-7i32).div_euclid(3), -3);
    assert_eq!((-7i32).rem_euclid(3), 2); // Always non-negative

    // For negative divisor
    assert_eq!(7i32.div_euclid(-3), -2);
    assert_eq!(7i32.rem_euclid(-3), 1);
}

/// Test that Integer implies Ring (algebraic hierarchy).
#[test]
fn test_integer_is_ring() {
    fn require_ring<T: Ring>() {}

    require_ring::<i8>();
    require_ring::<i16>();
    require_ring::<i32>();
    require_ring::<i64>();
    require_ring::<i128>();
    require_ring::<isize>();
    require_ring::<u8>();
    require_ring::<u16>();
    require_ring::<u32>();
    require_ring::<u64>();
    require_ring::<u128>();
    require_ring::<usize>();
}

/// Test that Integer types have Zero and One.
#[test]
fn test_integer_identity() {
    fn check_identity<T: Integer + Zero + One>() {
        let zero = T::zero();
        let one = T::one();
        assert!(zero.is_zero());
        assert!(one.is_one());
    }

    check_identity::<i32>();
    check_identity::<u64>();
    check_identity::<isize>();
    check_identity::<usize>();
}

/// Generic function test - demonstrates using Integer bound.
#[test]
fn test_generic_function() {
    fn safe_increment<T: Integer>(val: T) -> Option<T> {
        val.checked_add(T::one())
    }

    assert_eq!(safe_increment(10i32), Some(11));
    assert_eq!(safe_increment(i8::MAX), None);
    assert_eq!(safe_increment(0u8), Some(1));
}
