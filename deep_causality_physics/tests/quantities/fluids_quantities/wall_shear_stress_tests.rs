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
    let tw = WallShearStress::<f64>::new(0.0).unwrap();
    // `is_ok()` alone admitted any carried value, including a constant.
    assert!(
        (tw.value() - (0.0)).abs() < 1e-10,
        "constructed value = {}",
        tw.value()
    );
}

#[test]
fn test_wall_shear_stress_new_negative_error() {
    let tw = WallShearStress::<f64>::new(-0.1);
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
    match &tw.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_wall_shear_stress_new_infinity_error() {
    assert!(
        matches!(
            WallShearStress::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
    assert!(
        matches!(
            WallShearStress::<f64>::new(f64::NEG_INFINITY)
                .unwrap_err()
                .0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
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
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other = WallShearStress::<f64>::new(0.9).unwrap();
    assert_ne!(a, other, "distinct values must not compare equal");
    assert_ne!(
        format!("{a:?}"),
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}
