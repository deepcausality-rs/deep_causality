/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the inverse-CDF (quantile) transforms.
//!
//! Standard-normal references were computed with mpmath at 60 decimal digits
//! (`q(u) = √2 · erfinv(2u − 1)`) and split into double-double `(hi, lo)` components.

use deep_causality_num::{Float, Float106};
use deep_causality_stats::{
    bernoulli_inverse_cdf, standard_normal_inverse_cdf, standard_normal_inverse_cdf_at,
    standard_normal_inverse_cdf_f106, uniform_inverse_cdf,
};

fn d(x: f64) -> Float106 {
    Float106::from(x)
}

/// Standard-normal CDF `Φ(x) = ½·erfc(−x/√2)` at double-double precision (test oracle).
fn phi(x: Float106) -> Float106 {
    d(0.5) * (-x / d(2.0).sqrt()).erfc()
}

fn close_dd(got: Float106, want: Float106, rel: f64) {
    let diff = <Float106 as Float>::abs(got - want).hi();
    let scale = <Float106 as Float>::abs(want).hi().max(1e-300);
    assert!(
        diff <= rel * scale,
        "rel err {:e} > {:e} (got {:e}, want {:e})",
        diff / scale,
        rel,
        got.hi(),
        want.hi()
    );
}

// =============================================================================
// Standard-normal quantile — f64
// =============================================================================

#[test]
fn test_standard_normal_quantile_f64_reference() {
    assert!((standard_normal_inverse_cdf(0.5)).abs() < 1e-15);
    assert!((standard_normal_inverse_cdf(0.25) - (-0.6744897501960817)).abs() < 1e-14);
    assert!((standard_normal_inverse_cdf(0.9375) - 1.5341205443525463).abs() < 1e-13);
}

#[test]
fn test_standard_normal_quantile_f64_is_antisymmetric() {
    for &u in &[0.25, 0.0625, 0.0009765625] {
        let lo = standard_normal_inverse_cdf(u);
        let hi = standard_normal_inverse_cdf(1.0 - u);
        assert!((lo + hi).abs() < 1e-12);
    }
}

#[test]
fn test_standard_normal_quantile_f64_finite_at_endpoints() {
    assert!(standard_normal_inverse_cdf(0.0).is_finite());
    assert!(standard_normal_inverse_cdf(1.0).is_finite());
}

// =============================================================================
// Standard-normal quantile — Float106
// =============================================================================

#[test]
fn test_standard_normal_quantile_f106_reference() {
    close_dd(
        standard_normal_inverse_cdf_f106(d(0.25)),
        Float106::new(-0.6744897501960817, -3.7755511355050287e-17),
        1e-28,
    );
    close_dd(
        standard_normal_inverse_cdf_f106(d(0.0625)),
        Float106::new(-1.5341205443525463, -1.1348835811852896e-17),
        1e-28,
    );
    close_dd(
        standard_normal_inverse_cdf_f106(d(0.0009765625)),
        Float106::new(-3.0972690781987846, 1.463371683801674e-16),
        1e-28,
    );
}

#[test]
fn test_standard_normal_quantile_f106_roundtrips_through_cdf() {
    for &u in &[0.25, 0.5, 0.75, 0.0625, 0.9375] {
        let x = standard_normal_inverse_cdf_f106(d(u));
        close_dd(phi(x), d(u), 1e-28);
    }
}

#[test]
fn test_f106_refinement_beats_f64_widening() {
    // The Float106 quantile must satisfy Φ(x)=u far more tightly than the widened f64 quantile.
    let u = d(0.25);
    let x_f106 = standard_normal_inverse_cdf_f106(u);
    let x_widened = d(standard_normal_inverse_cdf(0.25));
    let res_f106 = <Float106 as Float>::abs(phi(x_f106) - u).hi();
    let res_widened = <Float106 as Float>::abs(phi(x_widened) - u).hi();
    assert!(
        res_f106 < res_widened,
        "f106 residual {:e} not below widened {:e}",
        res_f106,
        res_widened
    );
}

#[test]
fn test_standard_normal_quantile_f106_monotone() {
    let mut prev = standard_normal_inverse_cdf_f106(d(0.001));
    for &u in &[0.1, 0.3, 0.5, 0.7, 0.9, 0.999] {
        let x = standard_normal_inverse_cdf_f106(d(u));
        assert!(x.hi() >= prev.hi(), "not monotone at u={}", u);
        prev = x;
    }
}

// =============================================================================
// Uniform quantile
// =============================================================================

#[test]
fn test_uniform_quantile_f64_exact() {
    assert_eq!(uniform_inverse_cdf(0.0_f64, 2.0, 10.0), 2.0);
    assert_eq!(uniform_inverse_cdf(0.5_f64, 2.0, 10.0), 6.0);
}

