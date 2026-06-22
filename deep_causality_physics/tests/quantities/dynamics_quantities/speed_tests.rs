/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Speed;

#[test]
fn test_speed_new_valid() {
    let speed = Speed::<f64>::new(100.0);
    assert!(speed.is_ok());
    assert!((speed.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_speed_new_zero() {
    let speed = Speed::<f64>::new(0.0);
    assert!(speed.is_ok());
}

#[test]
fn test_speed_new_negative_error() {
    let speed = Speed::<f64>::new(-50.0);
    assert!(speed.is_err());
}

#[test]
fn test_speed_new_nan_error() {
    let speed = Speed::<f64>::new(f64::NAN);
    assert!(speed.is_err());
}

#[test]
fn test_speed_new_infinity_error() {
    let speed = Speed::<f64>::new(f64::INFINITY);
    assert!(speed.is_err());
}

#[test]
fn test_speed_into_f64() {
    let speed = Speed::<f64>::new(299792458.0).unwrap();
    let val: f64 = speed.into();
    assert!((val - 299792458.0).abs() < 1.0);
}

#[test]
fn test_speed_new_unchecked() {
    let speed = Speed::<f64>::new_unchecked(123.45);
    assert!((speed.value() - 123.45).abs() < 1e-10);
}

#[test]
fn test_speed_default() {
    let s: Speed<f64> = Speed::default();
    assert_eq!(s.value(), 0.0);
}
