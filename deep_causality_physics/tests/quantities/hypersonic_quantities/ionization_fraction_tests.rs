/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::IonizationFraction;

#[test]
fn test_ionization_fraction_valid() {
    assert_eq!(IonizationFraction::<f64>::new(0.0).unwrap().value(), 0.0);
    assert_eq!(IonizationFraction::<f64>::new(0.5).unwrap().value(), 0.5);
    assert_eq!(IonizationFraction::<f64>::new(1.0).unwrap().value(), 1.0);
}

#[test]
fn test_ionization_fraction_rejects_out_of_range() {
    assert!(IonizationFraction::<f64>::new(-0.01).is_err());
    assert!(IonizationFraction::<f64>::new(1.01).is_err());
}

#[test]
fn test_ionization_fraction_rejects_nonfinite() {
    assert!(IonizationFraction::<f64>::new(f64::NAN).is_err());
    assert!(IonizationFraction::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_ionization_fraction_new_unchecked() {
    let a = IonizationFraction::<f64>::new_unchecked(0.25);
    assert_eq!(a.value(), 0.25);
}

#[test]
fn test_ionization_fraction_default() {
    let a: IonizationFraction<f64> = Default::default();
    assert_eq!(a.value(), 0.0);
}

#[test]
fn test_ionization_fraction_into_f64() {
    let a = IonizationFraction::<f64>::new(0.3).unwrap();
    let v: f64 = a.into();
    assert_eq!(v, 0.3);
}
