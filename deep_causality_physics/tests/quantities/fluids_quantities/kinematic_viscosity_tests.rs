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
    let nu = KinematicViscosity::<f64>::new(0.0);
    assert!(nu.is_ok());
}

#[test]
fn test_kinematic_viscosity_new_negative_error() {
    let nu = KinematicViscosity::<f64>::new(-1.0e-5);
    assert!(nu.is_err());
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
    assert!(nu.is_err());
    match &nu.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_kinematic_viscosity_new_infinity_error() {
    assert!(KinematicViscosity::<f64>::new(f64::INFINITY).is_err());
    assert!(KinematicViscosity::<f64>::new(f64::NEG_INFINITY).is_err());
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
    let _ = format!("{:?}", a);
}
