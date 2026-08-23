/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where ℚ sits in the algebra tower.
//!
//! The negative cases are compile errors rather than assertions, so each is recorded in a comment
//! beside the positive case it bounds.

use deep_causality_algebra::{
    AbelianGroup, Annihilating, Associative, Commutative, CommutativeRing, Distributive, Field,
    Invertible, Ring, Semiring,
};
use deep_causality_num::Zero;
use deep_causality_num_rational::Rational;

fn assert_markers<T: Commutative + Associative + Distributive>() {}
fn assert_abelian<T: AbelianGroup>() {}
fn assert_annihilating<T: Annihilating>() {}
fn assert_semiring<T: Semiring>() {}
fn assert_invertible<T: Invertible>() {}
fn assert_ring<T: Ring>() {}
fn assert_commutative_ring<T: CommutativeRing>() {}
fn assert_field<T: Field>() {}

#[test]
fn rational_is_a_field() {
    assert_markers::<Rational<i64>>();
    assert_abelian::<Rational<i64>>();
    assert_annihilating::<Rational<i64>>();
    assert_semiring::<Rational<i64>>();
    assert_invertible::<Rational<i64>>();
    assert_ring::<Rational<i64>>();
    assert_commutative_ring::<Rational<i64>>();
    assert_field::<Rational<i64>>();
}

#[test]
fn rational_is_a_field_over_every_signed_width() {
    assert_field::<Rational<i8>>();
    assert_field::<Rational<i16>>();
    assert_field::<Rational<i32>>();
    assert_field::<Rational<i64>>();
    assert_field::<Rational<i128>>();
    assert_field::<Rational<isize>>();
}

#[test]
fn rational_is_not_analytic() {
    // None of these compile, and none should:
    //   assert_real::<Rational<i64>>();
    //   assert_real_field::<Rational<i64>>();
    // `Real` is the analytic axis — sqrt, exp, ln, sin — and ℚ is closed under none of them.
    // That sqrt(2) is irrational is the oldest theorem about this gap.
    //
    // Nor is ℚ a Euclidean domain:
    //   assert_euclidean::<Rational<i64>>();
    // Division with remainder is vacuous in a field, where every non-zero element divides every
    // other exactly.
    assert_field::<Rational<i64>>();
}

#[test]
fn the_field_axiom_that_z_cannot_satisfy() {
    // ℤ reaches CommutativeRing and stops, because integer `/` truncates: 1/5 == 0.
    // Constructing ℚ supplies the missing inverses, so ℚ earns `Invertible` where ℤ cannot.
    assert_commutative_ring::<i64>();
    // assert_field::<i64>();          // does NOT compile
    assert_invertible::<Rational<i64>>();
    assert_field::<Rational<i64>>();
}

#[test]
fn rational_is_a_semiring_because_zero_annihilates() {
    // `0 · a = a · 0 = 0` is a theorem in a ring, where the derivation cancels `0·a` by adding
    // its additive inverse. A semiring has no inverses, so the law is an axiom there and has to
    // be promised separately — which is what the `Annihilating` marker records, and what
    // `Semiring` membership then depends on.
    assert_annihilating::<Rational<i64>>();
    assert_semiring::<Rational<i64>>();
    // Every `Ring` is a `Semiring`, so the two must agree for this type.
    assert_ring::<Rational<i64>>();

    // The law itself, at the values where reduction is doing the work: 0/1 · a/b = 0/b = 0/1.
    let zero = Rational::<i64>::zero();
    for (n, d) in [(3_i64, 7_i64), (-2, 5), (1, 1), (i64::MAX, 2)] {
        let a = Rational::new(n, d);
        assert_eq!(zero * a, zero, "0 · {n}/{d}");
        assert_eq!(a * zero, zero, "{n}/{d} · 0");
    }
}
