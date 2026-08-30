/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Gf2;

#[test]
fn test_new_and_bit_round_trip() {
    assert!(!Gf2::new(false).bit());
    assert!(Gf2::new(true).bit());
}

#[test]
fn test_constants() {
    assert_eq!(Gf2::ZERO, Gf2::new(false));
    assert_eq!(Gf2::ONE, Gf2::new(true));
    assert_ne!(Gf2::ZERO, Gf2::ONE);
}

#[test]
fn test_default_is_zero() {
    assert_eq!(Gf2::default(), Gf2::ZERO);
}

#[test]
fn test_from_i64_mod2_on_the_boundary_alphabet() {
    // The alphabet of deep_causality_topology's boundary operators: -1 and 1 are both the F2 one.
    assert_eq!(Gf2::from_i64_mod2(-1), Gf2::ONE);
    assert_eq!(Gf2::from_i64_mod2(0), Gf2::ZERO);
    assert_eq!(Gf2::from_i64_mod2(1), Gf2::ONE);
}

#[test]
fn test_from_i64_mod2_beyond_the_alphabet() {
    assert_eq!(Gf2::from_i64_mod2(2), Gf2::ZERO);
    assert_eq!(Gf2::from_i64_mod2(-2), Gf2::ZERO);
    assert_eq!(Gf2::from_i64_mod2(7), Gf2::ONE);
    assert_eq!(Gf2::from_i64_mod2(-7), Gf2::ONE);
}

#[test]
fn test_ordering_and_hash_are_available() {
    use std::collections::HashSet;
    assert!(Gf2::ZERO < Gf2::ONE);
    let set: HashSet<Gf2> = [Gf2::ZERO, Gf2::ONE, Gf2::ZERO].into_iter().collect();
    assert_eq!(set.len(), 2);
}

#[test]
fn test_debug_and_clone() {
    let a = Gf2::ONE;
    let b = a;
    assert_eq!(a, b.clone());
    assert!(format!("{:?}", a).contains("Gf2"));
}
