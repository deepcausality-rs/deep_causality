/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algebra::iso::witness::iso::Iso;
use deep_causality_algebra::{
    Bijective, Hom, Injective, IsoBackward, IsoForward, Isomorphism, Surjective,
};

/// A witness for the isomorphism ℤ ≅ ℤ given by negation — its own inverse.
struct NegateWitness;
impl Iso<i64, i64> for NegateWitness {
    fn to_target(s: i64) -> i64 {
        -s
    }
    fn to_source(t: i64) -> i64 {
        -t
    }
}

/// A witness with genuinely different ends: i32 ≅ its widening image in i64.
struct WidenWitness;
impl Iso<i32, i64> for WidenWitness {
    fn to_target(s: i32) -> i64 {
        i64::from(s)
    }
    fn to_source(t: i64) -> i32 {
        t as i32
    }
}

fn assert_bijective<H: Bijective>() {}
fn assert_injective<H: Injective>() {}
fn assert_surjective<H: Surjective>() {}

#[test]
fn test_forward_view_is_a_hom() {
    let f = IsoForward::<NegateWitness, i64, i64>::new();
    assert_eq!(f.apply(7), -7);
    assert_eq!(f.apply(-3), 3);
}

#[test]
fn test_backward_view_is_a_hom() {
    let b = IsoBackward::<NegateWitness, i64, i64>::new();
    assert_eq!(b.apply(7), -7);
}

#[test]
fn test_iso_views_are_bijective() {
    assert_bijective::<IsoForward<NegateWitness, i64, i64>>();
    assert_bijective::<IsoBackward<NegateWitness, i64, i64>>();
    assert_injective::<IsoForward<NegateWitness, i64, i64>>();
    assert_surjective::<IsoForward<NegateWitness, i64, i64>>();
}

#[test]
fn test_ends_are_named_and_swap_in_the_inverse() {
    // The forward view goes i32 -> i64; its inverse goes i64 -> i32. That the ends swap is the
    // statement that only became sayable once maps carried named ends.
    let f = IsoForward::<WidenWitness, i32, i64>::new();
    assert_eq!(f.apply(5_i32), 5_i64);

    let inv = f.inverse();
    assert_eq!(inv.apply(5_i64), 5_i32);
}

#[test]
fn test_round_trip_through_the_inverse_is_the_identity() {
    let f = IsoForward::<NegateWitness, i64, i64>::new();
    let inv = f.inverse();
    for x in [-9_i64, 0, 1, 42] {
        assert_eq!(inv.apply(f.apply(x)), x);
    }
}

#[test]
fn test_inverse_of_the_inverse_is_the_original_direction() {
    let f = IsoForward::<WidenWitness, i32, i64>::new();
    let back = f.inverse();
    let fwd_again = back.inverse();
    assert_eq!(fwd_again.apply(11_i32), 11_i64);
}

#[test]
fn test_new_and_default_agree() {
    assert_eq!(
        IsoForward::<NegateWitness, i64, i64>::new(),
        IsoForward::<NegateWitness, i64, i64>::default()
    );
    assert_eq!(
        IsoBackward::<NegateWitness, i64, i64>::new(),
        IsoBackward::<NegateWitness, i64, i64>::default()
    );
}

#[test]
fn test_compile_time_assertion_helper() {
    deep_causality_algebra::hom::iso_bridge::assert_iso_is_bijective_hom::<NegateWitness, i64, i64>(
    );
}

// ---------------------------------------------------------------------------
// The hand-written marker impls. They exist so a view does not demand `Debug`,
// `Default` or `Eq` of its phantom parameters — `NegateWitness` implements none
// of them, so these tests would not compile if the derives were still in place.
// ---------------------------------------------------------------------------

#[test]
fn test_debug_does_not_require_the_witness_to_be_debug() {
    let f = IsoForward::<NegateWitness, i64, i64>::new();
    let b = IsoBackward::<NegateWitness, i64, i64>::new();
    assert_eq!(format!("{f:?}"), "IsoForward");
    assert_eq!(format!("{b:?}"), "IsoBackward");
}

#[test]
fn test_clone_and_copy() {
    let f = IsoForward::<NegateWitness, i64, i64>::new();
    #[allow(clippy::clone_on_copy)]
    let cloned = f.clone();
    let copied = f;
    assert_eq!(cloned.apply(4), -4);
    assert_eq!(copied.apply(4), -4);
    // the original is still usable, so it really is `Copy`
    assert_eq!(f.apply(4), -4);

    let b = IsoBackward::<NegateWitness, i64, i64>::new();
    #[allow(clippy::clone_on_copy)]
    let b_cloned = b.clone();
    assert_eq!(b_cloned.apply(4), -4);
    assert_eq!(b.apply(4), -4);
}

#[test]
fn test_equality_is_trivial_for_a_zero_sized_view() {
    // Two views of the same witness carry no data, so they are equal.
    assert_eq!(
        IsoForward::<NegateWitness, i64, i64>::new(),
        IsoForward::<NegateWitness, i64, i64>::new()
    );
    assert_eq!(
        IsoBackward::<NegateWitness, i64, i64>::new(),
        IsoBackward::<NegateWitness, i64, i64>::new()
    );
}
