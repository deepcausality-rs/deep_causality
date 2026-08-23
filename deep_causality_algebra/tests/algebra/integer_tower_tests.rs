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
    AbelianGroup, Associative, Commutative, CommutativeRing, Distributive, EuclideanDomain, Field,
    Real, RealField, Ring,
};

fn assert_markers<T: Commutative + Associative + Distributive>() {}
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
    assert_markers::<u16>();
    assert_markers::<u32>();
    assert_markers::<u64>();
    assert_markers::<u128>();
    assert_markers::<usize>();

    // ...but none of these compile, because ℕ has no additive inverses:
    //   assert_abelian::<u64>();
    //   assert_ring::<u64>();
    //   assert_commutative_ring::<u64>();
    //   assert_euclidean::<u64>();
}

#[test]
fn floats_keep_the_full_real_tower() {
    // Admitting the integers must not cost the floats anything.
    for_each_float();
}

fn for_each_float() {
    assert_markers::<f32>();
    assert_abelian::<f32>();
    assert_commutative_ring::<f32>();
    assert_field::<f32>();
    assert_real::<f32>();
    assert_real_field::<f32>();

    assert_markers::<f64>();
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
