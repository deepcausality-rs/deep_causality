/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::MagneticPressure;

#[test]
fn test_magnetic_pressure() {
    let p = MagneticPressure::<f64>::new(1000.0).unwrap();
    assert_eq!(p.value(), 1000.0);
    assert!(MagneticPressure::<f64>::new(-10.0).is_err());
    assert!(MagneticPressure::<f64>::new(f64::NAN).is_err());
    assert!(MagneticPressure::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_magnetic_pressure_new_unchecked() {
    let p = MagneticPressure::<f64>::new_unchecked(1000.0);
    assert_eq!(p.value(), 1000.0);
}

#[test]
fn test_magnetic_pressure_default() {
    let p: MagneticPressure<f64> = Default::default();
    assert_eq!(p.value(), 0.0);
}

#[test]
fn test_magnetic_pressure_new_nan_error() {
    assert!(MagneticPressure::<f64>::new(f64::NAN).is_err());
    assert!(MagneticPressure::<f64>::new(f64::INFINITY).is_err());
}
