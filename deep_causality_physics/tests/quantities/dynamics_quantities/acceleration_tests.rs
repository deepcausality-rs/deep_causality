/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Acceleration allows negative values for direction.

use deep_causality_physics::Acceleration;

#[test]
fn test_acceleration_new_positive() {
    let acc = Acceleration::<f64>::new(9.81);
    assert!(acc.is_ok());
}

#[test]
fn test_acceleration_new_negative() {
    // Negative acceleration is valid (deceleration)
    let acc = Acceleration::<f64>::new(-5.0);
    assert!(acc.is_ok());
    assert!((acc.unwrap().value() - (-5.0)).abs() < 1e-10);
}

#[test]
fn test_acceleration_new_nan_error() {
    let acc = Acceleration::<f64>::new(f64::NAN);
    assert!(acc.is_err());
}

#[test]
fn test_acceleration_new_infinity_error() {
    let acc = Acceleration::<f64>::new(f64::INFINITY);
    assert!(acc.is_err());
}

#[test]
fn test_acceleration_new_unchecked() {
    let acc = Acceleration::<f64>::new_unchecked(-9.81);
    assert!((acc.value() - (-9.81)).abs() < 1e-10);
}

#[test]
fn test_acceleration_into_f64() {
    let acc = Acceleration::<f64>::new(9.8).unwrap();
    let val: f64 = acc.into();
    assert!((val - 9.8).abs() < 1e-10);
}

#[test]
fn test_acceleration_default() {
    let a: Acceleration<f64> = Acceleration::default();
    assert_eq!(a.value(), 0.0);
}
