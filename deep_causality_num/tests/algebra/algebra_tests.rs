/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{AddSemigroup, Algebra, EuclideanDomain, Module, MulSemigroup};

/// Test that semigroup traits are implemented for integers.
#[test]
fn test_semigroup_impls() {
    fn require_add_semigroup<T: AddSemigroup>() {}
    fn require_mul_semigroup<T: MulSemigroup>() {}

    require_add_semigroup::<i32>();
    require_add_semigroup::<u64>();
    require_mul_semigroup::<i32>();
    require_mul_semigroup::<f64>();
}

/// Test GCD computation using Euclidean algorithm.
#[test]
fn test_gcd() {
    assert_eq!(EuclideanDomain::gcd(&12i32, &8), 4);
    assert_eq!(EuclideanDomain::gcd(&17i32, &13), 1); // Coprime
    assert_eq!(EuclideanDomain::gcd(&100i32, &0), 100);
    assert_eq!(EuclideanDomain::gcd(&0i32, &50), 50);
    assert_eq!(EuclideanDomain::gcd(&-12i32, &8), 4); // Works with negatives

    assert_eq!(EuclideanDomain::gcd(&48u64, &18), 6);
    assert_eq!(EuclideanDomain::gcd(&7u32, &11), 1);
}

/// Test LCM computation.
#[test]
fn test_lcm() {
    assert_eq!(EuclideanDomain::lcm(&4i32, &6), 12);
    assert_eq!(EuclideanDomain::lcm(&3i32, &5), 15);
    assert_eq!(EuclideanDomain::lcm(&0i32, &5), 0);
    assert_eq!(EuclideanDomain::lcm(&7i32, &0), 0);

    assert_eq!(EuclideanDomain::lcm(&12u32, &18), 36);
}

/// Test Euclidean division properties.
#[test]
fn test_euclidean_division() {
    // For positive numbers
    let a = 17i32;
    let b = 5i32;
    let q = EuclideanDomain::div_euclid(&a, &b);
    let r = EuclideanDomain::rem_euclid(&a, &b);
    assert_eq!(a, b * q + r);
    assert!(r >= 0 && r < b.abs());

    // For negative dividend
    let a = -17i32;
    let b = 5i32;
    let q = EuclideanDomain::div_euclid(&a, &b);
    let r = EuclideanDomain::rem_euclid(&a, &b);
    assert_eq!(a, b * q + r);
    assert!(r >= 0 && r < b.abs()); // Remainder always non-negative
}

/// Test Euclidean function values.
#[test]
fn test_euclidean_fn() {
    assert_eq!(EuclideanDomain::euclidean_fn(&-5i32), 5u32);
    assert_eq!(EuclideanDomain::euclidean_fn(&5i32), 5u32);
    assert_eq!(EuclideanDomain::euclidean_fn(&0i32), 0u32);

    assert_eq!(EuclideanDomain::euclidean_fn(&42u64), 42u64);
}

