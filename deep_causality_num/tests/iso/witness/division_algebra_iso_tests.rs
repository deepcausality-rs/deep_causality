/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `DivisionAlgebraIso<S, T, R>` (witness-typed).
//!
//! For real-valued source/target types (`FloatWrap`, `f64`) the conjugation
//! law is trivially satisfied by any iso because both sides' `conjugate` is
//! the identity function. The success-path tests verify the helper still
//! agrees on the real-valued case. The conjugation panic branch is exercised
//! using `Complex<f64>` as both source and target with a witness whose
//! `to_target` is an imaginary-shift that doesn't commute with complex
//! conjugation.

use deep_causality_num::Complex;
use deep_causality_num::iso::witness::test_support::assert_witness_division_algebra_iso_law;

use deep_causality_num::utils_tests::utils_iso_tests::FloatWrap;
use deep_causality_num::utils_tests::utils_iso_witness_tests::{
    ComplexIdWitness, ComplexShiftImWitness, IdWitness,
};

#[test]
fn witness_division_algebra_iso_law_holds_for_id_witness_real() {
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(2.5));
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(0.0));
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(-3.7));
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(1.0));
}

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
