/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Diffusivity, PhysicsErrorEnum};

#[test]
fn test_diffusivity() {
    let eta = Diffusivity::<f64>::new(1.0).unwrap();
    assert_eq!(eta.value(), 1.0);
    assert!(
        matches!(
            Diffusivity::<f64>::new(-1.0).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            Diffusivity::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            Diffusivity::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_diffusivity_new_unchecked() {
    let eta = Diffusivity::<f64>::new_unchecked(1.0);
    assert_eq!(eta.value(), 1.0);
}

#[test]
fn test_diffusivity_default() {
    let eta: Diffusivity<f64> = Default::default();
    assert_eq!(eta.value(), 0.0);
}

#[test]
fn test_diffusivity_new_nan_error() {
    assert!(
        matches!(
            Diffusivity::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            Diffusivity::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}
