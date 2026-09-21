/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, PlasmaBeta};

#[test]
fn test_plasma_beta() {
    let beta = PlasmaBeta::<f64>::new(0.5).unwrap();
    assert_eq!(beta.value(), 0.5);
    assert!(
        matches!(
            PlasmaBeta::<f64>::new(-0.1).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            PlasmaBeta::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            PlasmaBeta::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_plasma_beta_new_unchecked() {
    let beta = PlasmaBeta::<f64>::new_unchecked(0.5);
    assert_eq!(beta.value(), 0.5);
}

#[test]
fn test_plasma_beta_default() {
    let beta: PlasmaBeta<f64> = Default::default();
    assert_eq!(beta.value(), 0.0);
}

#[test]
fn test_plasma_beta_new_nan_error() {
    assert!(
        matches!(
            PlasmaBeta::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            PlasmaBeta::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}
