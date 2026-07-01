/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::MassFraction;

#[test]
fn test_mass_fraction_valid() {
    assert_eq!(MassFraction::<f64>::new(0.0).unwrap().value(), 0.0);
    assert_eq!(MassFraction::<f64>::new(0.77).unwrap().value(), 0.77);
    assert_eq!(MassFraction::<f64>::new(1.0).unwrap().value(), 1.0);
}

#[test]
fn test_mass_fraction_rejects_out_of_range() {
    assert!(MassFraction::<f64>::new(-0.01).is_err());
    assert!(MassFraction::<f64>::new(1.01).is_err());
}

#[test]
fn test_mass_fraction_rejects_nonfinite() {
    assert!(MassFraction::<f64>::new(f64::NAN).is_err());
    assert!(MassFraction::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_mass_fraction_new_unchecked() {
    let y = MassFraction::<f64>::new_unchecked(0.23);
    assert_eq!(y.value(), 0.23);
}

#[test]
fn test_mass_fraction_default() {
    let y: MassFraction<f64> = Default::default();
    assert_eq!(y.value(), 0.0);
}

#[test]
fn test_mass_fraction_into_f64() {
    let y = MassFraction::<f64>::new(0.5).unwrap();
    let v: f64 = y.into();
    assert_eq!(v, 0.5);
}
