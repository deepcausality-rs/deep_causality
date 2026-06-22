/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::Viscosity;

// =============================================================================
// Viscosity Tests
// =============================================================================

#[test]
fn test_viscosity_new_valid() {
    let visc = Viscosity::<f64>::new(0.001); // water at 20°C
    assert!(visc.is_ok());
}

#[test]
fn test_viscosity_new_negative_error() {
    let visc = Viscosity::<f64>::new(-0.5);
    assert!(visc.is_err());
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
    assert!(Viscosity::<f64>::new(f64::NAN).is_err());
}

#[test]
fn test_viscosity_new_infinity_error() {
    assert!(Viscosity::<f64>::new(f64::INFINITY).is_err());
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
    let _ = format!("{:?}", a);
}
