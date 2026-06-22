/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// Torque allows negative values for direction.

use deep_causality_physics::Torque;

#[test]
fn test_torque_new_positive() {
    let torque = Torque::<f64>::new(25.0);
    assert!(torque.is_ok());
}

#[test]
fn test_torque_new_negative() {
    // Negative torque = clockwise rotation
    let torque = Torque::<f64>::new(-25.0);
    assert!(torque.is_ok());
}

#[test]
fn test_torque_new_nan_error() {
    let torque = Torque::<f64>::new(f64::NAN);
    assert!(torque.is_err());
}

#[test]
fn test_torque_new_infinity_error() {
    let torque = Torque::<f64>::new(f64::INFINITY);
    assert!(torque.is_err());
}

#[test]
fn test_torque_new_unchecked() {
    let torque = Torque::<f64>::new_unchecked(-25.0);
    assert!((torque.value() - (-25.0)).abs() < 1e-10);
}

#[test]
fn test_torque_into_f64() {
    let torque = Torque::<f64>::new(50.0).unwrap();
    let val: f64 = torque.into();
    assert!((val - 50.0).abs() < 1e-10);
}

#[test]
fn test_torque_default() {
    let t: Torque<f64> = Torque::default();
    assert_eq!(t.value(), 0.0);
}
