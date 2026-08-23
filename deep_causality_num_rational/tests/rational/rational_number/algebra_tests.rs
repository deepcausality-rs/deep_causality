/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where ℚ sits in the algebra tower.
//!
//! The negative cases are compile errors rather than assertions, so each is recorded in a comment
//! beside the positive case it bounds.

use deep_causality_algebra::{
    AbelianGroup, Associative, Commutative, CommutativeRing, Distributive, Field, Invertible, Ring,
};
use deep_causality_num_rational::Rational;

fn assert_markers<T: Commutative + Associative + Distributive>() {}
fn assert_abelian<T: AbelianGroup>() {}
fn assert_invertible<T: Invertible>() {}
fn assert_ring<T: Ring>() {}
fn assert_commutative_ring<T: CommutativeRing>() {}
fn assert_field<T: Field>() {}

#[test]
fn rational_is_a_field() {
    assert_markers::<Rational<i64>>();
    assert_abelian::<Rational<i64>>();
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
