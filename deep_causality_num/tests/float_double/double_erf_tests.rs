/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `erf` / `erfc` tests for `Float106`.
//!
//! Reference values were computed with mpmath at 60 decimal digits and split into
//! double-double `(hi, lo)` components.

use deep_causality_num::{Float, Float106};

fn d(x: f64) -> Float106 {
    Float106::from_f64(x)
}

/// Builds a reference `Float106` from its mpmath-derived `(hi, lo)` split.
fn r(hi: f64, lo: f64) -> Float106 {
    Float106::new(hi, lo)
}

/// Asserts `got` matches `want` to a relative tolerance `rel`.
fn assert_close(got: Float106, want: Float106, rel: f64) {
    let diff = <Float106 as Float>::abs(got - want).hi();
    let scale = <Float106 as Float>::abs(want).hi().max(f64::MIN_POSITIVE);
    assert!(
        diff <= rel * scale,
        "relative error {:e} exceeds {:e} (got hi={:e}, want hi={:e})",
        diff / scale,
        rel,
        got.hi(),
        want.hi()
    );
}

// =============================================================================
// erf — reference values (body, |x| < 1.5)
// =============================================================================

#[test]
fn test_erf_small_reference_values() {
    // Inputs are exactly representable in f64 so the reference (computed at the exact
    // decimal) matches `erf(x_f64)`; 0.1 would not be and is deliberately avoided.
    assert_close(
        d(0.25).erf(),
        r(0.27632639016823696, -2.4227076221184163e-17),
        1e-28,
    );
    assert_close(
        d(0.5).erf(),
        r(0.5204998778130465, 1.900077467916287e-17),
        1e-28,
    );
    assert_close(
        d(1.0).erf(),
        r(0.8427007929497149, -2.4801011789118602e-17),
        1e-28,
    );
}

// =============================================================================
// erf — reference values (tail, |x| >= 1.5, computed as 1 - erfc)
// =============================================================================

#[test]
fn test_erf_tail_reference_values() {
    assert_close(
        d(1.5).erf(),
        r(0.9661051464753108, -3.3867031441680696e-17),
        1e-28,
    );
    assert_close(
        d(2.0).erf(),
        r(0.9953222650189527, 2.20719858329765e-17),
        1e-28,
    );
    assert_close(
        d(3.0).erf(),
        r(0.9999779095030014, 5.363397058636269e-17),
        1e-28,
    );
}

// =============================================================================
// erfc — reference values across body and tail
// =============================================================================

#[test]
fn test_erfc_reference_values() {
    assert_close(
        d(0.5).erfc(),
        r(0.4795001221869535, -1.900077467916287e-17),
        1e-28,
    );
    assert_close(
        d(1.5).erfc(),
        r(0.033894853524689274, -8.274380778554473e-19),
        1e-28,
    );
    assert_close(
        d(2.0).erfc(),
        r(0.004677734981047266, -3.8794238326641256e-19),
        1e-28,
    );
}

#[test]
fn test_erfc_tail_no_cancellation() {
    // erfc(6) ≈ 2.15e-17: `1 - erf(6)` would lose every significant digit in f64,
    // so a correct value here proves the tail is computed directly.
    assert_close(
        d(6.0).erfc(),
        r(2.1519736712498913e-17, 3.1898197253599377e-34),
        1e-24,
    );
    // Deep tail.
    assert_close(
        d(10.0).erfc(),
        r(2.088487583762545e-45, -1.2006565763501381e-61),
        1e-24,
    );
}

// =============================================================================
// Identities and edge cases
// =============================================================================

#[test]
fn test_erf_zero_and_erfc_zero() {
    assert_eq!(d(0.0).erf(), d(0.0));
    assert_eq!(d(0.0).erfc(), d(1.0));
}

#[test]
fn test_erf_is_odd_exactly() {
    for &x in &[0.3, 0.9, 1.7, 4.0] {
        assert_eq!(d(-x).erf(), -d(x).erf());
    }
}

#[test]
fn test_erf_plus_erfc_is_one() {
    for &x in &[-4.0, -1.2, -0.4, 0.0, 0.4, 1.2, 2.5, 6.0] {
        let sum = d(x).erf() + d(x).erfc();
        assert_close(sum, d(1.0), 1e-29);
    }
}

#[test]
fn test_erf_bounds_and_monotonicity() {
    let xs = [0.0, 0.25, 0.5, 1.0, 1.5, 2.0, 3.0, 5.0];
    let mut prev = d(-1.0).erf();
    for &x in &xs {
        let e = d(x).erf();
        assert!(e.hi() >= -1.0 && e.hi() <= 1.0);
        assert!(e.hi() >= prev.hi(), "erf not monotone at x={}", x);
        prev = e;
    }
}

#[test]
fn test_erfc_negative_argument_near_two() {
    // erfc(-x) = 2 - erfc(x); for large x this is 2 minus a tiny number.
    let got = d(-6.0).erfc();
    let want = d(2.0) - r(2.1519736712498913e-17, 3.1898197253599377e-34);
    assert_close(got, want, 1e-28);
}

#[test]
fn test_erf_erfc_nan_propagates() {
    assert!(Float106::nan().erf().is_nan());
    assert!(Float106::nan().erfc().is_nan());
}
