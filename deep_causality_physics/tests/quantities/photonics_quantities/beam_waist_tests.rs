/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{BeamWaist, PhysicsErrorEnum};

#[test]
fn test_beam_waist() {
    let w0 = BeamWaist::<f64>::new(1e-3).unwrap();
    assert_eq!(w0.value(), 1e-3);

    let err = BeamWaist::<f64>::new(-1.0);
    assert!(
        matches!(
            err.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_beam_waist_new_unchecked() {
    let w0 = BeamWaist::<f64>::new_unchecked(1e-3);
    assert_eq!(w0.value(), 1e-3);
}

#[test]
fn test_beam_waist_default() {
    let w0: BeamWaist<f64> = Default::default();
    assert_eq!(w0.value(), 0.0);
}

#[test]
fn test_beam_waist_into_f64() {
    let v: f64 = BeamWaist::<f64>::new(1e-3).unwrap().into();
    assert!((v - 1e-3).abs() < 1e-12);
}
