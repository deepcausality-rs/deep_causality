/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `GroupIso<S, T>` (witness-typed) via the `IdWitness` and
//! `BadWitness` test types from `super::common`.

use deep_causality_num::iso::witness::test_support::assert_witness_group_iso_law;

use super::super::common::FloatWrap;
use super::common::{BadWitness, IdWitness};

#[test]
fn witness_group_iso_law_holds_for_id_witness() {
    assert_witness_group_iso_law::<IdWitness, FloatWrap, f64>(FloatWrap(3.0), FloatWrap(5.0));
    assert_witness_group_iso_law::<IdWitness, FloatWrap, f64>(FloatWrap(0.0), FloatWrap(0.0));
    assert_witness_group_iso_law::<IdWitness, FloatWrap, f64>(FloatWrap(-1.5), FloatWrap(2.5));
}

#[test]
#[should_panic(expected = "Witness GroupIso homomorphism failed")]
fn witness_group_iso_law_panics_on_broken_homomorphism() {
    // BadWitness::to_target shifts by +1.0:
    //   to_target(a + b) = a + b + 1
    //   to_target(a) + to_target(b) = (a + 1) + (b + 1) = a + b + 2
    // They differ, hitting the homomorphism-failure panic.
    assert_witness_group_iso_law::<BadWitness, FloatWrap, f64>(FloatWrap(3.0), FloatWrap(5.0));
}
