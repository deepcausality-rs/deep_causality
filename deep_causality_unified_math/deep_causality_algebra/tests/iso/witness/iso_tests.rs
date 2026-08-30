/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the base `Iso<S, T>` witness-typed trait and its `to_target` /
//! `to_source` methods.

use deep_causality_algebra::iso::witness::Iso;
use deep_causality_algebra::iso::witness::test_support::assert_witness_iso_round_trip;

use deep_causality_algebra::utils_tests::utils_iso_tests::FloatWrap;
use deep_causality_algebra::utils_tests::utils_iso_witness_tests::{
    AbsReverseWitness, BadReverseWitness, BadWitness, IdWitness,
};
// BadWitness imported for the `round_trips_cleanly_despite_homomorphism_break`
// test below; not consumed in the round-trip-panic test (BadWitness has a
// symmetric +1/-1 pair that round-trips cleanly).

#[test]
fn iso_to_target_calls_correctly() {
    assert_eq!(
        <IdWitness as Iso<FloatWrap, f64>>::to_target(FloatWrap(2.5)),
        2.5
    );
    assert_eq!(
        <IdWitness as Iso<FloatWrap, f64>>::to_target(FloatWrap(0.0)),
        0.0
    );
    assert_eq!(
        <IdWitness as Iso<FloatWrap, f64>>::to_target(FloatWrap(-3.7)),
        -3.7
    );
}

#[test]
fn iso_to_source_calls_correctly() {
    assert_eq!(
        <IdWitness as Iso<FloatWrap, f64>>::to_source(2.5),
        FloatWrap(2.5)
    );
    assert_eq!(
        <IdWitness as Iso<FloatWrap, f64>>::to_source(0.0),
        FloatWrap(0.0)
    );
}

#[test]
fn iso_round_trip_holds_for_id_witness() {
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(2.5), 2.5);
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(0.0), 0.0);
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(-3.7), -3.7);
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(1.0), 1.0);
}

#[test]
#[should_panic(expected = "Witness iso round-trip S -> T -> S failed")]
fn iso_round_trip_panics_when_to_source_is_broken() {
    // BadReverseWitness::to_source always returns FloatWrap(0.0):
    //   to_target(FloatWrap(2.5)) = 2.5
    //   to_source(2.5) = FloatWrap(0.0) != FloatWrap(2.5)
    // The S -> T -> S branch panics.
    assert_witness_iso_round_trip::<BadReverseWitness, FloatWrap, f64>(FloatWrap(2.5), 2.5);
}

#[test]
#[should_panic(expected = "Witness iso round-trip T -> S -> T failed")]
fn iso_round_trip_panics_independently_on_t_to_s_to_t() {
    // AbsReverseWitness: to_target is identity, to_source collapses sign
    // via abs. For s = FloatWrap(2.5):
    //   S -> T -> S: to_source(to_target(2.5)) = FloatWrap(2.5) ✓
    // For t = -2.5:
    //   T -> S -> T: to_target(to_source(-2.5)) = to_target(FloatWrap(2.5))
    //              = 2.5 != -2.5
    // The T -> S -> T branch panics — this could not be caught if `t` were
    // derived from `s` (since to_target(s) is always non-negative for
    // non-negative s).
    assert_witness_iso_round_trip::<AbsReverseWitness, FloatWrap, f64>(FloatWrap(2.5), -2.5);
}

#[test]
fn bad_witness_round_trips_cleanly_despite_homomorphism_break() {
    // BadWitness shifts by +1.0 on both directions symmetrically, so it
    // does pass the round-trip law (but fails homomorphism laws — see other
    // test files). This test pins that fact.
    assert_witness_iso_round_trip::<BadWitness, FloatWrap, f64>(FloatWrap(2.5), 2.5);
    assert_witness_iso_round_trip::<BadWitness, FloatWrap, f64>(FloatWrap(-1.0), -1.0);
}
