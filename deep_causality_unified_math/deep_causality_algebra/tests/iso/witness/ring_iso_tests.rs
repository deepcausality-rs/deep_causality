/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `RingIso<S, T>` (witness-typed). Exercises both the addition and
//! multiplication homomorphism branches.

use deep_causality_algebra::iso::witness::test_support::assert_witness_ring_iso_laws;

use deep_causality_algebra::utils_tests::utils_iso_tests::FloatWrap;
use deep_causality_algebra::utils_tests::utils_iso_witness_tests::{
    BadWitness, DoubleWitness, IdWitness,
};

#[test]
fn witness_ring_iso_laws_hold_for_id_witness() {
    assert_witness_ring_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(3.0), FloatWrap(5.0));
    assert_witness_ring_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(0.0), FloatWrap(0.0));
    assert_witness_ring_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(-1.5), FloatWrap(2.5));
}

#[test]
#[should_panic(expected = "Witness RingIso addition homomorphism failed")]
fn witness_ring_iso_laws_panic_on_broken_addition() {
    // BadWitness::to_target shifts by +1.0, which breaks addition first:
    //   to_target(a + b) = a + b + 1
    //   to_target(a) + to_target(b) = (a + 1) + (b + 1) = a + b + 2
    // The addition check fires first and panics.
    assert_witness_ring_iso_laws::<BadWitness, FloatWrap, f64>(FloatWrap(3.0), FloatWrap(5.0));
}

#[test]
#[should_panic(expected = "Witness RingIso multiplication homomorphism failed")]
fn witness_ring_iso_laws_panic_on_broken_multiplication() {
    // DoubleWitness preserves addition (2(a+b) = 2a + 2b) but breaks
    // multiplication: 2(a*b) ≠ (2a)*(2b) = 4ab. The addition assertion
    // passes; the multiplication assertion panics. Use non-trivial values
    // so a*b > 0 and the doubling break surfaces.
    assert_witness_ring_iso_laws::<DoubleWitness, FloatWrap, f64>(FloatWrap(3.0), FloatWrap(5.0));
}
