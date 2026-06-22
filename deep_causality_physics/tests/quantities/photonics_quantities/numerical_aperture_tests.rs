/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::NumericalAperture;

#[test]
fn test_numerical_aperture() {
    let na = NumericalAperture::<f64>::new(0.5).unwrap();
    assert_eq!(na.value(), 0.5);

    let err = NumericalAperture::<f64>::new(0.0);
    assert!(err.is_err());
}

#[test]
fn test_numerical_aperture_new_unchecked() {
    let na = NumericalAperture::<f64>::new_unchecked(0.5);
    assert_eq!(na.value(), 0.5);
}

#[test]
fn test_numerical_aperture_default() {
    let na: NumericalAperture<f64> = Default::default();
    assert_eq!(na.value(), 0.0);
}

#[test]
fn test_numerical_aperture_into_f64() {
    let v: f64 = NumericalAperture::<f64>::new(0.65).unwrap().into();
    assert!((v - 0.65).abs() < 1e-10);
}
