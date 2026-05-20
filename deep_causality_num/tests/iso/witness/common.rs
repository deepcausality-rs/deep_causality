/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared witness types for the Tier 2 witness-iso tests.
//!
//! These reuse the well-behaved `FloatWrap(f64)` and broken `BadFieldWrap(f64)`
//! newtypes already defined in `tests/iso/common.rs` (which satisfy
//! `Field + Algebra<f64> + DivisionAlgebra<f64>` and have bidirectional
//! `From` to/from `f64`).
//!
//! The Tier 2 witness tests use these as `S` and `T` directly. The witness
//! types we add here are:
//!
//! - `IdWitness` — a manual witness implementing every Tier 2 marker on the
//!   `(FloatWrap, f64)` pair. Demonstrates the witness-type pattern.
//! - `BadWitness` — a manual witness whose `to_target` shifts by `+1.0`,
//!   used by `#[should_panic]` tests to cover the assertion-failure branches
//!   of the Tier 2 helpers.

use deep_causality_num::iso::witness::Iso;

use super::super::common::FloatWrap;

/// Identity-iso witness for `(FloatWrap, f64)`. Forwards conversion through
/// `FloatWrap`'s bidirectional `From` impls. Used as the implementer for
/// every Tier 2 marker in the witness-tests, demonstrating that a single
/// witness can carry the whole marker stack.
pub(crate) struct IdWitness;

impl Iso<FloatWrap, f64> for IdWitness {
    fn to_target(s: FloatWrap) -> f64 {
        f64::from(s)
    }

    fn to_source(t: f64) -> FloatWrap {
        FloatWrap::from(t)
    }
}

// Marker impls below — verified by property tests in the corresponding files.
impl deep_causality_num::iso::witness::GroupIso<FloatWrap, f64> for IdWitness {}
impl deep_causality_num::iso::witness::RingIso<FloatWrap, f64> for IdWitness {}
impl deep_causality_num::iso::witness::FieldIso<FloatWrap, f64> for IdWitness {}
impl deep_causality_num::iso::witness::AlgebraIso<FloatWrap, f64, f64> for IdWitness {}
impl deep_causality_num::iso::witness::DivisionAlgebraIso<FloatWrap, f64, f64> for IdWitness {}

/// Broken-iso witness for `(FloatWrap, f64)`. `to_target` shifts by `+1.0`,
/// which breaks every homomorphism law (addition, multiplication, scalar
/// multiplication, inverse, conjugation). Used by `#[should_panic]` tests.
pub(crate) struct BadWitness;

impl Iso<FloatWrap, f64> for BadWitness {
    fn to_target(s: FloatWrap) -> f64 {
        s.0 + 1.0
    }

    fn to_source(t: f64) -> FloatWrap {
        FloatWrap(t - 1.0)
    }
}

impl deep_causality_num::iso::witness::GroupIso<FloatWrap, f64> for BadWitness {}
impl deep_causality_num::iso::witness::RingIso<FloatWrap, f64> for BadWitness {}
impl deep_causality_num::iso::witness::FieldIso<FloatWrap, f64> for BadWitness {}
impl deep_causality_num::iso::witness::AlgebraIso<FloatWrap, f64, f64> for BadWitness {}
impl deep_causality_num::iso::witness::DivisionAlgebraIso<FloatWrap, f64, f64> for BadWitness {}

/// Broken-only-on-T-to-S witness. `to_target` is correct; `to_source` always
/// returns `FloatWrap(0.0)`. Used to exercise the `S -> T -> S` branch of
/// `assert_witness_iso_round_trip`: for any non-zero `s`, `to_target(s)` is
/// a non-zero `t`, but `to_source(t) = FloatWrap(0.0) != s`.
pub(crate) struct BadReverseWitness;

impl Iso<FloatWrap, f64> for BadReverseWitness {
    fn to_target(s: FloatWrap) -> f64 {
        s.0
    }

    fn to_source(_: f64) -> FloatWrap {
        FloatWrap(0.0)
    }
}