#[test]
fn test_uniform_quantile_f106_keeps_low_limb() {
    // 0.5·π carries a non-zero low limb; an f64 round-trip would discard it.
    let got = uniform_inverse_cdf(d(0.5), d(0.0), Float106::PI);
    close_dd(got, Float106::FRAC_PI_2, 1e-30);
    assert_ne!(got.lo(), 0.0);
}

// =============================================================================
// Bernoulli quantile
// =============================================================================

#[test]
fn test_bernoulli_quantile_thresholds_on_p() {
    assert!(bernoulli_inverse_cdf(0.29, 0.3));
    assert!(!bernoulli_inverse_cdf(0.30, 0.3));
    assert!(!bernoulli_inverse_cdf(0.99, 0.3));
    assert!(bernoulli_inverse_cdf(0.0, 0.3));
}

// -------------------------------------------------------------------------------------------
// The scalar-generic entry point.
//
// It exists so that code generic in its scalar has a quantile to call; the two per-precision
// functions above stay, and the property that makes that safe is that the generic one agrees with
// each of them exactly at its own scalar. Agreement is asserted bit for bit rather than to a
// tolerance, because anything looser would not distinguish "the same transform" from "a transform
// that happens to be close".
// -------------------------------------------------------------------------------------------

/// Coordinates that between them cover what a caller can hand this function: a uniform sweep, the
/// dyadic lattice a Sobol sequence actually produces, both clamp boundaries and either side of
/// them, the denormal floor, and the exact endpoints.
fn generic_probe_coordinates() -> Vec<f64> {
    let mut us = Vec::new();
    // A uniform sweep across the open interval.
    for i in 1..2000 {
        us.push(i as f64 / 2000.0);
    }
    // The dyadic lattice: a Sobol coordinate is k/2^32, so probe that grid at both ends.
    for k in 1..500u64 {
        us.push(k as f64 / 4_294_967_296.0);
        us.push(1.0 - k as f64 / 4_294_967_296.0);
    }
    // The clamp boundaries and their neighbours, the denormal floor, and the endpoints.
    us.extend_from_slice(&[
        0.0,
        f64::MIN_POSITIVE,
        5e-324,
        1e-300,
        1e-300 * (1.0 + f64::EPSILON),
        0.5,
        1.0 - f64::EPSILON,
        1.0 - f64::EPSILON / 2.0,
        1.0,
    ]);
    us
}

/// The generic form reproduces the `f64` form exactly, at every coordinate.
///
/// Both compute through the same `Float106` refinement; the generic one then takes the two limbs
/// into `R`, and at `R = f64` the low limb sits below the high limb's last place, so the sum is
/// the high limb. That is an argument, and this is the measurement.
#[test]
fn test_generic_quantile_reproduces_the_f64_form_bit_for_bit() {
    for u in generic_probe_coordinates() {
        let generic: f64 = standard_normal_inverse_cdf_at(u);
        let direct = standard_normal_inverse_cdf(u);
        assert_eq!(
            generic.to_bits(),
            direct.to_bits(),
            "generic and f64 quantiles differ at u = {u:e}: {generic} against {direct}"
        );
    }
}

/// The generic form reproduces the `Float106` form exactly, at every coordinate.
///
/// This is the half that could plausibly fail: the limbs are taken apart and put back together, so
/// a double-double whose reassembly is not the identity would show here. It is the identity
/// because every `Float106` operation ends in a normalising `quick_two_sum`, which leaves
/// `fl(hi + lo) == hi`.
#[test]
fn test_generic_quantile_reproduces_the_f106_form_bit_for_bit() {
    for u in generic_probe_coordinates() {
        let generic: Float106 = standard_normal_inverse_cdf_at(u);
        let direct = standard_normal_inverse_cdf_f106(d(u));
        assert_eq!(
            (generic.hi().to_bits(), generic.lo().to_bits()),
            (direct.hi().to_bits(), direct.lo().to_bits()),
            "generic and Float106 quantiles differ at u = {u:e}"
        );
    }
}

/// The generic form carries real precision at a scalar neither named function mentions.
///
/// `f32` is the check that the function is generic in fact and not only in signature: the result
/// must be the `f32` nearest the true quantile, which is what rounding the `Float106` value gives,
/// and it must be antisymmetric about `u = 0.5` as the transform itself is.
#[test]
fn test_generic_quantile_at_a_scalar_no_named_function_mentions() {
    for u in [0.025_f64, 0.1, 0.25, 0.4, 0.6, 0.75, 0.9, 0.975] {
        let got: f32 = standard_normal_inverse_cdf_at(u);
        let reference = standard_normal_inverse_cdf(u) as f32;
        assert_eq!(
            got.to_bits(),
            reference.to_bits(),
            "f32 quantile at u = {u} is {got}, not the rounded {reference}"
        );

        let mirrored: f32 = standard_normal_inverse_cdf_at(1.0 - u);
        assert_eq!(got, -mirrored, "Φ⁻¹(u) = −Φ⁻¹(1 − u) at u = {u}");
    }
}
