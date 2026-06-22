/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Force allows negative values for direction.

use deep_causality_physics::Force;

#[test]
fn test_force_new_positive() {
    let force = Force::<f64>::new(100.0);
    assert!(force.is_ok());
}

#[test]
fn test_force_new_negative() {
    let force = Force::<f64>::new(-50.0);
    assert!(force.is_ok());
}

#[test]
fn test_force_new_nan_error() {
    let force = Force::<f64>::new(f64::NAN);
    assert!(force.is_err());
}

#[test]
fn test_force_new_infinity_error() {
    let force = Force::<f64>::new(f64::INFINITY);
    assert!(force.is_err());
}

#[test]
fn test_force_new_unchecked() {
    let force = Force::<f64>::new_unchecked(-100.0);
    assert!((force.value() - (-100.0)).abs() < 1e-10);
}

#[test]
fn test_force_into_f64() {
    let force = Force::<f64>::new(10.0).unwrap();
    let val: f64 = force.into();
    assert!((val - 10.0).abs() < 1e-10);
}

#[test]
fn test_force_default() {
    let f: Force<f64> = Force::default();
    assert_eq!(f.value(), 0.0);
}
