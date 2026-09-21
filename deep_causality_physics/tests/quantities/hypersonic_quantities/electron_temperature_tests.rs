/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{ElectronTemperature, PhysicsErrorEnum};

#[test]
fn test_electron_temperature_valid() {
    let t = ElectronTemperature::<f64>::new(1.0e4).unwrap();
    assert_eq!(t.value(), 1.0e4);
    assert_eq!(ElectronTemperature::<f64>::new(0.0).unwrap().value(), 0.0);
}

#[test]
fn test_electron_temperature_rejects_negative() {
    assert!(
        matches!(
            ElectronTemperature::<f64>::new(-1.0).unwrap_err().0,
            PhysicsErrorEnum::ZeroKelvinViolation
        ),
        "expected a ZeroKelvinViolation refusal"
    );
}

#[test]
fn test_electron_temperature_rejects_nonfinite() {
    assert!(
        matches!(
            ElectronTemperature::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            ElectronTemperature::<f64>::new(f64::INFINITY)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_electron_temperature_new_unchecked() {
    let t = ElectronTemperature::<f64>::new_unchecked(8.0e3);
    assert_eq!(t.value(), 8.0e3);
}

#[test]
fn test_electron_temperature_default() {
    let t: ElectronTemperature<f64> = Default::default();
    assert_eq!(t.value(), 0.0);
}

#[test]
fn test_electron_temperature_into_f64() {
    let t = ElectronTemperature::<f64>::new(7.6e3).unwrap();
    let v: f64 = t.into();
    assert_eq!(v, 7.6e3);
}
