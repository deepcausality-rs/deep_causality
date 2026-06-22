/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Length;

#[test]
fn test_length_new_valid() {
    let length = Length::<f64>::new(1.5e11);
    assert!(length.is_ok());
}

#[test]
fn test_length_new_negative_error() {
    let length = Length::<f64>::new(-1.0);
    assert!(length.is_err());
}

#[test]
fn test_length_new_nan_error() {
    let length = Length::<f64>::new(f64::NAN);
    assert!(length.is_err());
}

#[test]
fn test_length_new_infinity_error() {
    let length = Length::<f64>::new(f64::INFINITY);
    assert!(length.is_err());
}

#[test]
fn test_length_default() {
    let length = Length::<f64>::default();
    assert!((length.value() - 0.0).abs() < 1e-10);
}

#[test]
fn test_length_new_unchecked() {
    let length = Length::<f64>::new_unchecked(99.9);
    assert!((length.value() - 99.9).abs() < 1e-10);
}

#[test]
fn test_length_into_f64() {
    let length = Length::<f64>::new(10.0).unwrap();
    let val: f64 = length.into();
    assert!((val - 10.0).abs() < 1e-10);
}
