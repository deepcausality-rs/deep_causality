/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Transcendental function tests for `DoubleFloat`.

use deep_causality_num::{Float, Float106, Real};

// =============================================================================
// Helper Functions
// =============================================================================

fn d(x: f64) -> Float106 {
    Float106::from_f64(x)
}

fn approx_eq(a: Float106, b: Float106, epsilon: f64) -> bool {
    let diff = <Float106 as Float>::abs(a - b);
    diff.hi() < epsilon
}

// =============================================================================
// Constants Tests
// =============================================================================

#[test]
fn test_pi_constant() {
    let pi = Float106::PI;
    // Verify hi component matches core::f64::consts::PI
    assert_eq!(pi.hi(), core::f64::consts::PI);
    // Verify lo component adds precision
    assert!(pi.lo() != 0.0);
}

#[test]
fn test_e_constant() {
    let e = Float106::E;
    assert_eq!(e.hi(), core::f64::consts::E);
    assert!(e.lo() != 0.0);
}

#[test]
fn test_ln2_constant() {
    let ln2 = Float106::LN_2;
    assert_eq!(ln2.hi(), core::f64::consts::LN_2);
}

// =============================================================================
// Exponential and Logarithm Tests
// =============================================================================

#[test]
fn test_exp_zero() {
    let result = <Float106 as Float>::exp(d(0.0));
    assert!(approx_eq(result, d(1.0), 1e-15));
}

#[test]
fn test_exp_one() {
    let result = <Float106 as Float>::exp(d(1.0));
    assert!(approx_eq(result, Float106::E, 1e-14));
}

#[test]
fn test_exp_negative() {
    let result = <Float106 as Float>::exp(d(-1.0));
    let expected = d(1.0) / Float106::E;
    assert!(approx_eq(result, expected, 1e-14));
}

#[test]
fn test_ln_one() {
    let result = <Float106 as Float>::ln(d(1.0));
    assert!(approx_eq(result, d(0.0), 1e-15));
}

#[test]
fn test_ln_e() {
    let result = <Float106 as Float>::ln(Float106::E);
    assert!(approx_eq(result, d(1.0), 1e-14));
}

#[test]
fn test_exp_ln_identity() {
    // e^(ln(x)) = x
    let x = d(2.5);
    let result = <Float106 as Float>::exp(<Float106 as Float>::ln(x));
    assert!(approx_eq(result, x, 1e-13));
}

#[test]
fn test_ln_exp_identity() {
    // ln(e^x) = x
    let x = d(1.5);
    let result = <Float106 as Float>::ln(<Float106 as Float>::exp(x));
    assert!(approx_eq(result, x, 1e-13));
}

#[test]
fn test_log_base() {
    // log_10(100) = 2
    let result = <Float106 as Float>::log(d(100.0), d(10.0));
    assert!(approx_eq(result, d(2.0), 1e-14));
}

#[test]
fn test_log2() {
    let result = <Float106 as Float>::log2(d(8.0));
    assert!(approx_eq(result, d(3.0), 1e-14));
}

#[test]
fn test_log10() {
    let result = <Float106 as Float>::log10(d(1000.0));
    assert!(approx_eq(result, d(3.0), 1e-14));
}

// =============================================================================
// Power Tests
// =============================================================================

#[test]
fn test_sqrt() {
    let result = <Float106 as Float>::sqrt(d(4.0));
    assert!(approx_eq(result, d(2.0), 1e-15));
}

#[test]
fn test_sqrt_precision() {
    let result = <Float106 as Float>::sqrt(d(2.0));
    // sqrt(2) * sqrt(2) should equal 2
    let product = result * result;
    assert!(approx_eq(product, d(2.0), 1e-14));
}

#[test]
fn test_cbrt() {
    let result = <Float106 as Float>::cbrt(d(8.0));
    assert!(approx_eq(result, d(2.0), 1e-14));
}

