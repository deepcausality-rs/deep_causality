/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{MagneticPressure, PhysicsErrorEnum};

#[test]
fn test_magnetic_pressure() {
    let p = MagneticPressure::<f64>::new(1000.0).unwrap();
    assert_eq!(p.value(), 1000.0);
    assert!(
        matches!(
            MagneticPressure::<f64>::new(-10.0).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            MagneticPressure::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            MagneticPressure::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
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
    assert!(
        matches!(
            MagneticPressure::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            MagneticPressure::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}
