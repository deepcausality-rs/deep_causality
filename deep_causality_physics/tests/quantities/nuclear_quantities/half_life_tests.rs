/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::HalfLife;

#[test]
fn test_half_life_new_valid() {
    let hl = HalfLife::<f64>::new(5730.0); // C-14
    assert!(hl.is_ok());
}

#[test]
fn test_half_life_new_zero() {
    // Zero half-life is invalid because it implies infinite decay rate
    let hl = HalfLife::<f64>::new(0.0);
    assert!(hl.is_err(), "Zero half-life should be rejected");
}

#[test]
fn test_half_life_new_negative_error() {
    let hl = HalfLife::<f64>::new(-100.0);
    assert!(hl.is_err());
}

#[test]
fn test_half_life_new_unchecked() {
    let hl = HalfLife::<f64>::new_unchecked(1600.0); // Ra-226
    assert!((hl.value() - 1600.0).abs() < 1e-10);
}

#[test]
fn test_half_life_from_f64() {
    let hl = HalfLife::<f64>::new(123.0).unwrap();
    let val: f64 = hl.into();
    assert!((val - 123.0).abs() < 1e-10);
}

#[test]
fn test_half_life_default_uses_epsilon() {
    let h: HalfLife<f64> = HalfLife::default();
    // Default uses R::epsilon() to preserve strict-positive invariant
    assert!(h.value() > 0.0);
}

#[test]
fn test_half_life_new_nan_error() {
    assert!(HalfLife::<f64>::new(f64::NAN).is_err());
    assert!(HalfLife::<f64>::new(f64::INFINITY).is_err());
}