#[test]
fn test_cbrt_negative() {
    let result = <Float106 as Float>::cbrt(d(-8.0));
    assert!(approx_eq(result, d(-2.0), 1e-14));
}

#[test]
fn test_powi() {
    let result = <Float106 as Float>::powi(d(2.0), 10);
    assert!(approx_eq(result, d(1024.0), 1e-14));
}

#[test]
fn test_powi_negative() {
    let result = <Float106 as Float>::powi(d(2.0), -2);
    assert!(approx_eq(result, d(0.25), 1e-15));
}

#[test]
#[cfg_attr(miri, ignore)] // Miri's soft-float emulation produces last-bit differences vs hardware; test is correct under normal CI.
fn test_powf() {
    let result = <Float106 as Float>::powf(d(2.0), d(3.0));
    assert!(approx_eq(result, d(8.0), 1e-14));
}

// =============================================================================
// Trigonometric Tests
// =============================================================================

#[test]
fn test_sin_zero() {
    let result = <Float106 as Float>::sin(d(0.0));
    assert!(approx_eq(result, d(0.0), 1e-15));
}

#[test]
fn test_sin_pi_2() {
    let result = <Float106 as Float>::sin(Float106::FRAC_PI_2);
    assert!(approx_eq(result, d(1.0), 1e-14));
}

#[test]
fn test_sin_pi() {
    let result = <Float106 as Float>::sin(Float106::PI);
    assert!(<Float106 as Float>::abs(result).hi() < 1e-14);
}

#[test]
fn test_cos_zero() {
    let result = <Float106 as Float>::cos(d(0.0));
    assert!(approx_eq(result, d(1.0), 1e-15));
}

#[test]
fn test_cos_pi_2() {
    let result = <Float106 as Float>::cos(Float106::FRAC_PI_2);
    assert!(<Float106 as Float>::abs(result).hi() < 1e-14);
}

#[test]
fn test_cos_pi() {
    let result = <Float106 as Float>::cos(Float106::PI);
    assert!(approx_eq(result, d(-1.0), 1e-14));
}

#[test]
fn test_sin_cos_identity() {
    // sin²(x) + cos²(x) = 1
    let x = d(0.5);
    let sin_x = <Float106 as Float>::sin(x);
    let cos_x = <Float106 as Float>::cos(x);
    let result = sin_x * sin_x + cos_x * cos_x;
    assert!(approx_eq(result, d(1.0), 1e-14));
}

#[test]
fn test_tan() {
    let result = <Float106 as Float>::tan(Float106::FRAC_PI_4);
    assert!(approx_eq(result, d(1.0), 1e-14));
}

#[test]
fn test_sin_cos() {
    let x = d(0.5);
    let (sin_x, cos_x) = <Float106 as Float>::sin_cos(x);
    assert!(approx_eq(sin_x, <Float106 as Float>::sin(x), 1e-15));
    assert!(approx_eq(cos_x, <Float106 as Float>::cos(x), 1e-15));
}

// =============================================================================
// Inverse Trigonometric Tests
// =============================================================================

#[test]
fn test_asin() {
    let result = <Float106 as Float>::asin(d(0.5));
    // asin(0.5) = π/6
    let expected = Float106::PI / d(6.0);
    assert!(approx_eq(result, expected, 1e-13));
}

#[test]
fn test_acos() {
    let result = <Float106 as Float>::acos(d(0.5));
    // acos(0.5) = π/3
    let expected = Float106::PI / d(3.0);
    assert!(approx_eq(result, expected, 1e-13));
}

#[test]
fn test_atan() {
    let result = <Float106 as Float>::atan(d(1.0));
    assert!(approx_eq(result, Float106::FRAC_PI_4, 1e-14));
}

#[test]
fn test_atan2() {
    let result = <Float106 as Float>::atan2(d(1.0), d(1.0));
    assert!(approx_eq(result, Float106::FRAC_PI_4, 1e-14));
}

