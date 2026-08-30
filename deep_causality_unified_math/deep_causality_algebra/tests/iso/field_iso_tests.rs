/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `deep_causality_algebra::iso::FieldIso<T>` and the helper
//! `assert_field_iso_from_laws`.
//!
//! Uses the shared `FloatWrap(f64)` newtype from `deep_causality_algebra::utils_tests::utils_iso_tests`. The
//! identity iso preserves multiplicative inverses; the broken `BadFieldWrap`
//! variant breaks the inverse law because the shifted `From` mapping doesn't
//! commute with `1.0 / x`.

use deep_causality_algebra::iso::test_support::assert_field_iso_from_laws;

use deep_causality_algebra::utils_tests::utils_iso_tests::{BadFieldWrap, FloatWrap};

// Tier-1 inheritance chain: each marker is implemented separately.

#[test]
fn field_iso_laws_hold_for_identity_iso() {
    assert_field_iso_from_laws::<FloatWrap, f64>(FloatWrap(2.5));
    assert_field_iso_from_laws::<FloatWrap, f64>(FloatWrap(-3.0));
    assert_field_iso_from_laws::<FloatWrap, f64>(FloatWrap(1.0));
    assert_field_iso_from_laws::<FloatWrap, f64>(FloatWrap(0.5));
}

#[test]
fn field_iso_laws_hold_in_reverse_direction() {
    assert_field_iso_from_laws::<f64, FloatWrap>(2.5);
    assert_field_iso_from_laws::<f64, FloatWrap>(-3.0);
}

#[test]
#[should_panic(expected = "FieldIso multiplicative-inverse homomorphism failed")]
fn field_iso_laws_panic_on_broken_inverse() {
    // BadFieldWrap's From shifts by +1.0:
    //   f64::from(BadFieldWrap(a).inverse()) = (1/a) + 1.0
    //   f64::from(BadFieldWrap(a)).inverse() = 1 / (a + 1.0)
    // These differ for any finite non-zero a, so the inverse-preservation
    // assertion fails.
    assert_field_iso_from_laws::<BadFieldWrap, f64>(BadFieldWrap(2.0));
}
