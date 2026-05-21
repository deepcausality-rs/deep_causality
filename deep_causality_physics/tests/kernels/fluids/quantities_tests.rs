/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{Density, PhysicsErrorEnum, Pressure, Viscosity};

// =============================================================================
// Pressure Tests
// =============================================================================

#[test]
fn test_pressure_new_valid() {
    let pressure = Pressure::<f64>::new(101325.0);
    assert!(pressure.is_ok());
    assert!((pressure.unwrap().value() - 101325.0).abs() < 1e-10);
}

#[test]
fn test_pressure_new_zero() {
    let pressure = Pressure::<f64>::new(0.0);
    assert!(pressure.is_ok());
}

#[test]
fn test_pressure_new_negative_error() {
    let pressure = Pressure::<f64>::new(-1.0);
    assert!(pressure.is_err());
    match &pressure.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("Pressure") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_pressure_new_unchecked() {
    let pressure = Pressure::<f64>::new_unchecked(50000.0);
    assert!((pressure.value() - 50000.0).abs() < 1e-10);
}

#[test]
fn test_pressure_into_f64() {
    let pressure = Pressure::<f64>::new(1000.0).unwrap();
    let val: f64 = pressure.into();
    assert!((val - 1000.0).abs() < 1e-10);
}

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
    assert!(density.is_err());
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
fn test_pressure_default() {
    let p: Pressure<f64> = Pressure::default();
    assert_eq!(p.value(), 0.0);
}

#[test]
fn test_density_default() {
    let d: Density<f64> = Density::default();
    assert_eq!(d.value(), 0.0);
}

#[test]
fn test_viscosity_default() {
    let v: Viscosity<f64> = Viscosity::default();
    assert_eq!(v.value(), 0.0);
}

// =============================================================================
// Non-finite validation paths
// =============================================================================

#[test]
fn test_pressure_new_nan_error() {
    let p = Pressure::<f64>::new(f64::NAN);
    assert!(p.is_err());
    match &p.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_pressure_new_infinity_error() {
    assert!(Pressure::<f64>::new(f64::INFINITY).is_err());
    assert!(Pressure::<f64>::new(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_density_new_nan_error() {
    let d = Density::<f64>::new(f64::NAN);
    assert!(d.is_err());
    match &d.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_density_new_infinity_error() {
    assert!(Density::<f64>::new(f64::INFINITY).is_err());
}

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
fn test_pressure_traits() {
    let p1 = Pressure::<f64>::new(1.0).unwrap();
    let p2 = p1; // Copy
    let p3 = p1.clone(); // Clone
    assert_eq!(p1, p2);
    assert_eq!(p1, p3);
    assert!(p1 < Pressure::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", p1); // Debug
}

#[test]
fn test_density_traits() {
    let a = Density::<f64>::new(1.0).unwrap();
    let b = a;
    assert_eq!(a, b);
    assert!(a < Density::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", a);
}

#[test]
fn test_viscosity_traits() {
    let a = Viscosity::<f64>::new(0.5).unwrap();
    let b = a;
    assert_eq!(a, b);
    assert!(a < Viscosity::<f64>::new(1.0).unwrap());
    let _ = format!("{:?}", a);
}
