/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Bijective, Hom, Injective, Surjective};

/// Injective and not surjective: doubling on ℤ misses every odd number.
struct Double;
impl Hom for Double {
    type Domain = i64;
    type Codomain = i64;
    fn apply(&self, x: i64) -> i64 {
        x * 2
    }
}
impl Injective for Double {}

/// Surjective and not injective: `n ↦ n mod 3` hits every residue and collapses infinitely many.
struct Mod3;
impl Hom for Mod3 {
    type Domain = i64;
    type Codomain = i64;
    fn apply(&self, x: i64) -> i64 {
        x.rem_euclid(3)
    }
}
impl Surjective for Mod3 {}

/// Both: negation on ℤ.
struct Negate;
impl Hom for Negate {
    type Domain = i64;
    type Codomain = i64;
    fn apply(&self, x: i64) -> i64 {
        -x
    }
}
impl Injective for Negate {}
impl Surjective for Negate {}

fn assert_injective<H: Injective>() {}
fn assert_surjective<H: Surjective>() {}
fn assert_bijective<H: Bijective>() {}

#[test]
fn test_injective_separates_points() {
    assert_injective::<Double>();
    assert_ne!(Double.apply(3), Double.apply(4));
    assert_eq!(Double.apply(3), Double.apply(3));
}

#[test]
fn test_injective_need_not_be_surjective() {
    // 5 is odd, so nothing doubles to it: the image misses it.
    assert!((0..100).map(|n| Double.apply(n)).all(|v| v % 2 == 0));
}

#[test]
fn test_surjective_covers_the_codomain() {
    assert_surjective::<Mod3>();
    let hit: Vec<i64> = (0..3).map(|n| Mod3.apply(n)).collect();
    assert_eq!(hit, vec![0, 1, 2]);
}

#[test]
fn test_surjective_need_not_be_injective() {
    assert_eq!(Mod3.apply(1), Mod3.apply(4));
    assert_eq!(Mod3.apply(1), Mod3.apply(7));
}

#[test]
fn test_bijective_is_blanket_derived() {
    // `Bijective` is a definition rather than a promise: `Negate` never implements it directly,
    // yet satisfies it because it is both injective and surjective.
    assert_bijective::<Negate>();
    assert_eq!(Negate.apply(Negate.apply(9)), 9);
}

#[test]
fn test_bijective_requires_both() {
    // A generic that demands both resolves only for maps carrying both.
    fn both<H: Injective + Surjective>() {
        assert_bijective::<H>();
    }
    both::<Negate>();
}
