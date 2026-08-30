/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::{Compose, Hom, Injective, RingHom, Surjective};

struct Double;
impl Hom for Double {
    type Domain = i64;
    type Codomain = i64;
    fn apply(&self, x: i64) -> i64 {
        x * 2
    }
}
impl RingHom for Double {}
impl Injective for Double {}

struct Widen;
impl Hom for Widen {
    type Domain = i32;
    type Codomain = i64;
    fn apply(&self, x: i32) -> i64 {
        i64::from(x)
    }
}
impl RingHom for Widen {}
impl Injective for Widen {}

/// Surjective but not injective and not a ring hom, so composites through it lose those labels.
struct Mod3;
impl Hom for Mod3 {
    type Domain = i64;
    type Codomain = i64;
    fn apply(&self, x: i64) -> i64 {
        x.rem_euclid(3)
    }
}
impl Surjective for Mod3 {}

fn assert_ring_hom<H: RingHom>() {}
fn assert_injective<H: Injective>() {}

#[test]
fn test_composite_applies_in_order() {
    // g ∘ f: widen an i32, then double.
    let c = Compose::new(Widen, Double);
    assert_eq!(c.apply(21_i32), 42_i64);
}

#[test]
fn test_composite_ends_are_the_outer_ones() {
    fn ends<H: Hom>(h: &H, x: H::Domain) -> H::Codomain {
        h.apply(x)
    }
    // Domain is Widen's i32; codomain is Double's i64.
    let c = Compose::new(Widen, Double);
    assert_eq!(ends(&c, 5_i32), 10_i64);
}

#[test]
fn test_ring_hom_is_closed_under_composition() {
    assert_ring_hom::<Compose<Widen, Double>>();
    let c = Compose::new(Widen, Double);
    let (a, b) = (3_i32, 4_i32);
    assert_eq!(c.apply(a + b), c.apply(a) + c.apply(b));
}

#[test]
fn test_injective_is_closed_under_composition() {
    assert_injective::<Compose<Widen, Double>>();
    let c = Compose::new(Widen, Double);
    assert_ne!(c.apply(1), c.apply(2));
}

#[test]
fn test_composite_keeps_only_shared_labels() {
    // Double ∘ Mod3 is a Hom, and Mod3 carries neither RingHom nor Injective, so the composite
    // carries neither either. Only the `Hom` bound resolves.
    fn only_hom<H: Hom>() {}
    only_hom::<Compose<Mod3, Double>>();
    let c = Compose::new(Mod3, Double);
    assert_eq!(c.apply(7), 2); // 7 mod 3 = 1, doubled = 2
}

#[test]
fn test_composition_is_associative_in_value() {
    let left = Compose::new(Compose::new(Widen, Double), Double);
    let right = Compose::new(Widen, Compose::new(Double, Double));
    assert_eq!(left.apply(5_i32), right.apply(5_i32));
    assert_eq!(left.apply(5_i32), 20_i64);
}

#[test]
fn test_new_and_fields() {
    let c = Compose::new(Widen, Double);
    assert_eq!(c.f.apply(2_i32), 2_i64);
    assert_eq!(c.g.apply(2_i64), 4_i64);
}
