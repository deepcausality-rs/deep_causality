/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{AddSemigroup, EuclideanDomain, MulSemigroup};

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
