/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `AlgebraIso<S, T, R>` (witness-typed).

use deep_causality_num::iso::witness::test_support::assert_witness_algebra_iso_law;

use super::super::common::FloatWrap;
use super::common::{BadWitness, IdWitness};

#[test]
fn witness_algebra_iso_law_holds_for_id_witness() {
    assert_witness_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(2.5), 3.0);
    assert_witness_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(-1.0), 0.0);
    assert_witness_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(0.0), 7.0);
    assert_witness_algebra_iso_law::<IdWitness, FloatWrap, f64, f64>(FloatWrap(4.0), -2.0);
}

#[test]
#[should_panic(expected = "Witness AlgebraIso scalar-multiplication homomorphism failed")]
fn witness_algebra_iso_law_panics_on_broken_scalar_multiplication() {
    // BadWitness shifts by +1.0:
    //   to_target(a.scale(r)) = (a * r) + 1
    //   to_target(a).scale(r) = (a + 1) * r = ar + r
    // For (a=2, r=3): lhs = 7, rhs = 9; differ.
    assert_witness_algebra_iso_law::<BadWitness, FloatWrap, f64, f64>(FloatWrap(2.0), 3.0);
}
