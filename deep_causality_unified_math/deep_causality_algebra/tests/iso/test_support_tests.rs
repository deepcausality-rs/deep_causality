/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the base `assert_iso_from_round_trip` helper plus panic-path
//! coverage for all helpers in [`deep_causality_algebra::iso::test_support`].
//!
//! The marker-trait files (`group_iso_tests.rs`, `ring_iso_tests.rs`, etc.)
//! cover the success-path of their corresponding helper. This file covers:
//!
//! - The base round-trip helper, which is not algebraic-structure-aware and
//!   therefore not exercised in any of the marker-trait test files.
//! - The panic branches of every helper, via `#[should_panic]` tests on
//!   deliberately broken `From` pairs.

use deep_causality_algebra::iso::test_support::assert_iso_from_round_trip;

#[test]
fn round_trip_identity_i32_succeeds() {
    assert_iso_from_round_trip::<i32, i32>(42, 42);
    assert_iso_from_round_trip::<i32, i32>(0, 0);
    assert_iso_from_round_trip::<i32, i32>(-1, -1);
    assert_iso_from_round_trip::<i32, i32>(i32::MAX, i32::MAX);
    assert_iso_from_round_trip::<i32, i32>(i32::MIN, i32::MIN);
}

#[test]
fn round_trip_identity_f64_succeeds() {
    assert_iso_from_round_trip::<f64, f64>(1.5, 1.5);
    assert_iso_from_round_trip::<f64, f64>(0.0, 0.0);
    assert_iso_from_round_trip::<f64, f64>(-42.0, -42.0);
}

#[test]
fn round_trip_identity_u64_succeeds() {
    assert_iso_from_round_trip::<u64, u64>(0, 0);
    assert_iso_from_round_trip::<u64, u64>(1, 1);
    assert_iso_from_round_trip::<u64, u64>(u64::MAX, u64::MAX);
}

// A wrapper that breaks the round-trip in the T -> S direction.
// `S = i32`, `T = BadRoundTrip`. The forward conversion (i32 -> BadRoundTrip)
// stores the original value; the backward conversion (BadRoundTrip -> i32)
// always returns 0, so the round-trip i32 -> BadRoundTrip -> i32 fails for
// any non-zero input.
#[derive(Clone, PartialEq, Debug)]
struct BadRoundTrip(#[allow(dead_code)] i32);

impl From<i32> for BadRoundTrip {
    fn from(x: i32) -> Self {
        BadRoundTrip(x)
    }
}

impl From<BadRoundTrip> for i32 {
    fn from(_: BadRoundTrip) -> Self {
        0
    }
}

#[test]
#[should_panic(expected = "From round-trip S -> T -> S failed")]
fn round_trip_broken_panics_on_forward_direction() {
    // 42 -> BadRoundTrip(42) -> 0 != 42, so the S -> T -> S branch panics.
    assert_iso_from_round_trip::<i32, BadRoundTrip>(42, BadRoundTrip(0));
}

// A wrapper that breaks the round-trip in the S -> T direction.
// `S = u32`, `T = ZeroOnReverse`. Forward (u32 -> ZeroOnReverse) preserves
// data; backward (ZeroOnReverse -> u32) returns the inner value unchanged;
// but the symmetric From<u32> for ZeroOnReverse always wraps 0, so the
// T -> S -> T branch fails for any non-zero wrapped value.
#[derive(Clone, PartialEq, Debug)]
struct AsymmetricBackwards(u32);

impl From<u32> for AsymmetricBackwards {
    fn from(_: u32) -> Self {
        AsymmetricBackwards(0)
    }
}

impl From<AsymmetricBackwards> for u32 {
    fn from(w: AsymmetricBackwards) -> Self {
        w.0
    }
}

#[test]
#[should_panic(expected = "From round-trip")]
fn round_trip_broken_panics_when_forward_is_lossy() {
    // 7 -> AsymmetricBackwards(0) -> 0 != 7. Hits the S -> T -> S panic.
    assert_iso_from_round_trip::<u32, AsymmetricBackwards>(7, AsymmetricBackwards(0));
}

// A wrapper that passes the S -> T -> S round-trip for every valid `S`
// (`u32`) but fails the T -> S -> T round-trip for any negative `i32`
// wrapped in `ReverseLossy`. S::from collapses the sign via `unsigned_abs`,
// so two distinct `T` values (`ReverseLossy(5)` and `ReverseLossy(-5)`) map
// to the same `S = 5u32`, then back to `ReverseLossy(5)`, losing the sign.
// This pins the independent T -> S -> T check.
#[derive(Clone, PartialEq, Debug)]
struct ReverseLossy(i32);

impl From<u32> for ReverseLossy {
    fn from(x: u32) -> Self {
        ReverseLossy(x as i32)
    }
}

impl From<ReverseLossy> for u32 {
    fn from(w: ReverseLossy) -> Self {
        w.0.unsigned_abs()
    }
}

#[test]
#[should_panic(expected = "From round-trip T -> S -> T failed")]
fn round_trip_panics_independently_on_reverse_direction() {
    // S -> T -> S: u32::from(ReverseLossy::from(7u32)) = 7 ✓
    // T -> S -> T: ReverseLossy::from(u32::from(ReverseLossy(-5)))
    //            = ReverseLossy::from(5u32) = ReverseLossy(5) != ReverseLossy(-5)
    assert_iso_from_round_trip::<u32, ReverseLossy>(7, ReverseLossy(-5));
}
