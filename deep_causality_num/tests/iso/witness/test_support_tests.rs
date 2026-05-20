/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cross-cutting tests for the Tier 2 test_support helpers. The marker-trait
//! test files (`group_iso_tests.rs`, `ring_iso_tests.rs`, etc.) cover the
//! success and panic paths of each helper individually. This file pins
//! additional helper invariants that don't fit into a single marker file.

use deep_causality_num::iso::witness::test_support::{
    assert_witness_algebra_iso_law, assert_witness_division_algebra_iso_law,
    assert_witness_field_iso_laws, assert_witness_group_iso_law, assert_witness_iso_round_trip,
    assert_witness_ring_iso_laws,
};

use super::super::common::FloatWrap;
use super::common::IdWitness;

#[test]
fn helpers_accept_id_witness_across_full_marker_stack() {
    // One witness, every helper level. Verifies the witness can be passed
    // through the entire Tier 2 marker hierarchy without re-implementing
    // anything per level.
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(2.0), 2.0);
    assert_witness_group_iso_law::<IdWitness, FloatWrap, f64>(FloatWrap(2.0), FloatWrap(3.0));
    assert_witness_ring_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(2.0), FloatWrap(3.0));
    assert_witness_field_iso_laws::<IdWitness, FloatWrap, f64>(FloatWrap(2.0));
    assert_witness_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(2.0), 3.0);
    assert_witness_division_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(2.0));
}

#[test]
fn round_trip_helper_handles_zero_and_negative_inputs() {
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(0.0), 0.0);
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(-100.0), -100.0);
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(1e-9), 1e-9);
    assert_witness_iso_round_trip::<IdWitness, FloatWrap, f64>(FloatWrap(1e9), 1e9);
}
