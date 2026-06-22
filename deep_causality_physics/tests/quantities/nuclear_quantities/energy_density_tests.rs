/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::EnergyDensity;

#[test]
fn test_energy_density_new_valid() {
    let ed = EnergyDensity::<f64>::new(100.0);
    assert!(ed.is_ok());
    assert!((ed.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_energy_density_new_negative_error() {
    let ed = EnergyDensity::<f64>::new(-50.0);
    assert!(ed.is_err());
}

#[test]
fn test_energy_density_unchecked() {
    let ed = EnergyDensity::<f64>::new_unchecked(25.0);
    assert!((ed.value() - 25.0).abs() < 1e-10);
}

#[test]
fn test_energy_density_into_f64() {
    let ed = EnergyDensity::<f64>::new(10.0).unwrap();
    let val: f64 = ed.into();
    assert!((val - 10.0).abs() < 1e-10);
}

#[test]
fn test_energy_density_default() {
    let e: EnergyDensity<f64> = EnergyDensity::default();
    assert_eq!(e.value(), 0.0);
}
