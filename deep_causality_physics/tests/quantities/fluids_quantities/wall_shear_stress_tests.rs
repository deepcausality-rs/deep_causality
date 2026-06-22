/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, WallShearStress};

// =============================================================================
// WallShearStress Tests (Pa) — magnitude, non-negative
// =============================================================================

#[test]
fn test_wall_shear_stress_new_valid() {
    let tw = WallShearStress::<f64>::new(0.5);
    assert!(tw.is_ok());
    assert!((tw.unwrap().value() - 0.5).abs() < 1e-12);
}

#[test]
fn test_wall_shear_stress_new_zero() {
    let tw = WallShearStress::<f64>::new(0.0);
    assert!(tw.is_ok());
}

#[test]
fn test_wall_shear_stress_new_negative_error() {
    let tw = WallShearStress::<f64>::new(-0.1);
    assert!(tw.is_err());
    match &tw.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("Negative") || msg.contains("WallShearStress"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_wall_shear_stress_new_nan_error() {
    let tw = WallShearStress::<f64>::new(f64::NAN);
    assert!(tw.is_err());
    match &tw.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_wall_shear_stress_new_infinity_error() {
    assert!(WallShearStress::<f64>::new(f64::INFINITY).is_err());
    assert!(WallShearStress::<f64>::new(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_wall_shear_stress_new_unchecked() {
    let tw = WallShearStress::<f64>::new_unchecked(0.25);
    assert!((tw.value() - 0.25).abs() < 1e-12);
}

#[test]
fn test_wall_shear_stress_default() {
    let tw: WallShearStress<f64> = WallShearStress::default();
    assert_eq!(tw.value(), 0.0);
}

#[test]
fn test_wall_shear_stress_into_f64() {
    let tw = WallShearStress::<f64>::new(1.25).unwrap();
    let v: f64 = tw.into();
    assert!((v - 1.25).abs() < 1e-12);
}

#[test]
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_wall_shear_stress_traits() {
    let a = WallShearStress::<f64>::new(0.1).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert!(a < WallShearStress::<f64>::new(0.2).unwrap());
    let _ = format!("{:?}", a);
}