// =============================================================================
// Hyperbolic Tests
// =============================================================================

#[test]
fn test_sinh_zero() {
    let result = <Float106 as Float>::sinh(d(0.0));
    assert!(approx_eq(result, d(0.0), 1e-15));
}

#[test]
fn test_cosh_zero() {
    let result = <Float106 as Float>::cosh(d(0.0));
    assert!(approx_eq(result, d(1.0), 1e-15));
}

#[test]
fn test_tanh_zero() {
    let result = <Float106 as Float>::tanh(d(0.0));
    assert!(approx_eq(result, d(0.0), 1e-15));
}

#[test]
fn test_sinh_cosh_identity() {
    // cosh²(x) - sinh²(x) = 1
    let x = d(1.0);
    let sinh_x = <Float106 as Float>::sinh(x);
    let cosh_x = <Float106 as Float>::cosh(x);
    let result = cosh_x * cosh_x - sinh_x * sinh_x;
    assert!(approx_eq(result, d(1.0), 1e-14));
}

#[test]
fn test_asinh() {
    let x = d(1.0);
    let result = <Float106 as Float>::asinh(<Float106 as Float>::sinh(x));
    assert!(approx_eq(result, x, 1e-14));
}

#[test]
fn test_acosh() {
    let x = d(2.0);
    let result = <Float106 as Float>::acosh(<Float106 as Float>::cosh(x));
    assert!(approx_eq(result, x, 1e-14));
}

#[test]
fn test_atanh() {
    let x = d(0.5);
    let result = <Float106 as Float>::atanh(<Float106 as Float>::tanh(x));
    assert!(approx_eq(result, x, 1e-14));
}

// =============================================================================
// Rounding and Floor/Ceil Tests
// =============================================================================

#[test]
fn test_floor() {
    assert_eq!(<Float106 as Float>::floor(d(2.7)), d(2.0));
    assert_eq!(<Float106 as Float>::floor(d(-2.7)), d(-3.0));
}

#[test]
fn test_ceil() {
    assert_eq!(<Float106 as Float>::ceil(d(2.3)), d(3.0));
    assert_eq!(<Float106 as Float>::ceil(d(-2.3)), d(-2.0));
}

#[test]
fn test_round() {
    assert_eq!(<Float106 as Float>::round(d(2.4)), d(2.0));
    assert_eq!(<Float106 as Float>::round(d(2.5)), d(3.0));
    assert_eq!(<Float106 as Float>::round(d(-2.5)), d(-3.0));
}

#[test]
fn test_trunc() {
    assert_eq!(<Float106 as Float>::trunc(d(2.7)), d(2.0));
    assert_eq!(<Float106 as Float>::trunc(d(-2.7)), d(-2.0));
}

#[test]
fn test_fract() {
    let x = d(2.75);
    let frac = <Float106 as Float>::fract(x);
    assert!(approx_eq(frac, d(0.75), 1e-15));
}

// =============================================================================
// Utility Function Tests
// =============================================================================

#[test]
fn test_abs() {
    assert_eq!(<Float106 as Float>::abs(d(5.0)), d(5.0));
    assert_eq!(<Float106 as Float>::abs(d(-5.0)), d(5.0));
}

#[test]
fn test_signum() {
    assert_eq!(<Float106 as Float>::signum(d(5.0)), d(1.0));
    assert_eq!(<Float106 as Float>::signum(d(-5.0)), d(-1.0));
    assert_eq!(<Float106 as Float>::signum(d(0.0)), d(0.0));
}

#[test]
fn test_clamp() {
    assert_eq!(<Float106 as Float>::clamp(d(5.0), d(0.0), d(10.0)), d(5.0));
    assert_eq!(<Float106 as Float>::clamp(d(-5.0), d(0.0), d(10.0)), d(0.0));
    assert_eq!(
        <Float106 as Float>::clamp(d(15.0), d(0.0), d(10.0)),
        d(10.0)
    );
}

