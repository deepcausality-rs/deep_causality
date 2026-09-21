/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{AlfvenSpeed, PhysicsErrorEnum};

#[test]
fn test_alfven_speed() {
    let v = AlfvenSpeed::<f64>::new(100.0).unwrap();
    assert_eq!(v.value(), 100.0);
    assert!(
        matches!(
            AlfvenSpeed::<f64>::new(-1.0).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            AlfvenSpeed::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            AlfvenSpeed::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_alfven_speed_new_unchecked() {
    let v = AlfvenSpeed::<f64>::new_unchecked(100.0);
    assert_eq!(v.value(), 100.0);
}

#[test]
fn test_alfven_speed_default() {
    let v: AlfvenSpeed<f64> = Default::default();
    assert_eq!(v.value(), 0.0);
}

#[test]
fn test_alfven_speed_new_nan_error() {
    assert!(
        matches!(
            AlfvenSpeed::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            AlfvenSpeed::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}
