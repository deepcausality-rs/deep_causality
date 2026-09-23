/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{LarmorRadius, PhysicsErrorEnum};

#[test]
fn test_larmor_radius() {
    let r = LarmorRadius::<f64>::new(1.0).unwrap();
    assert_eq!(r.value(), 1.0);
    assert!(
        matches!(
            LarmorRadius::<f64>::new(0.0).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    ); // Must be positive
    assert!(
        matches!(
            LarmorRadius::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            LarmorRadius::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_larmor_radius_new_unchecked() {
    let r = LarmorRadius::<f64>::new_unchecked(1.0);
    assert_eq!(r.value(), 1.0);
}

#[test]
fn test_larmor_radius_default() {
    let r: LarmorRadius<f64> = Default::default();
    assert!(r.value() > 0.0);
}

#[test]
fn test_larmor_radius_new_nan_error() {
    assert!(
        matches!(
            LarmorRadius::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            LarmorRadius::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}
