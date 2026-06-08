/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::{DivisionAlgebra, Real};

#[test]
fn test_nan() {
    assert!(<f32 as Real>::nan().is_nan());
}

#[test]
fn test_clamp() {
    let x = 5.5_f32;
    assert_eq!(Real::clamp(x, 0.0, 10.0), x.clamp(0.0, 10.0));
}

#[test]
fn test_sqrt() {
    let x = 9.0_f32;
    assert_eq!(Real::sqrt(x), x.sqrt());
}

#[test]
fn test_abs() {
    let x = -3.2_f32;
    assert_eq!(Real::abs(x), x.abs());
}

#[test]
fn test_floor() {
    let x = 3.7_f32;
    assert_eq!(Real::floor(x), x.floor());
}

#[test]
fn test_ceil() {
    let x = 3.2_f32;
    assert_eq!(Real::ceil(x), x.ceil());
}

#[test]
fn test_round() {
    let x = 3.5_f32;
    assert_eq!(Real::round(x), x.round());
}

#[test]
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg_attr(miri, ignore)]
fn test_exp() {
    let x = 1.0_f32;
    assert_eq!(Real::exp(x), x.exp());
}

#[test]
fn test_ln() {
    let x: f32 = core::f32::consts::E;
    assert!((Real::ln(x) - x.ln()).abs() < 1e-5);
}

#[test]
fn test_log() {
    let x = 8.0_f32;
    let base = 2.0_f32;
    assert!((Real::log(x, base) - x.log(base)).abs() < 1e-5);
}

#[test]
fn test_log2() {
    let x = 8.0_f32;
    assert!((Real::log2(x) - 3.0).abs() < 1e-5);
    assert!((Real::log2(x) - x.log2()).abs() < 1e-5);
    assert!(Real::log2(1.0_f32).abs() < 1e-5);
}

#[test]
fn test_log10() {
    let x = 1000.0_f32;
    assert!((Real::log10(x) - 3.0).abs() < 1e-5);
    assert!((Real::log10(x) - x.log10()).abs() < 1e-5);
    assert!(Real::log10(1.0_f32).abs() < 1e-5);
}

#[test]
// Disabled under Miri: software-emulated floats produce different last-bit
// results for transcendental ops, so exact equality cannot hold. The test
// itself is correct and runs under normal CI.
#[cfg_attr(miri, ignore)]
fn test_powf() {
    let x = 2.0_f32;
    let n = 3.0_f32;
    assert_eq!(Real::powf(x, n), x.powf(n));
}

#[test]
fn test_trig() {
    let angle = 0.5_f32;
    assert!((Real::sin(angle) - angle.sin()).abs() < 1e-6);
    assert!((Real::cos(angle) - angle.cos()).abs() < 1e-6);
    assert!((Real::tan(angle) - angle.tan()).abs() < 1e-6);
    assert!((Real::acos(1.0_f32) - 0.0_f32).abs() < 1e-6);
    assert!((Real::asin(0.5_f32) - 0.5_f32.asin()).abs() < 1e-6);
    assert!((Real::atan(1.0_f32) - 1.0_f32.atan()).abs() < 1e-6);
}

#[test]
fn test_hyperbolic() {
    let x = 0.5_f32;
    assert!((Real::sinh(x) - x.sinh()).abs() < 1e-6);
    assert!((Real::cosh(x) - x.cosh()).abs() < 1e-6);
    assert!((Real::tanh(x) - x.tanh()).abs() < 1e-6);
}

#[test]
fn test_atan2() {
    let y = 1.0_f32;
    let x = 2.0_f32;
    assert!((Real::atan2(y, x) - y.atan2(x)).abs() < 1e-6);
}

#[test]
fn test_division_algebra_f32() {
    let x = 2.0_f32;
    let y = 0.0_f32;

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
    assert!(<f32 as Real>::is_nan(f32::NAN));
    assert!(!<f32 as Real>::is_nan(1.0));
    assert!(<f32 as Real>::is_infinite(f32::INFINITY));
    assert!(!<f32 as Real>::is_infinite(1.0));
    assert!(<f32 as Real>::is_finite(1.0));
    assert!(!<f32 as Real>::is_finite(f32::INFINITY));
}

#[test]
fn test_constants() {
    let f: f32 = Real::pi();
    assert_eq!(f, core::f32::consts::PI);

    let f: f32 = Real::e();
    assert_eq!(f, core::f32::consts::E);

    let f: f32 = Real::epsilon();
    assert_eq!(f, f32::EPSILON);
}
