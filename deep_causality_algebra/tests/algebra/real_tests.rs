/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `Real` analytic-scalar trait and its relationship to `RealField`.
//!
//! The analytic method surface itself (sqrt/exp/ln/sin/…) is exercised, per type,
//! in `field_real_f32_tests`, `field_real_f64_tests`, and the `float_double` suite
//! (all now through `Real`, which owns those methods). This file covers the trait
//! relationship the refactor introduces: `RealField: Real + Field`, that every
//! concrete real scalar is a `Real`, and that a `Real` bound accepts any `RealField`.

use deep_causality_algebra::{Real, RealField};
use deep_causality_num::Float106;

/// A generic that needs only the analytic surface — bounded on `Real`, not `RealField`.
fn analytic_sqrt<T: Real>(x: T) -> T {
    x.sqrt()
}

/// `RealField` implies `Real`: a `RealField`-bounded value is usable where `Real` is required.
fn through_realfield<T: RealField>(x: T) -> T {
    analytic_sqrt(x)
}

#[test]
fn realfield_implies_real_f64() {
    assert_eq!(through_realfield(9.0_f64), 3.0);
}

#[test]
fn realfield_implies_real_f32() {
    assert_eq!(through_realfield(16.0_f32), 4.0);
}

#[test]
fn real_bound_accepts_concrete_scalars() {
    assert_eq!(analytic_sqrt(25.0_f64), 5.0);
    assert_eq!(analytic_sqrt(49.0_f32), 7.0);
}

#[test]
fn real_surface_constants_and_functions_f64() {
    assert_eq!(<f64 as Real>::pi(), core::f64::consts::PI);
    assert_eq!(<f64 as Real>::e(), core::f64::consts::E);
    assert_eq!(<f64 as Real>::sqrt(4.0), 2.0);
    assert!((<f64 as Real>::exp(1.0) - core::f64::consts::E).abs() < 1e-12);
    assert!((<f64 as Real>::ln(core::f64::consts::E) - 1.0).abs() < 1e-12);
}

#[test]
fn real_surface_constants_and_functions_f32() {
    assert_eq!(<f32 as Real>::pi(), core::f32::consts::PI);
    assert_eq!(<f32 as Real>::sqrt(9.0), 3.0);
}

#[test]
fn concrete_scalars_are_real() {
    fn assert_real<T: Real>() {}
    assert_real::<f32>();
    assert_real::<f64>();
    assert_real::<Float106>();
}

#[test]
fn concrete_scalars_are_realfield() {
    fn assert_real_field<T: RealField>() {}
    assert_real_field::<f32>();
    assert_real_field::<f64>();
    assert_real_field::<Float106>();
}

#[test]
fn float106_real_surface() {
    let four = Float106::from_f64(4.0);
    let two = Float106::from_f64(2.0);
    let diff = <Float106 as Real>::sqrt(four) - two;
    assert!(diff.abs() < Float106::from_f64(1e-12));
}
