/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for where the integers sit in the algebra tower.
//!
//! ℤ is a `CommutativeRing` and a `EuclideanDomain`, and it stops there: it is not a `Field`,
//! because integer `/` is a truncating quotient rather than a multiplicative inverse. ℕ stops
//! earlier still: the unsigned types satisfy the commutative, associative, and distributive
//! laws, so they are a commutative semiring, but they have no additive inverses and so are not
//! an `AbelianGroup`, and therefore not a `Ring`.
//!
//! The negative cases below are enforced by the type system rather than by assertions — a
//! `Field` bound on `i64`, or an `AbelianGroup` bound on `u64`, is a compile error (E0277), so
//! it cannot be written as a passing runtime test. Each is named in a comment beside the
//! positive case it bounds, so that the intended limit is recorded next to what is asserted.

use deep_causality_algebra::{
    AbelianGroup, AddGroup, AddMonoid, Additive, Annihilating, Associative, Commutative,
    CommutativeRing, CommutativeSemiring, Distributive, EuclideanDomain, Field, Group, MulMonoid,
    Multiplicative, Real, RealField, Ring, Semiring,
};
use deep_causality_num::Zero;

fn assert_markers<T: Commutative<Multiplicative> + Associative<Multiplicative> + Distributive>() {}
/// The additive laws, which the flat markers could not state separately. Every ring's addition is
/// an abelian group, so ℤ and ℕ both carry these.
fn assert_additive_markers<T: Commutative<Additive> + Associative<Additive>>() {}
/// The full commutative-semiring surface, not just the three empty markers: two monoids on one
/// carrier, with `Zero`, `One`, `Add`, and `Mul` actually present.
fn assert_semiring<T: AddMonoid + MulMonoid + Distributive + Commutative<Multiplicative>>() {}
fn assert_abelian<T: AbelianGroup>() {}
fn assert_ring<T: Ring>() {}
fn assert_commutative_ring<T: CommutativeRing>() {}
fn assert_euclidean<T: EuclideanDomain>() {}
fn assert_field<T: Field>() {}
fn assert_real<T: Real>() {}
fn assert_real_field<T: RealField>() {}

#[test]
fn signed_integers_are_commutative_rings() {
    // ℤ: an Abelian group under +, a monoid under ×, distributive, and commutative.
    assert_markers::<i64>();
    assert_additive_markers::<i64>();
    assert_abelian::<i64>();
    assert_ring::<i64>();
    assert_commutative_ring::<i64>();

    assert_commutative_ring::<i8>();
    assert_commutative_ring::<i16>();
    assert_commutative_ring::<i32>();
    assert_commutative_ring::<i128>();
    assert_commutative_ring::<isize>();
}

#[test]
fn signed_integers_are_euclidean_domains() {
    // The level that licenses exact gcd, and with it exact elimination over ℤ.
    // `assert_field::<i64>()` does NOT compile: ℤ has no multiplicative inverses.
    assert_euclidean::<i64>();
    assert_euclidean::<i32>();
}

#[test]
fn unsigned_integers_are_a_semiring_only() {
    // ℕ satisfies the three laws...
    assert_markers::<u8>();
    assert_additive_markers::<u8>();
    assert_markers::<u16>();
    assert_additive_markers::<u16>();
    assert_markers::<u32>();
    assert_additive_markers::<u32>();
    assert_markers::<u64>();
    assert_additive_markers::<u64>();
    assert_markers::<u128>();
    assert_additive_markers::<u128>();
    assert_markers::<usize>();
    assert_additive_markers::<usize>();

    // ...and carry the full two-operation semiring structure, not merely the empty markers.
    assert_semiring::<u8>();
    assert_semiring::<u16>();
    assert_semiring::<u32>();
    assert_semiring::<u64>();
    assert_semiring::<u128>();
    assert_semiring::<usize>();

    // But none of these compile, because ℕ has no additive inverses. `Neg` is the witness for
    // them, and it is what `AddGroup` and `AbelianGroup` both require:
    //   assert_add_group::<u64>();
    //   assert_group::<u64>();
    //   assert_abelian::<u64>();
    //   assert_ring::<u64>();
    //   assert_commutative_ring::<u64>();
    //   assert_euclidean::<u64>();
}

#[test]
fn signed_integers_are_a_semiring_too() {
    // A ring is a semiring, so admitting ℤ to `CommutativeRing` must not cost it the weaker
    // structure. The same holds for the reals.
    assert_semiring::<i64>();
    assert_semiring::<f64>();
}

