/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{PhysicsErrorEnum, Viscosity};

// =============================================================================
// Viscosity Tests
// =============================================================================

#[test]
fn test_viscosity_new_valid() {
    let visc = Viscosity::<f64>::new(0.001).unwrap(); // water at 20°C
    assert!(
        (visc.value() - 0.001).abs() < 1e-12,
        "mu = {}",
        visc.value()
    );
}

#[test]
fn test_viscosity_new_negative_error() {
    let visc = Viscosity::<f64>::new(-0.5);
    assert!(
        matches!(
            visc.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_viscosity_new_unchecked() {
    let visc = Viscosity::<f64>::new_unchecked(1.0);
    assert!((visc.value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_viscosity_into_f64() {
    let visc = Viscosity::<f64>::new(0.005).unwrap();
    let val: f64 = visc.into();
    assert!((val - 0.005).abs() < 1e-10);
}

// =============================================================================
// Default impls (R::zero())
// =============================================================================

#[test]
fn test_viscosity_default() {
    let v: Viscosity<f64> = Viscosity::default();
    assert_eq!(v.value(), 0.0);
}

// =============================================================================
// Non-finite validation paths
// =============================================================================

#[test]
fn test_viscosity_new_nan_error() {
    assert!(
        matches!(
            Viscosity::<f64>::new(f64::NAN).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_viscosity_new_infinity_error() {
    assert!(
        matches!(
            Viscosity::<f64>::new(f64::INFINITY).unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

// =============================================================================
// Trait coverage: Debug / Clone / Copy / PartialEq / PartialOrd
// =============================================================================

#[test]
fn test_viscosity_traits() {
    let a = Viscosity::<f64>::new(0.5).unwrap();
    let b = a;
    assert_eq!(a, b);
    assert!(a < Viscosity::<f64>::new(1.0).unwrap());
    // `assert_eq!(x, x.clone())` is reflexive and holds for a `PartialEq` that always
    // returns true. The inequality discriminates, and comparing the two `Debug`
    // renderings makes `Debug` observable rather than discarded.
    let other = Viscosity::<f64>::new(1.5).unwrap();
    assert_ne!(a, other, "distinct values must not compare equal");
    assert_ne!(
        format!("{a:?}"),
        format!("{other:?}"),
        "Debug must distinguish distinct values"
    );
}
