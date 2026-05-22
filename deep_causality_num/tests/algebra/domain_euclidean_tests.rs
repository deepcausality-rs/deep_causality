/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::EuclideanDomain;

// ---------- Signed integer impls ----------

#[test]
fn test_i8_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&-7i8), 7u8);
    assert_eq!(EuclideanDomain::euclidean_fn(&7i8), 7u8);
    assert_eq!(EuclideanDomain::div_euclid(&17i8, &5), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&-17i8, &5), 3);
    assert_eq!(EuclideanDomain::gcd(&12i8, &8), 4);
    assert_eq!(EuclideanDomain::lcm(&4i8, &6), 12);
}

#[test]
fn test_i16_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&-123i16), 123u16);
    assert_eq!(EuclideanDomain::div_euclid(&1000i16, &7), 142);
    assert_eq!(EuclideanDomain::rem_euclid(&-1000i16, &7), 1);
    assert_eq!(EuclideanDomain::gcd(&100i16, &75), 25);
    assert_eq!(EuclideanDomain::lcm(&6i16, &9), 18);
}

#[test]
fn test_i64_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&-1_000_000i64), 1_000_000u64);
    assert_eq!(EuclideanDomain::div_euclid(&100i64, &7), 14);
    assert_eq!(EuclideanDomain::rem_euclid(&-100i64, &7), 5);
    assert_eq!(EuclideanDomain::gcd(&252i64, &105), 21);
    assert_eq!(EuclideanDomain::lcm(&21i64, &6), 42);
}

#[test]
fn test_i128_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&-42i128), 42u128);
    assert_eq!(EuclideanDomain::div_euclid(&100i128, &7), 14);
    assert_eq!(EuclideanDomain::rem_euclid(&-100i128, &7), 5);
    assert_eq!(EuclideanDomain::gcd(&252i128, &105), 21);
    assert_eq!(EuclideanDomain::lcm(&5i128, &7), 35);
}

#[test]
fn test_isize_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&-9isize), 9usize);
    assert_eq!(EuclideanDomain::div_euclid(&20isize, &6), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&-20isize, &6), 4);
    assert_eq!(EuclideanDomain::gcd(&54isize, &24), 6);
    assert_eq!(EuclideanDomain::lcm(&4isize, &10), 20);
}

// ---------- Unsigned integer impls ----------

#[test]
fn test_u8_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&7u8), 7u8);
    assert_eq!(EuclideanDomain::div_euclid(&17u8, &5), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17u8, &5), 2);
    assert_eq!(EuclideanDomain::gcd(&12u8, &8), 4);
    assert_eq!(EuclideanDomain::lcm(&4u8, &6), 12);
}

#[test]
fn test_u16_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&500u16), 500u16);
    assert_eq!(EuclideanDomain::div_euclid(&1000u16, &7), 142);
    assert_eq!(EuclideanDomain::rem_euclid(&1000u16, &7), 6);
    assert_eq!(EuclideanDomain::gcd(&100u16, &75), 25);
    assert_eq!(EuclideanDomain::lcm(&6u16, &9), 18);
}

#[test]
fn test_u128_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&42u128), 42u128);
    assert_eq!(EuclideanDomain::div_euclid(&100u128, &7), 14);
    assert_eq!(EuclideanDomain::rem_euclid(&100u128, &7), 2);
    assert_eq!(EuclideanDomain::gcd(&252u128, &105), 21);
    assert_eq!(EuclideanDomain::lcm(&5u128, &7), 35);
}

#[test]
fn test_usize_euclidean() {
    assert_eq!(EuclideanDomain::euclidean_fn(&9usize), 9usize);
    assert_eq!(EuclideanDomain::div_euclid(&20usize, &6), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&20usize, &6), 2);
    assert_eq!(EuclideanDomain::gcd(&54usize, &24), 6);
    assert_eq!(EuclideanDomain::lcm(&4usize, &10), 20);
}

// ---------- gcd/lcm edge cases ----------

#[test]
fn test_gcd_self_and_zero_unsigned() {
    // gcd(a, 0) = a
    assert_eq!(EuclideanDomain::gcd(&15u32, &0u32), 15);
    // gcd(0, b) = b
    assert_eq!(EuclideanDomain::gcd(&0u32, &21u32), 21);
    // gcd(a, a) = a
    assert_eq!(EuclideanDomain::gcd(&13u64, &13u64), 13);
}

#[test]
fn test_lcm_both_zero_branches() {
    // Both inputs zero -> zero (covers the early-return is_zero branch).
    assert_eq!(EuclideanDomain::lcm(&0i64, &0i64), 0);
    // One side zero on unsigned
    assert_eq!(EuclideanDomain::lcm(&0u16, &5u16), 0);
    assert_eq!(EuclideanDomain::lcm(&5u16, &0u16), 0);
}