/// `to_target` is identity; `to_source` collapses the sign via `abs`. The
/// `S -> T -> S` branch passes for every non-negative `s` (sign is never
/// destroyed in that direction), but the `T -> S -> T` branch fails for any
/// negative `t`: `to_source(-2.5) = FloatWrap(2.5)`, then
/// `to_target(FloatWrap(2.5)) = 2.5 != -2.5`. Used to pin the independent
/// `T -> S -> T` check in `assert_witness_iso_round_trip`.
pub(crate) struct AbsReverseWitness;

impl Iso<FloatWrap, f64> for AbsReverseWitness {
    fn to_target(s: FloatWrap) -> f64 {
        s.0
    }

    fn to_source(t: f64) -> FloatWrap {
        FloatWrap(t.abs())
    }
}

/// Preserves addition (linear scale by 2) but breaks multiplication: doubling
/// gives `2(a+b) = 2a+2b` (addition OK) but `2(a·b) ≠ (2a)·(2b) = 4ab`. Used to
/// hit the multiplication panic branch of `assert_witness_ring_iso_laws`
/// without first tripping its addition panic branch.
pub(crate) struct DoubleWitness;

impl Iso<FloatWrap, f64> for DoubleWitness {
    fn to_target(s: FloatWrap) -> f64 {
        s.0 * 2.0
    }

    fn to_source(t: f64) -> FloatWrap {
        FloatWrap(t / 2.0)
    }
}

impl deep_causality_num::iso::witness::GroupIso<FloatWrap, f64> for DoubleWitness {}
impl deep_causality_num::iso::witness::RingIso<FloatWrap, f64> for DoubleWitness {}

// =============================================================================
// Complex-based witnesses (for conjugation-panel coverage)
// =============================================================================
//
// On real-valued types (FloatWrap, f64) the conjugation law is trivially
// preserved by any iso because both sides' `conjugate` is the identity. To
// exercise the conjugation panic branch of
// `assert_witness_division_algebra_iso_law` we need a type pair with
// non-trivial conjugation. `Complex<f64>` is the simplest such type already
// in the codebase.

use deep_causality_num::Complex;

/// Identity iso on `Complex<f64>` — round-trip, group, ring, field, algebra,
/// and division-algebra laws all hold trivially.
pub(crate) struct ComplexIdWitness;

impl Iso<Complex<f64>, Complex<f64>> for ComplexIdWitness {
    fn to_target(s: Complex<f64>) -> Complex<f64> {
        s
    }
    fn to_source(t: Complex<f64>) -> Complex<f64> {
        t
    }
}

impl deep_causality_num::iso::witness::GroupIso<Complex<f64>, Complex<f64>> for ComplexIdWitness {}
impl deep_causality_num::iso::witness::AlgebraIso<Complex<f64>, Complex<f64>, f64>
    for ComplexIdWitness
{
}
impl deep_causality_num::iso::witness::DivisionAlgebraIso<Complex<f64>, Complex<f64>, f64>
    for ComplexIdWitness
{
}

/// Shifts the imaginary part by `+1.0`. Round-trip clean (paired with the
/// `-1.0` inverse on `to_source`); breaks conjugation preservation because
/// the shift is constant-affine while `conjugate` flips the sign of the
/// imaginary part. Used to hit the conjugation panic branch of
/// `assert_witness_division_algebra_iso_law`.
pub(crate) struct ComplexShiftImWitness;

impl Iso<Complex<f64>, Complex<f64>> for ComplexShiftImWitness {
    fn to_target(s: Complex<f64>) -> Complex<f64> {
        Complex::new(s.re, s.im + 1.0)
    }
    fn to_source(t: Complex<f64>) -> Complex<f64> {
        Complex::new(t.re, t.im - 1.0)
    }
}

impl deep_causality_num::iso::witness::GroupIso<Complex<f64>, Complex<f64>>
    for ComplexShiftImWitness
{
}
impl deep_causality_num::iso::witness::AlgebraIso<Complex<f64>, Complex<f64>, f64>
    for ComplexShiftImWitness
{
}
impl deep_causality_num::iso::witness::DivisionAlgebraIso<Complex<f64>, Complex<f64>, f64>
    for ComplexShiftImWitness
{
}
