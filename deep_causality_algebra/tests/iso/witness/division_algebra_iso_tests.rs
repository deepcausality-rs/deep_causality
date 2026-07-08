/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `DivisionAlgebraIso<S, T, R>` (witness-typed).
//!
//! For real-valued source/target types (`FloatWrap`, `f64`) the conjugation
//! law is trivially satisfied by any iso because both sides' `conjugate` is
//! the identity function. The success-path test verifies the helper agrees on
//! the real-valued case. The conjugation panic branch requires a type with
//! non-trivial conjugation (`Complex`) and is exercised in
//! `deep_causality_num_complex`.

use deep_causality_algebra::iso::witness::test_support::assert_witness_division_algebra_iso_law;
use deep_causality_algebra::utils_tests::utils_iso_tests::FloatWrap;
use deep_causality_algebra::utils_tests::utils_iso_witness_tests::IdWitness;

#[test]
fn witness_division_algebra_iso_law_holds_for_id_witness_real() {
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(2.5));
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(0.0));
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(-3.7));
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(1.0));
}
