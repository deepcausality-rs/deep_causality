/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, PlasmaFrequency};

#[test]
fn test_plasma_frequency() {
    let w = PlasmaFrequency::<f64>::new(1e9).unwrap();
    assert_eq!(w.value(), 1e9);
    assert!(
        matches!(
            PlasmaFrequency::<f64>::new(0.0).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            PlasmaFrequency::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            PlasmaFrequency::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
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
    assert!(
        matches!(
            PlasmaFrequency::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            PlasmaFrequency::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}
