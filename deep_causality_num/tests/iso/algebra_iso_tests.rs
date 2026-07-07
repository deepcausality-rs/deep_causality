/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `deep_causality_num::iso::AlgebraIso<T, R>` and the helper
//! `assert_algebra_iso_from_law`.
//!
//! Uses the shared `FloatWrap(f64)` newtype from `deep_causality_num::utils_tests::utils_iso_tests`. `FloatWrap`
//! is an `Algebra<f64>` (Module<f64> + Mul<Output = Self> + One + Distributive).
//! The identity iso preserves scalar multiplication trivially.

use deep_causality_num::iso::test_support::assert_algebra_iso_from_law;

use deep_causality_num::utils_tests::utils_iso_tests::{BadFieldWrap, FloatWrap};

#[test]
fn algebra_iso_law_holds_for_identity_iso() {
    assert_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(2.5), 3.0);
    assert_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(-1.0), 0.0);
    assert_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(0.0), 7.0);
    assert_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(4.0), -2.0);
}

#[test]
fn algebra_iso_law_holds_in_reverse_direction() {
    assert_algebra_iso_from_law::<f64, FloatWrap, f64>(2.5, 3.0);
    assert_algebra_iso_from_law::<f64, FloatWrap, f64>(-1.5, 4.0);
}

#[test]
#[should_panic(expected = "AlgebraIso scalar-multiplication homomorphism failed")]
fn algebra_iso_law_panics_on_broken_scalar_multiplication() {
    // BadFieldWrap's From shifts by +1.0:
    //   f64::from(BadFieldWrap(a).scale(r)) = (a * r) + 1.0
    //   f64::from(BadFieldWrap(a)).scale(r) = (a + 1.0) * r
    // For (a=2, r=3): lhs = 7.0, rhs = 9.0 -- differ.
    assert_algebra_iso_from_law::<BadFieldWrap, f64, f64>(BadFieldWrap(2.0), 3.0);
}
