/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Area;

#[test]
fn test_area_new_valid() {
    let area = Area::<f64>::new(100.0);
    assert!(area.is_ok());
    assert!((area.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_area_new_negative_error() {
    let area = Area::<f64>::new(-10.0);
    assert!(area.is_err());
}

#[test]
fn test_area_new_nan_error() {
    let area = Area::<f64>::new(f64::NAN);
    assert!(area.is_err());
}

#[test]
fn test_area_new_infinity_error() {
    let area = Area::<f64>::new(f64::INFINITY);
    assert!(area.is_err());
}

#[test]
fn test_area_new_unchecked() {
    let area = Area::<f64>::new_unchecked(50.5);
    assert!((area.value() - 50.5).abs() < 1e-10);
}

#[test]
fn test_area_into_f64() {
    let area = Area::<f64>::new(25.0).unwrap();
    let val: f64 = area.into();
    assert!((val - 25.0).abs() < 1e-10);
}

#[test]
fn test_area_default() {
    let a: Area<f64> = Area::default();
    assert_eq!(a.value(), 0.0);
}
