/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Conductivity;

#[test]
fn test_conductivity() {
    let s = Conductivity::<f64>::new(1e7).unwrap();
    assert_eq!(s.value(), 1e7);
    assert!(Conductivity::<f64>::new(0.0).is_err());
    assert!(Conductivity::<f64>::new(f64::NAN).is_err());
    assert!(Conductivity::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_conductivity_new_unchecked() {
    let s = Conductivity::<f64>::new_unchecked(1e7);
    assert_eq!(s.value(), 1e7);
}

#[test]
fn test_conductivity_default() {
    let s: Conductivity<f64> = Default::default();
    assert!(s.value() > 0.0);
}

#[test]
fn test_conductivity_new_nan_error() {
    assert!(Conductivity::<f64>::new(f64::NAN).is_err());
    assert!(Conductivity::<f64>::new(f64::INFINITY).is_err());
}
