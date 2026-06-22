/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Conductance, PhysicsErrorEnum};

#[test]
fn test_conductance() {
    let c = Conductance::<f64>::new(0.1).unwrap();
    assert_eq!(c.value(), 0.1);

    let err = Conductance::<f64>::new(-0.1);
    assert!(err.is_err());
    match err.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(_) => {}
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_conductance_new_unchecked() {
    let c = Conductance::<f64>::new_unchecked(0.1);
    assert_eq!(c.value(), 0.1);
}

#[test]
fn test_conductance_default() {
    let c: Conductance<f64> = Default::default();
    assert_eq!(c.value(), 0.0);
}

#[test]
fn test_conductance_into_f64() {
    let c = Conductance::<f64>::new(3.5).unwrap();
    let val: f64 = c.into();
    assert!((val - 3.5).abs() < 1e-10);
}
