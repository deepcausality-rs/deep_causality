/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `StandardIso<S, T>` — the generic zero-sized witness that
//! blanket-implements `Iso<S, T>` and every Tier 2 marker subtrait when the
//! underlying types satisfy bidirectional `From` plus the corresponding
//! algebraic-structure trait.

use deep_causality_num::iso::witness::test_support::{
    assert_witness_algebra_iso_law, assert_witness_division_algebra_iso_law,
    assert_witness_field_iso_laws, assert_witness_group_iso_law, assert_witness_iso_round_trip,
    assert_witness_ring_iso_laws,
};
use deep_causality_num::iso::witness::{Iso, StandardIso};

use super::super::common::FloatWrap;

// =============================================================================
// Constructor / trait derivations
// =============================================================================

#[test]
fn standard_iso_new_is_zero_sized() {
    let s: StandardIso<FloatWrap, f64> = StandardIso::new();
    assert_eq!(core::mem::size_of_val(&s), 0);
}

#[test]
fn standard_iso_default_constructs() {
    let _: StandardIso<FloatWrap, f64> = StandardIso::default();
}

#[test]
fn standard_iso_clone_and_copy() {
    let s: StandardIso<FloatWrap, f64> = StandardIso::new();
    let s2 = s; // exercises Copy
    // Explicitly call the Clone impl to verify it exists and behaves
    // identically to Copy. Use the trait-qualified form to avoid the
    // `clippy::clone_on_copy` lint that would fire on `s.clone()`.
    let s3 = Clone::clone(&s);
    let _ = (s2, s3);
}

#[test]
fn standard_iso_debug_format() {
    let s: StandardIso<FloatWrap, f64> = StandardIso::new();
    assert_eq!(format!("{:?}", s), "StandardIso");
}

// =============================================================================
// Blanket impl: Iso<S, T>
// =============================================================================

#[test]
fn standard_iso_blanket_iso_to_target() {
    // `StandardIso<FloatWrap, f64>` blanket-implements `Iso<FloatWrap, f64>`
    // because both directions of `From` exist on the FloatWrap type pair.
    assert_eq!(
        <StandardIso<FloatWrap, f64> as Iso<FloatWrap, f64>>::to_target(FloatWrap(2.5)),
        2.5
    );
    assert_eq!(
        <StandardIso<FloatWrap, f64> as Iso<FloatWrap, f64>>::to_target(FloatWrap(-1.0)),
        -1.0
    );
}

#[test]
fn standard_iso_blanket_iso_to_source() {
    assert_eq!(
        <StandardIso<FloatWrap, f64> as Iso<FloatWrap, f64>>::to_source(2.5),
        FloatWrap(2.5)
    );
    assert_eq!(
        <StandardIso<FloatWrap, f64> as Iso<FloatWrap, f64>>::to_source(0.0),
        FloatWrap(0.0)
    );
}

#[test]
fn standard_iso_round_trip_via_helper() {
    assert_witness_iso_round_trip::<StandardIso<FloatWrap, f64>, FloatWrap, f64>(FloatWrap(2.5));
    assert_witness_iso_round_trip::<StandardIso<FloatWrap, f64>, FloatWrap, f64>(FloatWrap(0.0));
    assert_witness_iso_round_trip::<StandardIso<FloatWrap, f64>, FloatWrap, f64>(FloatWrap(-3.7));
}

// =============================================================================
// Blanket impl: GroupIso<S, T>, RingIso<S, T>, FieldIso<S, T>,
//                AlgebraIso<S, T, R>, DivisionAlgebraIso<S, T, R>
// =============================================================================

#[test]
fn standard_iso_blanket_group_iso() {
    assert_witness_group_iso_law::<StandardIso<FloatWrap, f64>, FloatWrap, f64>(
        FloatWrap(2.5),
        FloatWrap(1.5),
    );
}

#[test]
fn standard_iso_blanket_ring_iso() {
    assert_witness_ring_iso_laws::<StandardIso<FloatWrap, f64>, FloatWrap, f64>(
        FloatWrap(3.0),
        FloatWrap(2.0),
    );
}

#[test]
fn standard_iso_blanket_field_iso() {
    assert_witness_field_iso_laws::<StandardIso<FloatWrap, f64>, FloatWrap, f64>(FloatWrap(2.5));
}

#[test]
fn standard_iso_blanket_algebra_iso() {
    assert_witness_algebra_iso_law::<StandardIso<FloatWrap, f64>, FloatWrap, f64, f64>(
        FloatWrap(4.0),
        2.5,
    );
}

#[test]
fn standard_iso_blanket_division_algebra_iso() {
    assert_witness_division_algebra_iso_law::<StandardIso<FloatWrap, f64>, FloatWrap, f64, f64>(
        FloatWrap(2.0),
    );
}

// =============================================================================
// Generic-bound usage demonstration
// =============================================================================

fn convert_through_witness<W>(s: FloatWrap) -> f64
where
    W: Iso<FloatWrap, f64>,
{
    W::to_target(s)
}

#[test]
fn standard_iso_satisfies_iso_bound_in_generic_context() {
    // Demonstrates that StandardIso can be used as a type-parameter implementer
    // in generic code that bounds on `W: Iso<S, T>`. The blanket fires and
    // monomorphization produces direct From::from calls.
    let result = convert_through_witness::<StandardIso<FloatWrap, f64>>(FloatWrap(7.5));
    assert_eq!(result, 7.5);
}