#[test]
fn additive_inverses_are_gated_on_neg() {
    // `AddGroup` requires `Neg`, not merely `Sub`. `Sub` gives `a - a = 0`, which ℕ satisfies
    // without having a single additive inverse — so requiring only `Sub` would admit ℕ, and
    // `T::zero() - x` would then hand back a garbage "inverse" for `u64`.
    fn assert_add_group<T: AddGroup>() {}
    fn assert_group<T: Group>() {}
    assert_add_group::<i64>();
    assert_group::<i64>();
    assert_add_group::<f64>();
    // `assert_add_group::<u64>()` and `assert_group::<u64>()` do NOT compile.
}

#[test]
fn floats_keep_the_full_real_tower() {
    // Admitting the integers must not cost the floats anything.
    for_each_float();
}

fn for_each_float() {
    assert_markers::<f32>();
    assert_additive_markers::<f32>();
    assert_abelian::<f32>();
    assert_commutative_ring::<f32>();
    assert_field::<f32>();
    assert_real::<f32>();
    assert_real_field::<f32>();

    assert_markers::<f64>();

    assert_additive_markers::<f64>();
    assert_abelian::<f64>();
    assert_commutative_ring::<f64>();
    assert_field::<f64>();
    assert_real::<f64>();
    assert_real_field::<f64>();

    // `assert_euclidean::<f64>()` does NOT compile: a field is a Euclidean domain in the
    // abstract, but the trait here exists to carry exact integer division, and ℝ has no
    // meaningful Euclidean function.
}

#[test]
fn integers_are_not_analytic() {
    // `assert_real::<i64>()` does NOT compile, and should not: `Real` is the analytic axis
    // (sqrt, exp, ln, nan, is_infinite). `sqrt` does not close over ℤ and there is no integer
    // NaN. ℤ reaches `CommutativeRing` and stops.
    assert_commutative_ring::<i64>();
}

#[test]
fn every_ring_is_a_semiring() {
    // Not a claim about particular types: this compiles only if `Semiring`'s bounds are a strict
    // subset of `Ring`'s, for every `T` at once. It is the regression guard for a real inversion —
    // `Semiring` once required `AddMonoid`, which carries `AddAssign<Self>`, while `Ring` requires
    // only `AbelianGroup`, which does not. That made the "weaker" structure strictly stronger on
    // the additive side, and evicted `CausalTensor` (which has `AddAssign<T>` for a scalar
    // right-hand side but no `AddAssign<Self>`) from `Semiring` while it stayed a `Ring`.
    fn ring_implies_semiring<T: Ring>() {
        fn require_semiring<U: Semiring>() {}
        require_semiring::<T>();
    }
    ring_implies_semiring::<f64>();
    ring_implies_semiring::<i64>();
}

#[test]
fn naturals_are_a_semiring_without_being_a_ring() {
    // The other direction: the weakening has to actually admit something new, or it is pointless.
    fn semi<T: Semiring>() {}
    fn csemi<T: CommutativeSemiring>() {}
    semi::<u64>();
    csemi::<u64>();
    csemi::<usize>();
    // `assert_ring::<u64>()` still does NOT compile — N has no additive inverses.
}

#[test]
fn annihilation_is_promised_rather_than_assumed() {
    // `0 * a = 0` is a THEOREM in a ring — the proof cancels by adding -(0*a) — but an AXIOM in a
    // semiring, which has no additive inverse to cancel with. `Semiring` therefore requires the
    // marker rather than inferring the law, and `Ring` carries it too so that every ring stays a
    // semiring.
    fn assert_annihilating<T: Annihilating>() {}
    assert_annihilating::<f64>();
    assert_annihilating::<i64>();
    assert_annihilating::<u64>();

    // And the law itself, at the carriers that matter. Read through variables so the products are
    // evaluated rather than constant-folded away.
    for a in 1u64..=8 {
        let zero = u64::zero();
        assert_eq!(zero * a, zero, "0 * {a} must annihilate");
        assert_eq!(a * zero, zero, "{a} * 0 must annihilate");
    }
    for a in -8i64..=8 {
        let zero = i64::zero();
        assert_eq!(zero * a, zero, "0 * {a} must annihilate over Z");
    }
}

#[test]
fn the_law_markers_are_not_handed_out_by_inference() {
    // The markers are implemented per type rather than blanket-implemented over `Num`, which is
    // unsealed: a downstream type implementing `Num` must not silently acquire a law nobody
    // promised, then enter `CommutativeRing` and `Field` on that claim. These assertions check the
    // supported types still have the markers after that change.
    assert_markers::<f32>();
    assert_additive_markers::<f32>();
    assert_markers::<f64>();
    assert_additive_markers::<f64>();
    assert_markers::<i8>();
    assert_additive_markers::<i8>();
    assert_markers::<i128>();
    assert_additive_markers::<i128>();
    assert_markers::<u8>();
    assert_additive_markers::<u8>();
    assert_markers::<u128>();
    assert_additive_markers::<u128>();
    assert_markers::<usize>();
    assert_additive_markers::<usize>();
    assert_markers::<isize>();
    assert_additive_markers::<isize>();
}
