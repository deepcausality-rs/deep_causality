/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Conjugation-branch coverage for `DivisionAlgebraIso<S, T, R>` (witness-typed).
//!
//! On real-valued types the conjugation law is trivially preserved because both sides' `conjugate`
//! is the identity. Exercising the conjugation panic branch of the law helper needs a type with
//! non-trivial conjugation, so it lives here with `Complex<f64>`.

use deep_causality_algebra::iso::witness::test_support::assert_witness_division_algebra_iso_law;
use deep_causality_algebra::iso::witness::{AlgebraIso, DivisionAlgebraIso, GroupIso, Iso};
use deep_causality_num_complex::Complex;

/// Identity iso on `Complex<f64>` — round-trip, group, algebra, and division-algebra laws all hold.
struct ComplexIdWitness;

impl Iso<Complex<f64>, Complex<f64>> for ComplexIdWitness {
    fn to_target(s: Complex<f64>) -> Complex<f64> {
        s
    }
    fn to_source(t: Complex<f64>) -> Complex<f64> {
        t
    }
}

impl GroupIso<Complex<f64>, Complex<f64>> for ComplexIdWitness {}
impl AlgebraIso<Complex<f64>, Complex<f64>, f64> for ComplexIdWitness {}
impl DivisionAlgebraIso<Complex<f64>, Complex<f64>, f64> for ComplexIdWitness {}

/// Shifts the imaginary part by `+1.0`. Round-trip clean (paired with the `-1.0` inverse on
/// `to_source`); breaks conjugation preservation because the shift is constant-affine while
/// `conjugate` flips the sign of the imaginary part.
struct ComplexShiftImWitness;

impl Iso<Complex<f64>, Complex<f64>> for ComplexShiftImWitness {
    fn to_target(s: Complex<f64>) -> Complex<f64> {
        Complex::new(s.re, s.im + 1.0)
    }
    fn to_source(t: Complex<f64>) -> Complex<f64> {
        Complex::new(t.re, t.im - 1.0)
    }
}

impl GroupIso<Complex<f64>, Complex<f64>> for ComplexShiftImWitness {}
impl AlgebraIso<Complex<f64>, Complex<f64>, f64> for ComplexShiftImWitness {}
impl DivisionAlgebraIso<Complex<f64>, Complex<f64>, f64> for ComplexShiftImWitness {}

#[test]
fn witness_division_algebra_iso_law_holds_for_id_witness_complex() {
    let z = Complex::new(2.0_f64, 3.0_f64);
    assert_witness_division_algebra_iso_law::<ComplexIdWitness, Complex<f64>, Complex<f64>, f64>(z);
    let z0 = Complex::new(0.0_f64, 0.0_f64);
    assert_witness_division_algebra_iso_law::<ComplexIdWitness, Complex<f64>, Complex<f64>, f64>(
        z0,
    );
    let z_neg = Complex::new(-1.5_f64, 4.0_f64);
    assert_witness_division_algebra_iso_law::<ComplexIdWitness, Complex<f64>, Complex<f64>, f64>(
        z_neg,
    );
}

#[test]
#[should_panic(expected = "Witness DivisionAlgebraIso conjugation homomorphism failed")]
fn witness_division_algebra_iso_law_panics_on_broken_conjugation() {
    // ComplexShiftImWitness::to_target(s) = s + i  (constant imaginary shift).
    // For a = 2 + 3i:
    //   a.conjugate() = 2 - 3i
    //   to_target(2 - 3i) = 2 - 3i + i = 2 - 2i
    //   to_target(a) = 2 + 3i + i = 2 + 4i
    //   to_target(a).conjugate() = 2 - 4i
    //   (2 - 2i) ≠ (2 - 4i) → panic.
    let z = Complex::new(2.0_f64, 3.0_f64);
    assert_witness_division_algebra_iso_law::<ComplexShiftImWitness, Complex<f64>, Complex<f64>, f64>(
        z,
    );
}
