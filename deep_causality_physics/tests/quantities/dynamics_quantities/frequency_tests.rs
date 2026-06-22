/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Frequency;

#[test]
fn test_frequency_new_valid() {
    let freq = Frequency::<f64>::new(440.0); // A4 note
    assert!(freq.is_ok());
}

#[test]
fn test_frequency_new_negative_error() {
    let freq = Frequency::<f64>::new(-1.0);
    assert!(freq.is_err());
}

#[test]
fn test_frequency_new_nan_error() {
    let freq = Frequency::<f64>::new(f64::NAN);
    assert!(freq.is_err());
}

#[test]
fn test_frequency_new_infinity_error() {
    let freq = Frequency::<f64>::new(f64::INFINITY);
    assert!(freq.is_err());
}

#[test]
fn test_frequency_new_unchecked() {
    let freq = Frequency::<f64>::new_unchecked(60.0);
    assert!((freq.value() - 60.0).abs() < 1e-10);
}

#[test]
fn test_frequency_into_f64() {
    let freq = Frequency::<f64>::new(50.0).unwrap();
    let val: f64 = freq.into();
    assert!((val - 50.0).abs() < 1e-10);
}

#[test]
fn test_frequency_default() {
    let f: Frequency<f64> = Frequency::default();
    assert_eq!(f.value(), 0.0);
}
