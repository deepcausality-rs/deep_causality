/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Density, KinematicViscosity, PhysicsErrorEnum, Pressure, SpecificEnthalpy, Viscosity,
    WallShearStress,
};

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
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_pressure_traits() {
    let p1 = Pressure::<f64>::new(1.0).unwrap();
    let p2 = p1; // Copy
    let p3 = p1.clone(); // Clone (deliberately on Copy type)
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

// =============================================================================
// SpecificEnthalpy Tests (J/kg) — signed allowed
// =============================================================================

#[test]
fn test_specific_enthalpy_new_positive() {
    let h = SpecificEnthalpy::<f64>::new(2.5e5);
    assert!(h.is_ok());
    assert!((h.unwrap().value() - 2.5e5).abs() < 1e-6);
}

#[test]
fn test_specific_enthalpy_new_zero() {
    let h = SpecificEnthalpy::<f64>::new(0.0);
    assert!(h.is_ok());
}

#[test]
fn test_specific_enthalpy_new_negative_allowed() {
    // h is reference-state dependent and may be negative.
    let h = SpecificEnthalpy::<f64>::new(-1.0e4);
    assert!(h.is_ok());
    assert!((h.unwrap().value() + 1.0e4).abs() < 1e-6);
}

#[test]
fn test_specific_enthalpy_new_nan_error() {
    let h = SpecificEnthalpy::<f64>::new(f64::NAN);
    assert!(h.is_err());
    match &h.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected finite-check error"),
    }
}

#[test]
fn test_specific_enthalpy_new_infinity_error() {
    assert!(SpecificEnthalpy::<f64>::new(f64::INFINITY).is_err());
    assert!(SpecificEnthalpy::<f64>::new(f64::NEG_INFINITY).is_err());
}

#[test]
fn test_specific_enthalpy_new_unchecked() {
    let h = SpecificEnthalpy::<f64>::new_unchecked(3.0e5);
    assert!((h.value() - 3.0e5).abs() < 1e-6);
}

#[test]
fn test_specific_enthalpy_default() {
    let h: SpecificEnthalpy<f64> = SpecificEnthalpy::default();
    assert_eq!(h.value(), 0.0);
}

#[test]
fn test_specific_enthalpy_into_f64() {
    let h = SpecificEnthalpy::<f64>::new(1.0e5).unwrap();
    let v: f64 = h.into();
    assert!((v - 1.0e5).abs() < 1e-6);
}

#[test]
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_specific_enthalpy_traits() {
    let a = SpecificEnthalpy::<f64>::new(1.0e5).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    assert!(a < SpecificEnthalpy::<f64>::new(2.0e5).unwrap());
    let _ = format!("{:?}", a);
}

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