/// Exercise EuclideanDomain trait methods on every signed integer type.
#[test]
fn test_euclidean_signed_all_types() {
    // i8
    assert_eq!(EuclideanDomain::euclidean_fn(&-5i8), 5u8);
    assert_eq!(EuclideanDomain::div_euclid(&17i8, &5i8), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17i8, &5i8), 2);
    assert_eq!(EuclideanDomain::gcd(&12i8, &8i8), 4);
    assert_eq!(EuclideanDomain::lcm(&4i8, &6i8), 12);

    // i16
    assert_eq!(EuclideanDomain::euclidean_fn(&-5i16), 5u16);
    assert_eq!(EuclideanDomain::div_euclid(&17i16, &5i16), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17i16, &5i16), 2);
    assert_eq!(EuclideanDomain::gcd(&12i16, &8i16), 4);
    assert_eq!(EuclideanDomain::lcm(&4i16, &6i16), 12);

    // i64
    assert_eq!(EuclideanDomain::euclidean_fn(&-5i64), 5u64);
    assert_eq!(EuclideanDomain::div_euclid(&17i64, &5i64), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17i64, &5i64), 2);
    assert_eq!(EuclideanDomain::gcd(&12i64, &8i64), 4);
    assert_eq!(EuclideanDomain::lcm(&4i64, &6i64), 12);

    // i128
    assert_eq!(EuclideanDomain::euclidean_fn(&-5i128), 5u128);
    assert_eq!(EuclideanDomain::div_euclid(&17i128, &5i128), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17i128, &5i128), 2);
    assert_eq!(EuclideanDomain::gcd(&12i128, &8i128), 4);
    assert_eq!(EuclideanDomain::lcm(&4i128, &6i128), 12);

    // isize
    assert_eq!(EuclideanDomain::euclidean_fn(&-5isize), 5usize);
    assert_eq!(EuclideanDomain::div_euclid(&17isize, &5isize), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17isize, &5isize), 2);
    assert_eq!(EuclideanDomain::gcd(&12isize, &8isize), 4);
    assert_eq!(EuclideanDomain::lcm(&4isize, &6isize), 12);
}

/// Exercise EuclideanDomain trait methods on every unsigned integer type.
#[test]
fn test_euclidean_unsigned_all_types() {
    // u8
    assert_eq!(EuclideanDomain::euclidean_fn(&5u8), 5u8);
    assert_eq!(EuclideanDomain::div_euclid(&17u8, &5u8), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17u8, &5u8), 2);
    assert_eq!(EuclideanDomain::gcd(&12u8, &8u8), 4);
    assert_eq!(EuclideanDomain::lcm(&4u8, &6u8), 12);

    // u16
    assert_eq!(EuclideanDomain::euclidean_fn(&5u16), 5u16);
    assert_eq!(EuclideanDomain::div_euclid(&17u16, &5u16), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17u16, &5u16), 2);
    assert_eq!(EuclideanDomain::gcd(&12u16, &8u16), 4);
    assert_eq!(EuclideanDomain::lcm(&4u16, &6u16), 12);

    // u128
    assert_eq!(EuclideanDomain::euclidean_fn(&5u128), 5u128);
    assert_eq!(EuclideanDomain::div_euclid(&17u128, &5u128), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17u128, &5u128), 2);
    assert_eq!(EuclideanDomain::gcd(&12u128, &8u128), 4);
    assert_eq!(EuclideanDomain::lcm(&4u128, &6u128), 12);

    // usize
    assert_eq!(EuclideanDomain::euclidean_fn(&5usize), 5usize);
    assert_eq!(EuclideanDomain::div_euclid(&17usize, &5usize), 3);
    assert_eq!(EuclideanDomain::rem_euclid(&17usize, &5usize), 2);
    assert_eq!(EuclideanDomain::gcd(&12usize, &8usize), 4);
    assert_eq!(EuclideanDomain::lcm(&4usize, &6usize), 12);
}

/// Cover the Module::scale and Module::scale_mut blanket-impl helpers.
#[test]
fn test_module_scale_helpers() {
    let mut x: f64 = 3.0;
    let scaled = <f64 as Module<f64>>::scale(&x, 4.0);
    assert_eq!(scaled, 12.0);
    <f64 as Module<f64>>::scale_mut(&mut x, 2.0);
    assert_eq!(x, 6.0);
}

/// Cover the Algebra::sqr default helper.
#[test]
fn test_algebra_sqr() {
    let x: f64 = 5.0;
    assert_eq!(<f64 as Algebra<f64>>::sqr(&x), 25.0);
}

/// Test GCD properties.
#[test]
fn test_gcd_properties() {
    let a = 36i32;
    let b = 24i32;
    let g = EuclideanDomain::gcd(&a, &b);

    // GCD divides both
    assert_eq!(a % g, 0);
    assert_eq!(b % g, 0);

    // GCD is greatest
    for d in 1..=g {
        if a % d == 0 && b % d == 0 {
            assert!(d <= g);
        }
    }
}
