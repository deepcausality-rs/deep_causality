/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The scalar bands, as compile-time admissions.
//!
//! Each witness function compiles only if the bound admits the scalar, so calling it is the
//! assertion. These pass before any implementation exists, because they check the type system.
//!
//! The negative half — that `i64` is refused by `Field`, that `u64` is refused by `Ring`, that
//! `Gf2` is refused by `NormedScalar` and `DivisibleByIntegers` — cannot be written as a failing
//! call: this MSRV has no negative impls and the repository has no `trybuild` harness. It is
//! covered by the compile-fail doctest on `DivisibleByIntegers` and by the probes recorded in
//! `openspec/notes/linear/BOUND-LEDGER.md`.

use deep_causality_algebra::{
    CommutativeRing, CommutativeSemiring, ConjugateScalar, DivisibleByIntegers, EuclideanDomain,
    Field, IntegralDomain, NormedScalar, RealField,
};
use deep_causality_num::{Float106, Gf2};
use deep_causality_num_complex::Complex;
use deep_causality_num_rational::Rational;

fn semiring<T: CommutativeSemiring>() {}
fn ring<T: CommutativeRing>() {}
fn integral<T: IntegralDomain>() {}
fn euclidean<T: EuclideanDomain>() {}
fn field<T: Field>() {}
fn divisible<T: DivisibleByIntegers>() {}
fn normed<T: NormedScalar>() {}
fn real<T: RealField>() {}
fn conjugate<T: ConjugateScalar>() {}

#[test]
fn test_the_semiring_band_admits_the_naturals() {
    semiring::<u8>();
    semiring::<u16>();
    semiring::<u32>();
    semiring::<u64>();
    semiring::<usize>();
}

#[test]
fn test_the_ring_band_admits_the_integers_and_everything_above() {
    ring::<i8>();
    ring::<i64>();
    ring::<f64>();
    ring::<Float106>();
    ring::<Complex<f64>>();
    ring::<Rational<i64>>();
    ring::<Gf2>();
}

#[test]
fn test_the_integral_domain_band_admits_what_cancellation_holds_for() {
    integral::<i64>();
    integral::<f64>();
    integral::<Gf2>();
}

#[test]
fn test_the_euclidean_band_admits_the_integers_only() {
    euclidean::<i8>();
    euclidean::<i64>();
    euclidean::<isize>();
}

#[test]
fn test_the_field_band_admits_the_fields_including_gf2() {
    field::<f32>();
    field::<f64>();
    field::<Float106>();
    field::<Complex<f64>>();
    field::<Rational<i64>>();
    field::<Gf2>();
}

#[test]
fn test_the_integer_divisible_band_excludes_gf2_by_admitting_only_characteristic_zero() {
    divisible::<f32>();
    divisible::<f64>();
    divisible::<Float106>();
    divisible::<Complex<f64>>();
    divisible::<Rational<i64>>();
    // Gf2 is deliberately absent: 1 + 1 = 0 there, so halving divides by zero.
}

#[test]
fn test_the_normed_band_admits_the_moduli() {
    normed::<f64>();
    normed::<Float106>();
    normed::<Complex<f64>>();
}

#[test]
fn test_the_real_field_band_admits_the_ordered_reals() {
    real::<f32>();
    real::<f64>();
    real::<Float106>();
}

#[test]
fn test_the_conjugate_band_admits_the_reals_and_the_complexes() {
    conjugate::<f64>();
    conjugate::<Complex<f64>>();
}

#[test]
fn test_field_and_euclidean_domain_are_disjoint_here() {
    // Mathematically every field is a Euclidean domain. This tower reserves the rung for the
    // integers, so the two admit disjoint sets of concrete types and an operation wanted for both
    // is written at CommutativeRing or provided twice.
    field::<f64>();
    euclidean::<i64>();
    ring::<f64>();
    ring::<i64>();
}
