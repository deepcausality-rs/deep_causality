/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::DebyeLength;

#[test]
fn test_debye_length() {
    let l = DebyeLength::<f64>::new(1e-6).unwrap();
    assert_eq!(l.value(), 1e-6);
    assert!(DebyeLength::<f64>::new(0.0).is_err());
    assert!(DebyeLength::<f64>::new(f64::NAN).is_err());
    assert!(DebyeLength::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_debye_length_new_unchecked() {
    let l = DebyeLength::<f64>::new_unchecked(1e-6);
    assert_eq!(l.value(), 1e-6);
}

#[test]
fn test_debye_length_default() {
    let l: DebyeLength<f64> = Default::default();
    assert!(l.value() > 0.0);
}

#[test]
fn test_debye_length_new_nan_error() {
    assert!(DebyeLength::<f64>::new(f64::NAN).is_err());
    assert!(DebyeLength::<f64>::new(f64::INFINITY).is_err());
}
