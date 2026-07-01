/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::VibrationalTemperature;

#[test]
fn test_vibrational_temperature_valid() {
    let t = VibrationalTemperature::<f64>::new(6.0e3).unwrap();
    assert_eq!(t.value(), 6.0e3);
    assert_eq!(
        VibrationalTemperature::<f64>::new(0.0).unwrap().value(),
        0.0
    );
}

#[test]
fn test_vibrational_temperature_rejects_negative() {
    assert!(VibrationalTemperature::<f64>::new(-1.0).is_err());
}

#[test]
fn test_vibrational_temperature_rejects_nonfinite() {
    assert!(VibrationalTemperature::<f64>::new(f64::NAN).is_err());
    assert!(VibrationalTemperature::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_vibrational_temperature_new_unchecked() {
    let t = VibrationalTemperature::<f64>::new_unchecked(5.5e3);
    assert_eq!(t.value(), 5.5e3);
}

#[test]
fn test_vibrational_temperature_default() {
    let t: VibrationalTemperature<f64> = Default::default();
    assert_eq!(t.value(), 0.0);
}

#[test]
fn test_vibrational_temperature_into_f64() {
    let t = VibrationalTemperature::<f64>::new(4.2e3).unwrap();
    let v: f64 = t.into();
    assert_eq!(v, 4.2e3);
}
