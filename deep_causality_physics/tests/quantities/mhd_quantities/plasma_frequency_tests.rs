/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::PlasmaFrequency;

#[test]
fn test_plasma_frequency() {
    let w = PlasmaFrequency::<f64>::new(1e9).unwrap();
    assert_eq!(w.value(), 1e9);
    assert!(PlasmaFrequency::<f64>::new(0.0).is_err());
    assert!(PlasmaFrequency::<f64>::new(f64::NAN).is_err());
    assert!(PlasmaFrequency::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_plasma_frequency_new_unchecked() {
    let w = PlasmaFrequency::<f64>::new_unchecked(1e9);
    assert_eq!(w.value(), 1e9);
}

#[test]
fn test_plasma_frequency_default() {
    let w: PlasmaFrequency<f64> = Default::default();
    assert!(w.value() > 0.0);
}

#[test]
fn test_plasma_frequency_new_nan_error() {
    assert!(PlasmaFrequency::<f64>::new(f64::NAN).is_err());
    assert!(PlasmaFrequency::<f64>::new(f64::INFINITY).is_err());
}
