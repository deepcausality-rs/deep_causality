/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::{DivisionAlgebra, RealField};

#[test]
fn test_nan() {
    assert!(<f32 as RealField>::nan().is_nan());
}

#[test]
fn test_clamp() {
    let x = 5.5_f32;
    assert_eq!(RealField::clamp(x, 0.0, 10.0), x.clamp(0.0, 10.0));
}

#[test]
fn test_sqrt() {
    let x = 9.0_f32;
    assert_eq!(RealField::sqrt(x), x.sqrt());
}

#[test]
fn test_abs() {
    let x = -3.2_f32;
    assert_eq!(RealField::abs(x), x.abs());
}

#[test]
fn test_floor() {
    let x = 3.7_f32;
    assert_eq!(RealField::floor(x), x.floor());
}

#[test]
fn test_ceil() {
    let x = 3.2_f32;
    assert_eq!(RealField::ceil(x), x.ceil());
}

#[test]
fn test_round() {
    let x = 3.5_f32;
    assert_eq!(RealField::round(x), x.round());
}

#[test]
fn test_exp() {
    let x = 1.0_f32;
    assert_eq!(RealField::exp(x), x.exp());
}

#[test]
fn test_ln() {
    let x: f32 = core::f32::consts::E;
    assert!((RealField::ln(x) - x.ln()).abs() < 1e-5);
}

#[test]
fn test_log() {
    let x = 8.0_f32;
    let base = 2.0_f32;
    assert!((RealField::log(x, base) - x.log(base)).abs() < 1e-5);
}

#[test]
fn test_powf() {
    let x = 2.0_f32;
    let n = 3.0_f32;
    assert_eq!(RealField::powf(x, n), x.powf(n));
}

#[test]
fn test_trig() {
    let angle = 0.5_f32;
    assert!((RealField::sin(angle) - angle.sin()).abs() < 1e-6);
    assert!((RealField::cos(angle) - angle.cos()).abs() < 1e-6);
    assert!((RealField::tan(angle) - angle.tan()).abs() < 1e-6);
    assert!((RealField::acos(1.0_f32) - 0.0_f32).abs() < 1e-6);
}

#[test]
fn test_hyperbolic() {
    let x = 0.5_f32;
    assert!((RealField::sinh(x) - x.sinh()).abs() < 1e-6);
    assert!((RealField::cosh(x) - x.cosh()).abs() < 1e-6);
    assert!((RealField::tanh(x) - x.tanh()).abs() < 1e-6);
}

#[test]
fn test_atan2() {
    let y = 1.0_f32;
    let x = 2.0_f32;
    assert!((RealField::atan2(y, x) - y.atan2(x)).abs() < 1e-6);
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
fn test_constants() {
    let f: f32 = RealField::pi();
    assert_eq!(f, core::f32::consts::PI);

    let f: f32 = RealField::e();
    assert_eq!(f, core::f32::consts::E);

    let f: f32 = RealField::epsilon();
    assert_eq!(f, f32::EPSILON);
}
