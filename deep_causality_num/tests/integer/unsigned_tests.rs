/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::UnsignedInt;

/// Test is_power_of_two.
#[test]
fn test_is_power_of_two() {
    assert!(1u32.is_power_of_two());
    assert!(2u32.is_power_of_two());
    assert!(4u32.is_power_of_two());
    assert!(8u32.is_power_of_two());
    assert!(16u32.is_power_of_two());
    assert!(1024u32.is_power_of_two());

    assert!(!0u32.is_power_of_two());
    assert!(!3u32.is_power_of_two());
    assert!(!5u32.is_power_of_two());
    assert!(!6u32.is_power_of_two());
    assert!(!100u32.is_power_of_two());
}

/// Test next_power_of_two.
#[test]
fn test_next_power_of_two() {
    assert_eq!(0u32.next_power_of_two(), 1);
    assert_eq!(1u32.next_power_of_two(), 1);
    assert_eq!(2u32.next_power_of_two(), 2);
    assert_eq!(3u32.next_power_of_two(), 4);
    assert_eq!(5u32.next_power_of_two(), 8);
    assert_eq!(7u32.next_power_of_two(), 8);
    assert_eq!(8u32.next_power_of_two(), 8);
    assert_eq!(9u32.next_power_of_two(), 16);
    assert_eq!(100u32.next_power_of_two(), 128);
}

/// Test checked_next_power_of_two.
#[test]
fn test_checked_next_power_of_two() {
    assert_eq!(0u32.checked_next_power_of_two(), Some(1));
    assert_eq!(1u32.checked_next_power_of_two(), Some(1));
    assert_eq!(3u32.checked_next_power_of_two(), Some(4));
    assert_eq!(100u32.checked_next_power_of_two(), Some(128));

    // Overflow cases
    assert_eq!(u8::MAX.checked_next_power_of_two(), None);
    assert_eq!(129u8.checked_next_power_of_two(), None); // Would be 256, overflow
    assert_eq!((u32::MAX / 2 + 2).checked_next_power_of_two(), None);
}

/// Test is_non_negative (always true for unsigned).
#[test]
fn test_is_non_negative() {
    assert!(0u32.is_non_negative());
    assert!(1u32.is_non_negative());
    assert!(u32::MAX.is_non_negative());
    assert!(u64::MAX.is_non_negative());
}

/// Test abs for unsigned (identity function).
#[test]
fn test_unsigned_abs() {
    assert_eq!(0u32.abs(), 0);
    assert_eq!(42u32.abs(), 42);
    assert_eq!(u64::MAX.abs(), u64::MAX);
}

/// Test all unsigned types implement UnsignedInt.
#[test]
fn test_unsigned_types() {
    fn require_unsigned<T: UnsignedInt>() {}

    require_unsigned::<u8>();
    require_unsigned::<u16>();
    require_unsigned::<u32>();
    require_unsigned::<u64>();
    require_unsigned::<u128>();
    require_unsigned::<usize>();
}

/// Generic function test using UnsignedInt bound.
#[test]
fn test_generic_unsigned_function() {
    fn align_to_power_of_two<T: UnsignedInt>(val: T) -> Option<T> {
        if val.is_power_of_two() {
            Some(val)
        } else {
            val.checked_next_power_of_two()
        }
    }

    assert_eq!(align_to_power_of_two(4u32), Some(4));
    assert_eq!(align_to_power_of_two(5u32), Some(8));
    assert_eq!(align_to_power_of_two(100u64), Some(128));
    assert_eq!(align_to_power_of_two(u8::MAX), None);
}

/// Test various unsigned types with different sizes.
#[test]
fn test_various_sizes() {
    assert!(128u8.is_power_of_two());
    assert_eq!(100u16.next_power_of_two(), 128);
    assert_eq!(1000u32.next_power_of_two(), 1024);
    assert_eq!(1_000_000u64.next_power_of_two(), 1048576);
}
