/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Density, PhysicsErrorEnum};

// =============================================================================
// Density Tests
// =============================================================================

#[test]
fn test_density_new_valid() {
    let density = Density::<f64>::new(1000.0); // water
    assert!(density.is_ok());
    assert!((density.unwrap().value() - 1000.0).abs() < 1e-10);
}

#[test]
fn test_density_new_negative_error() {
    let density = Density::<f64>::new(-1.0);
    assert!(
        matches!(
            density.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_density_new_unchecked() {
    let density = Density::<f64>::new_unchecked(7874.0); // iron
    assert!((density.value() - 7874.0).abs() < 1e-10);
}

#[test]
fn test_density_into_f64() {
    let density = Density::<f64>::new(100.0).unwrap();
    let val: f64 = density.into();
    assert!((val - 100.0).abs() < 1e-10);
}

// =============================================================================
// Default impls (R::zero())
// =============================================================================

#[test]
fn test_density_default() {
    let d: Density<f64> = Density::default();
    assert_eq!(d.value(), 0.0);
}

// =============================================================================
// Non-finite validation paths
// =============================================================================

#[test]
fn test_density_new_nan_error() {
    let d = Density::<f64>::new(f64::NAN);
    match &d.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_density_new_infinity_error() {
    assert!(
        matches!(
            Density::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

// =============================================================================
// Trait coverage: Debug / Clone / Copy / PartialEq / PartialOrd
// =============================================================================

#[test]
fn test_density_traits() {
    let a = Density::<f64>::new(1.0).unwrap();
    let b = a;
    assert_eq!(a, b);
    assert!(a < Density::<f64>::new(2.0).unwrap());
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other = Density::<f64>::new(2.0).unwrap();
    assert_ne!(a, other, "distinct values must not compare equal");
    assert_ne!(
        format!("{a:?}"),
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}
