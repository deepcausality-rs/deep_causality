/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{KinematicViscosity, PhysicsErrorEnum};

// =============================================================================
// KinematicViscosity Tests (m^2/s)
// =============================================================================

#[test]
fn test_kinematic_viscosity_new_valid() {
    // water at 20C: ~1.0e-6 m^2/s
    let nu = KinematicViscosity::<f64>::new(1.0e-6);
    assert!(nu.is_ok());
    assert!((nu.unwrap().value() - 1.0e-6).abs() < 1e-18);
}

#[test]
fn test_kinematic_viscosity_new_zero() {
    let nu = KinematicViscosity::<f64>::new(0.0).unwrap();
    // `is_ok()` alone admitted any carried value, including a constant.
    assert!(
        (nu.value() - (0.0)).abs() < 1e-10,
        "constructed value = {}",
        nu.value()
    );
}

#[test]
fn test_kinematic_viscosity_new_negative_error() {
    let nu = KinematicViscosity::<f64>::new(-1.0e-5);
    match &nu.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("Negative") || msg.contains("KinematicViscosity"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_kinematic_viscosity_new_nan_error() {
    let nu = KinematicViscosity::<f64>::new(f64::NAN);
    match &nu.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_kinematic_viscosity_new_infinity_error() {
    assert!(
        matches!(
            KinematicViscosity::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            KinematicViscosity::<f64>::new(f64::NEG_INFINITY)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_kinematic_viscosity_new_unchecked() {
    let nu = KinematicViscosity::<f64>::new_unchecked(2.0e-5);
    assert!((nu.value() - 2.0e-5).abs() < 1e-18);
}

#[test]
fn test_kinematic_viscosity_default() {
    let nu: KinematicViscosity<f64> = KinematicViscosity::default();
    assert_eq!(nu.value(), 0.0);
}

#[test]
fn test_kinematic_viscosity_into_f64() {
    let nu = KinematicViscosity::<f64>::new(1.5e-6).unwrap();
    let v: f64 = nu.into();
    assert!((v - 1.5e-6).abs() < 1e-18);
}

#[test]
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_kinematic_viscosity_traits() {
    let a = KinematicViscosity::<f64>::new(1.0e-6).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert!(a < KinematicViscosity::<f64>::new(2.0e-6).unwrap());
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other = KinematicViscosity::<f64>::new(2.0e-6).unwrap();
    assert_ne!(a, other, "distinct values must not compare equal");
    assert_ne!(
        format!("{a:?}"),
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}
