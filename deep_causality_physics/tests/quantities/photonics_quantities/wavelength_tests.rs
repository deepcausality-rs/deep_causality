/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, Wavelength};

#[test]
fn test_wavelength() {
    let w = Wavelength::<f64>::new(500e-9).unwrap();
    assert_eq!(w.value(), 500e-9);

    let err = Wavelength::<f64>::new(-1.0);
    assert!(
        matches!(
            err.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_wavelength_new_unchecked() {
    let w = Wavelength::<f64>::new_unchecked(500e-9);
    assert_eq!(w.value(), 500e-9);
}

#[test]
fn test_wavelength_default() {
    let w: Wavelength<f64> = Default::default();
    assert_eq!(w.value(), 0.0);
}

#[test]
fn test_wavelength_into_f64() {
    let v: f64 = Wavelength::<f64>::new(500e-9).unwrap().into();
    assert!((v - 500e-9).abs() < 1e-18);
}