#[test]
fn test_hypot() {
    let result = <Float106 as Float>::hypot(d(3.0), d(4.0));
    assert!(approx_eq(result, d(5.0), 1e-15));
}

#[test]
fn test_recip() {
    let result = <Float106 as Float>::recip(d(4.0));
    assert!(approx_eq(result, d(0.25), 1e-15));
}

// =============================================================================
// Angle Conversion Tests
// =============================================================================

#[test]
fn test_to_degrees() {
    let result = <Float106 as Float>::to_degrees(Float106::PI);
    assert!(approx_eq(result, d(180.0), 1e-13));
}

#[test]
fn test_to_radians() {
    let result = <Float106 as Float>::to_radians(d(180.0));
    assert!(approx_eq(result, Float106::PI, 1e-13));
}

// =============================================================================
// Real Trait Tests
// =============================================================================

#[test]
fn test_realfield_pi() {
    let pi = <Float106 as Real>::pi();
    assert_eq!(pi, Float106::PI);
}

#[test]
fn test_realfield_e() {
    let e = <Float106 as Real>::e();
    assert_eq!(e, Float106::E);
}

#[test]
fn test_realfield_epsilon() {
    let eps = <Float106 as Real>::epsilon();
    assert!(eps.hi() > 0.0);
    assert!(eps.hi() < 1e-30);
}

// =============================================================================
// High Precision Verification
// =============================================================================

#[test]
fn test_sin_precision_extended() {
    // sin(1.0) for precision check
    let x = d(1.0);
    let result = <Float106 as Float>::sin(x);

    // Check consistency: sin^2 + cos^2 - 1 should be extremely close to 0
    let cos_x = <Float106 as Float>::cos(x);
    let unity = result * result + cos_x * cos_x;
    let diff = unity - d(1.0);

    // With 15 iterations (old), error was ~1e-16
    // With 60 iterations (new), error should be < 1e-31
    assert!(
        <Float106 as Float>::abs(diff).hi() < 1e-31,
        "sin^2 + cos^2 prec: {:e}",
        diff.hi()
    );
}

// =============================================================================
// Table-based sin_cos fast-path accuracy tests
// =============================================================================

/// One reference sample: input, sin as (hi, lo), cos as (hi, lo).
type SinCosRef = (f64, (f64, f64), (f64, f64));

/// Reference values computed with mpmath at 50 decimal digits and split into
/// double-double (hi, lo) components. The inputs are f64-exact.
const SIN_COS_REFS: [SinCosRef; 12] = [
    (
        0.1,
        (0.09983341664682815, 3.08001512929492e-18),
        (0.9950041652780258, -5.50210156918377e-17),
    ),
    (
        0.5,
        (0.479425538604203, -5.103969860556013e-18),
        (0.8775825618903728, -4.2623149864279997e-17),
    ),
    (
        1.0,
        (0.8414709848078965, 1.776845092935536e-18),
        (0.5403023058681398, -4.760954612604417e-17),
    ),
    (
        1.5,
        (0.9974949866040544, -1.4558643538840918e-17),
        (0.0707372016677029, 3.683512075225569e-18),
    ),
    (
        2.0,
        (0.9092974268256817, -1.4020906557816256e-17),
        (-0.4161468365471424, 1.990596398957495e-17),
    ),
    (
        3.0,
        (0.1411200080598672, 8.577269787017502e-18),
        (-0.9899924966004454, -4.2060261566099734e-17),
    ),
    (
        3.1,
        (0.04158066243329049, -7.108355207879104e-19),
        (-0.9991351502732795, 1.3850578802683375e-17),
    ),
    (
        4.0,
        (-0.7568024953079282, -4.892224089158451e-17),
        (-0.6536436208636119, 2.5846614087018284e-17),
    ),
    (
        5.0,
        (-0.9589242746631385, -1.4926316946126356e-17),
        (0.28366218546322625, 1.8192990004462368e-17),
    ),
    (
        6.0,
        (-0.27941549819892586, -1.2659979684764697e-17),
        (0.960170286650366, 5.330529085568646e-17),
    ),
    (
        -0.7,
        (-0.644217687237691, -2.8740567927338755e-18),
        (0.7648421872844885, -4.013780434022238e-17),
    ),
    (
        -2.5,
        (-0.5984721441039565, 5.521403334082375e-17),
        (-0.8011436155469337, -1.8674742705085553e-17),
    ),
];

