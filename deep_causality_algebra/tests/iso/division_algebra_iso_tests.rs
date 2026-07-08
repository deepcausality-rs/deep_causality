/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `deep_causality_algebra::iso::DivisionAlgebraIso<T, R>` and the
//! helper `assert_division_algebra_iso_from_law`.
//!
//! Uses the shared `FloatWrap(f64)` newtype from `deep_causality_algebra::utils_tests::utils_iso_tests`. For
//! real-valued division algebras the conjugate is the identity, so the
//! identity iso trivially preserves conjugation. `BadFieldWrap` claims
//! `conjugate = negation`, which breaks the law.

use deep_causality_algebra::iso::test_support::assert_division_algebra_iso_from_law;

use deep_causality_algebra::utils_tests::utils_iso_tests::{BadFieldWrap, FloatWrap};

#[test]
fn division_algebra_iso_law_holds_for_identity_iso() {
    assert_division_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(2.5));
    assert_division_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(0.0));
    assert_division_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(-3.7));
    assert_division_algebra_iso_from_law::<FloatWrap, f64, f64>(FloatWrap(1.0));
}

#[test]
fn division_algebra_iso_law_holds_in_reverse_direction() {
    assert_division_algebra_iso_from_law::<f64, FloatWrap, f64>(2.5);
    assert_division_algebra_iso_from_law::<f64, FloatWrap, f64>(-3.7);
}

#[test]
#[should_panic(expected = "DivisionAlgebraIso conjugation homomorphism failed")]
fn division_algebra_iso_law_panics_on_broken_conjugation() {
    // BadFieldWrap claims conjugate(BadFieldWrap(a)) == BadFieldWrap(-a),
    // but the From mapping is `+1.0` so:
    //   f64::from(BadFieldWrap(a).conjugate()) = f64::from(BadFieldWrap(-a)) = -a + 1.0
    //   f64::from(BadFieldWrap(a)).conjugate() = (a + 1.0).conjugate() = a + 1.0
    //     (for real f64, conjugate is identity in DivisionAlgebra<f64> blanket; here
    //      DivisionAlgebra is implemented on f64 in the algebra crate where conjugate
    //      is the identity)
    // The two differ for a != 0, hitting the conjugation-preservation panic.
    assert_division_algebra_iso_from_law::<BadFieldWrap, f64, f64>(BadFieldWrap(2.0));
}
