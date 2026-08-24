/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Hom, Injective, RingHom};
use deep_causality_num_rational::{IntToRational, Rational};

fn assert_ring_hom<H: RingHom>() {}
fn assert_injective<H: Injective>() {}

#[test]
fn test_embeds_as_n_over_one() {
    let e = IntToRational::<i64>::new();
    assert_eq!(e.apply(7), Rational::from_integer(7));
    assert_eq!(e.apply(0), Rational::from_integer(0));
    assert_eq!(e.apply(-5), Rational::from_integer(-5));
}

#[test]
fn test_is_a_ring_hom() {
    assert_ring_hom::<IntToRational<i64>>();
}

#[test]
fn test_additive_law() {
    let e = IntToRational::<i64>::new();
    let (a, b) = (17_i64, 25_i64);
    assert_eq!(e.apply(a + b), e.apply(a) + e.apply(b));
}

#[test]
fn test_multiplicative_law() {
    let e = IntToRational::<i64>::new();
    let (a, b) = (6_i64, 7_i64);
    assert_eq!(e.apply(a * b), e.apply(a) * e.apply(b));
}

#[test]
fn test_unital_law() {
    let e = IntToRational::<i64>::new();
    assert_eq!(e.apply(1), Rational::from_integer(1));
}

#[test]
fn test_is_injective() {
    assert_injective::<IntToRational<i64>>();
    let e = IntToRational::<i64>::new();
    assert_ne!(e.apply(3), e.apply(4));
}

#[test]
fn test_is_not_surjective() {
    // The map's image is exactly the integers-as-rationals, so a half is unreachable.
    let e = IntToRational::<i64>::new();
    let half = Rational::new(1, 2);
    assert!((-50..50).map(|n| e.apply(n)).all(|q| q != half));
}

#[test]
fn test_default_and_new_agree() {
    let a = IntToRational::<i64>::new();
    let b = IntToRational::<i64>::default();
    assert_eq!(a.apply(9), b.apply(9));
    assert_eq!(a, b);
}

#[test]
fn test_works_at_other_widths() {
    assert_eq!(
        IntToRational::<i32>::new().apply(4),
        Rational::from_integer(4_i32)
    );
    assert_eq!(
        IntToRational::<i128>::new().apply(4),
        Rational::from_integer(4_i128)
    );
}
