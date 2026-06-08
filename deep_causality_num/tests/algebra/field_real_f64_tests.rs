/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::{DivisionAlgebra, Real};

#[test]
fn test_nan() {
    let f = f64::NAN;
    assert!(f.is_nan());
}

#[test]
fn test_clamp() {
    let x = 5.5_f64;
    assert_eq!(Real::clamp(x, 0.0, 10.0), x.clamp(0.0, 10.0));
}

#[test]
fn test_sqrt() {
    let x = 9.0_f64;
    assert_eq!(Real::sqrt(x), x.sqrt());
}

#[test]
fn test_abs() {
    let x = -3.2_f64;
    assert_eq!(Real::abs(x), x.abs());
}

#[test]
fn test_floor() {
    let x = 3.7_f64;
    assert_eq!(Real::floor(x), x.floor());
}

#[test]
fn test_ceil() {
    let x = 3.2_f64;
    assert_eq!(Real::ceil(x), x.ceil());
}

#[test]
fn test_round() {
    let x = 3.5_f64;
    assert_eq!(Real::round(x), x.round());
}

#[test]
fn test_exp() {
    // Use tolerance comparison: `Real::exp` may dispatch through the
    // `libm` path (no_std + libm_math feature) while `f64::exp` is the std
    // inherent method. The two implementations agree to ~1 ULP. Match the
    // pattern used by `test_ln` immediately below.
    let x = 1.0_f64;
    assert!((Real::exp(x) - x.exp()).abs() < 1e-10);
}

#[test]
fn test_ln() {
    let x: f64 = core::f64::consts::E;
    assert!((Real::ln(x) - x.ln()).abs() < 1e-10);
}

#[test]
fn test_log() {
    let x = 8.0_f64;
    let base = 2.0_f64;
    assert!((Real::log(x, base) - x.log(base)).abs() < 1e-10);
}

#[test]
fn test_log2() {
    let x = 8.0_f64;
    assert_eq!(Real::log2(x), 3.0);
    assert!((Real::log2(x) - x.log2()).abs() < 1e-10);
    // base-2 log of 1 is 0
    assert_eq!(Real::log2(1.0_f64), 0.0);
}

#[test]
fn test_log10() {
    let x = 1000.0_f64;
    assert_eq!(Real::log10(x), 3.0);
    assert!((Real::log10(x) - x.log10()).abs() < 1e-10);
    // base-10 log of 1 is 0
    assert_eq!(Real::log10(1.0_f64), 0.0);
}

#[test]
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg_attr(miri, ignore)]
fn test_powf() {
    let x = 2.0_f64;
    let n = 3.0_f64;
    assert_eq!(Real::powf(x, n), x.powf(n));
}

#[test]
fn test_trig() {
    let angle = 0.5_f64;
    assert!((Real::sin(angle) - angle.sin()).abs() < 1e-12);
    assert!((Real::cos(angle) - angle.cos()).abs() < 1e-12);
    assert!((Real::tan(angle) - angle.tan()).abs() < 1e-12);
    assert!((Real::acos(1.0_f64) - 0.0_f64).abs() < 1e-12);
    assert!((Real::asin(0.5_f64) - 0.5_f64.asin()).abs() < 1e-12);
    assert!((Real::atan(1.0_f64) - 1.0_f64.atan()).abs() < 1e-12);
}

#[test]
fn test_hyperbolic() {
    let x = 0.5_f64;
    assert!((Real::sinh(x) - x.sinh()).abs() < 1e-12);
    assert!((Real::cosh(x) - x.cosh()).abs() < 1e-12);
    assert!((Real::tanh(x) - x.tanh()).abs() < 1e-12);
}

#[test]
fn test_atan2() {
    let y = 1.0_f64;
    let x = 2.0_f64;
    assert!((Real::atan2(y, x) - y.atan2(x)).abs() < 1e-12);
}

#[test]
fn test_division_algebra_f64() {
    let x = 2.0_f64;
    let y = 0.0_f64;

    // conjugate
    assert_eq!(x.conjugate(), x);
    assert_eq!(y.conjugate(), y);

    // norm_sqr
    assert_eq!(x.norm_sqr(), x * x);
    assert_eq!(y.norm_sqr(), y * y);

    // inverse
    assert_eq!(x.inverse(), 1.0 / x);
    assert!(y.inverse().is_infinite());
}

#[test]
fn test_is_nan_inf_finite_traits() {
    assert!(<f64 as Real>::is_nan(f64::NAN));
    assert!(!<f64 as Real>::is_nan(1.0));
    assert!(<f64 as Real>::is_infinite(f64::INFINITY));
    assert!(!<f64 as Real>::is_infinite(1.0));
    assert!(<f64 as Real>::is_finite(1.0));
    assert!(!<f64 as Real>::is_finite(f64::INFINITY));
}

#[test]
fn test_constants() {
    let f: f64 = Real::pi();
    assert_eq!(f, core::f64::consts::PI);

    let f: f64 = Real::e();
    assert_eq!(f, core::f64::consts::E);

    let f: f64 = Real::epsilon();
    assert_eq!(f, f64::EPSILON);
}
