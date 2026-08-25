/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::Hom;

/// Doubling on ℤ: a group homomorphism that is injective and not surjective.
struct Double;
impl Hom for Double {
    type Domain = i64;
    type Codomain = i64;
    fn apply(&self, x: i64) -> i64 {
        x * 2
    }
}

/// A map whose ends genuinely differ, so the associated types carry information.
struct Widen;
impl Hom for Widen {
    type Domain = i32;
    type Codomain = i64;
    fn apply(&self, x: i32) -> i64 {
        i64::from(x)
    }
}

#[test]
fn test_apply() {
    assert_eq!(Double.apply(21), 42);
    assert_eq!(Double.apply(0), 0);
    assert_eq!(Double.apply(-3), -6);
}

#[test]
fn test_domain_and_codomain_are_named() {
    fn round_trip<H>(h: &H, x: H::Domain) -> H::Codomain
    where
        H: Hom,
    {
        h.apply(x)
    }
    assert_eq!(round_trip(&Widen, 7_i32), 7_i64);
}

#[test]
fn test_ends_may_differ() {
    // The point of the associated types: `Widen` goes i32 -> i64, and the codomain holds values
    // the domain cannot.
    let big = Widen.apply(i32::MAX);
    assert_eq!(big, 2_147_483_647_i64);
    assert!(big < i64::MAX);
}
