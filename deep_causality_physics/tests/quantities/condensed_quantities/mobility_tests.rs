/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Mobility, PhysicsErrorEnum};

#[test]
fn test_mobility() {
    let m = Mobility::<f64>::new(100.0).unwrap();
    assert_eq!(m.value(), 100.0);

    let err = Mobility::<f64>::new(-1.0);
    assert!(
        matches!(
            err.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_mobility_new_unchecked() {
    let m = Mobility::<f64>::new_unchecked(100.0);
    assert_eq!(m.value(), 100.0);
}

#[test]
fn test_mobility_default() {
    let m: Mobility<f64> = Default::default();
    assert_eq!(m.value(), 0.0);
}

#[test]
fn test_mobility_into_f64() {
    let m = Mobility::<f64>::new(0.25).unwrap();
    let val: f64 = m.into();
    assert!((val - 0.25).abs() < 1e-10);
}