#[test]
fn test_sin_cos_against_mpmath_references() {
    for (x, (s_hi, s_lo), (c_hi, c_lo)) in SIN_COS_REFS {
        let (sin_x, cos_x) = <Float106 as Float>::sin_cos(d(x));
        let sin_ref = Float106::new(s_hi, s_lo);
        let cos_ref = Float106::new(c_hi, c_lo);
        let sin_err = <Float106 as Float>::abs(sin_x - sin_ref);
        let cos_err = <Float106 as Float>::abs(cos_x - cos_ref);
        assert!(
            sin_err.hi() < 1e-31,
            "sin({x}) err {:e} exceeds 1e-31",
            sin_err.hi()
        );
        assert!(
            cos_err.hi() < 1e-31,
            "cos({x}) err {:e} exceeds 1e-31",
            cos_err.hi()
        );
    }
}

#[test]
fn test_sin_cos_pythagorean_identity_dense() {
    // 256 samples across [-2π, 2π); the identity must hold to double-double
    // precision everywhere, including near the octant boundaries k·π/16.
    for i in 0..256 {
        let x = d((i as f64 - 128.0) * (4.0 * core::f64::consts::PI / 256.0));
        let (sin_x, cos_x) = <Float106 as Float>::sin_cos(x);
        let unity = sin_x * sin_x + cos_x * cos_x;
        let diff = <Float106 as Float>::abs(unity - d(1.0));
        assert!(
            diff.hi() < 1e-31,
            "sin²+cos² at sample {i}: {:e}",
            diff.hi()
        );
    }
}

#[test]
fn test_sin_cos_exact_anchors() {
    // The table makes the anchors nearly exact: sin(π) is the double-double
    // residual of π itself (~1e-33), far below the old Taylor error.
    let (sin_pi, cos_pi) = <Float106 as Float>::sin_cos(Float106::PI);
    assert!(<Float106 as Float>::abs(sin_pi).hi() < 1e-31);
    assert!(<Float106 as Float>::abs(cos_pi + d(1.0)).hi() < 1e-31);

    let (sin_h, cos_h) = <Float106 as Float>::sin_cos(Float106::FRAC_PI_2);
    assert!(<Float106 as Float>::abs(sin_h - d(1.0)).hi() < 1e-31);
    assert!(<Float106 as Float>::abs(cos_h).hi() < 1e-31);

    let (sin_q, cos_q) = <Float106 as Float>::sin_cos(Float106::FRAC_PI_4);
    assert!(<Float106 as Float>::abs(sin_q - cos_q).hi() < 1e-31);
}

#[test]
fn test_sin_cos_odd_even_symmetry() {
    for v in [0.3, 1.2, 2.9, 3.7] {
        let (sin_p, cos_p) = <Float106 as Float>::sin_cos(d(v));
        let (sin_n, cos_n) = <Float106 as Float>::sin_cos(d(-v));
        assert!(<Float106 as Float>::abs(sin_p + sin_n).hi() < 1e-31);
        assert!(<Float106 as Float>::abs(cos_p - cos_n).hi() < 1e-31);
    }
}

#[test]
fn test_sin_cos_nan_propagates() {
    let (s, c) = <Float106 as Float>::sin_cos(<Float106 as Float>::nan());
    assert!(s.hi().is_nan());
    assert!(c.hi().is_nan());
}
