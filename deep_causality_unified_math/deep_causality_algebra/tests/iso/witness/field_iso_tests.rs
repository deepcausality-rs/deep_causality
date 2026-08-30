/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `FieldIso<S, T>` (witness-typed).

use deep_causality_algebra::iso::witness::test_support::assert_witness_field_iso_laws;

use deep_causality_algebra::utils_tests::utils_iso_tests::FloatWrap;
use deep_causality_algebra::utils_tests::utils_iso_witness_tests::{BadWitness, IdWitness};

#[test]
fn witness_field_iso_laws_hold_for_id_witness() {
    assert_witness_field_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(2.5));
    assert_witness_field_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(-3.0));
    assert_witness_field_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(1.0));
    assert_witness_field_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(0.5));
}

#[test]
#[should_panic(expected = "Witness FieldIso multiplicative-inverse homomorphism failed")]
fn witness_field_iso_laws_panic_on_broken_inverse() {
    // BadWitness shifts by +1.0:
    //   to_target(a.inverse()) = 1/a + 1
    //   to_target(a).inverse() = 1/(a + 1)
    // These differ for any non-zero finite a.
    assert_witness_field_iso_laws::<BadWitness, FloatWrap, f64>(FloatWrap(2.0));
}
