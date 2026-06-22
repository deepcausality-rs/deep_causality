/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Volume;

#[test]
fn test_volume_new_valid() {
    let volume = Volume::<f64>::new(1000.0);
    assert!(volume.is_ok());
}

#[test]
fn test_volume_new_negative_error() {
    let volume = Volume::<f64>::new(-1.0);
    assert!(volume.is_err());
}

#[test]
fn test_volume_new_nan_error() {
    let volume = Volume::<f64>::new(f64::NAN);
    assert!(volume.is_err());
}

#[test]
fn test_volume_new_infinity_error() {
    let volume = Volume::<f64>::new(f64::INFINITY);
    assert!(volume.is_err());
}

#[test]
fn test_volume_new_unchecked() {
    let volume = Volume::<f64>::new_unchecked(123.0);
    assert!((volume.value() - 123.0).abs() < 1e-10);
}

#[test]
fn test_volume_into_f64() {
    let volume = Volume::<f64>::new(100.0).unwrap();
    let val: f64 = volume.into();
    assert!((val - 100.0).abs() < 1e-10);
}

#[test]
fn test_volume_default() {
    let v: Volume<f64> = Volume::default();
    assert_eq!(v.value(), 0.0);
}
