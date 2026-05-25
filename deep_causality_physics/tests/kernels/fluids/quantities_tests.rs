/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AccelerationVector, BodyForceDensity, CauchyStress, Density, KinematicViscosity,
    PhysicsErrorEnum, Pressure, RotationRateTensor, SpecificEnthalpy, StrainRateTensor, Velocity3,
    VelocityGradient, Viscosity, VorticityVector, WallShearStress,
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

// =============================================================================
// Velocity3 — finiteness-only invariant
// =============================================================================

#[test]
fn test_velocity3_new_valid() {
    let u = Velocity3::<f64>::new([1.0, -2.0, 3.5]).unwrap();
    assert_eq!(u.value(), &[1.0, -2.0, 3.5]);
}

#[test]
fn test_velocity3_new_rejects_nan() {
    assert!(Velocity3::<f64>::new([f64::NAN, 0.0, 0.0]).is_err());
    assert!(Velocity3::<f64>::new([0.0, f64::NAN, 0.0]).is_err());
    assert!(Velocity3::<f64>::new([0.0, 0.0, f64::NAN]).is_err());
}

#[test]
fn test_velocity3_new_rejects_infinity() {
    assert!(Velocity3::<f64>::new([f64::INFINITY, 0.0, 0.0]).is_err());
    assert!(Velocity3::<f64>::new([0.0, f64::NEG_INFINITY, 0.0]).is_err());
}

#[test]
fn test_velocity3_new_error_message_mentions_finite() {
    match &Velocity3::<f64>::new([f64::NAN, 0.0, 0.0]).unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("finite")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_velocity3_default() {
    assert_eq!(Velocity3::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_velocity3_new_unchecked() {
    let u = Velocity3::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(u.value(), &[1.0, 2.0, 3.0]);
}

#[test]
fn test_velocity3_into_inner_consumes() {
    let u = Velocity3::<f64>::new([4.0, 5.0, 6.0]).unwrap();
    assert_eq!(u.into_inner(), [4.0, 5.0, 6.0]);
}

#[test]
#[allow(clippy::clone_on_copy)] // exercising Clone impl for coverage
fn test_velocity3_traits() {
    let a = Velocity3::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// VorticityVector — finiteness only
// =============================================================================

#[test]
fn test_vorticity_vector_new_valid() {
    let w = VorticityVector::<f64>::new([0.5, -0.5, 1.0]).unwrap();
    assert_eq!(w.value(), &[0.5, -0.5, 1.0]);
}

#[test]
fn test_vorticity_vector_rejects_non_finite() {
    assert!(VorticityVector::<f64>::new([f64::NAN, 0.0, 0.0]).is_err());
    assert!(VorticityVector::<f64>::new([f64::INFINITY, 0.0, 0.0]).is_err());
}

#[test]
fn test_vorticity_vector_default() {
    assert_eq!(VorticityVector::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_vorticity_vector_new_unchecked() {
    let w = VorticityVector::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(w.into_inner(), [1.0, 2.0, 3.0]);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_vorticity_vector_traits() {
    let a = VorticityVector::<f64>::new([0.1, 0.2, 0.3]).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// AccelerationVector — finiteness only
// =============================================================================

#[test]
fn test_acceleration_vector_new_valid() {
    let a = AccelerationVector::<f64>::new([9.81, 0.0, 0.0]).unwrap();
    assert_eq!(a.value(), &[9.81, 0.0, 0.0]);
}

#[test]
fn test_acceleration_vector_rejects_non_finite() {
    assert!(AccelerationVector::<f64>::new([f64::NAN, 0.0, 0.0]).is_err());
    assert!(AccelerationVector::<f64>::new([0.0, 0.0, f64::INFINITY]).is_err());
}

#[test]
fn test_acceleration_vector_default() {
    assert_eq!(AccelerationVector::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_acceleration_vector_new_unchecked() {
    let a = AccelerationVector::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(a.into_inner(), [1.0, 2.0, 3.0]);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_acceleration_vector_traits() {
    let a = AccelerationVector::<f64>::new([1.0, 2.0, 3.0]).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// BodyForceDensity — finiteness only
// =============================================================================

#[test]
fn test_body_force_density_new_valid() {
    let f = BodyForceDensity::<f64>::new([0.0, 0.0, -9810.0]).unwrap();
    assert_eq!(f.value(), &[0.0, 0.0, -9810.0]);
}

#[test]
fn test_body_force_density_rejects_non_finite() {
    assert!(BodyForceDensity::<f64>::new([f64::NAN, 0.0, 0.0]).is_err());
    assert!(BodyForceDensity::<f64>::new([0.0, f64::NEG_INFINITY, 0.0]).is_err());
}

#[test]
fn test_body_force_density_default() {
    assert_eq!(BodyForceDensity::<f64>::default().into_inner(), [0.0; 3]);
}

#[test]
fn test_body_force_density_new_unchecked() {
    let f = BodyForceDensity::<f64>::new_unchecked([1.0, 2.0, 3.0]);
    assert_eq!(f.into_inner(), [1.0, 2.0, 3.0]);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_body_force_density_traits() {
    let a = BodyForceDensity::<f64>::new([0.1, 0.2, 0.3]).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// VelocityGradient — Jacobian convention pinned at construction
// =============================================================================

#[test]
fn test_velocity_gradient_new_valid() {
    let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let g = VelocityGradient::<f64>::new(m).unwrap();
    assert_eq!(g.value(), &m);
}

#[test]
fn test_velocity_gradient_rejects_non_finite() {
    let mut m = [[0.0; 3]; 3];
    m[1][2] = f64::NAN;
    assert!(VelocityGradient::<f64>::new(m).is_err());
}

#[test]
fn test_velocity_gradient_default_is_zero() {
    assert_eq!(
        VelocityGradient::<f64>::default().into_inner(),
        [[0.0; 3]; 3]
    );
}

#[test]
fn test_velocity_gradient_new_unchecked() {
    let m = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    assert_eq!(VelocityGradient::<f64>::new_unchecked(m).into_inner(), m);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_velocity_gradient_traits() {
    let m = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let a = VelocityGradient::<f64>::new(m).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// StrainRateTensor — symmetric (S_ij == S_ji)
// =============================================================================

#[test]
fn test_strain_rate_tensor_new_valid_symmetric() {
    let s = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let t = StrainRateTensor::<f64>::new(s).unwrap();
    assert_eq!(t.value(), &s);
}

#[test]
fn test_strain_rate_tensor_rejects_asymmetric() {
    // S_01 = 2.0 but S_10 = 9.0 — clearly asymmetric
    let s = [[1.0, 2.0, 3.0], [9.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let r = StrainRateTensor::<f64>::new(s);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("symmetric")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_strain_rate_tensor_rejects_non_finite() {
    let s = [[f64::NAN, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    assert!(StrainRateTensor::<f64>::new(s).is_err());
}

#[test]
fn test_strain_rate_tensor_default_is_zero() {
    assert_eq!(
        StrainRateTensor::<f64>::default().into_inner(),
        [[0.0; 3]; 3]
    );
}

#[test]
fn test_strain_rate_tensor_new_unchecked_bypasses_check() {
    // Asymmetric matrix — accepted via new_unchecked, would be rejected by new.
    let s = [[1.0, 2.0, 0.0], [9.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let t = StrainRateTensor::<f64>::new_unchecked(s);
    assert_eq!(t.into_inner(), s);
}

#[test]
fn test_strain_rate_tensor_from_self_to_raw_drops_invariant() {
    let s = [[1.0, 2.0, 3.0], [2.0, 4.0, 5.0], [3.0, 5.0, 6.0]];
    let t = StrainRateTensor::<f64>::new(s).unwrap();
    let raw: [[f64; 3]; 3] = t.into();
    assert_eq!(raw, s);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_strain_rate_tensor_traits() {
    let s = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let a = StrainRateTensor::<f64>::new(s).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// RotationRateTensor — antisymmetric (Ω_ij == -Ω_ji, Ω_ii == 0)
// =============================================================================

#[test]
fn test_rotation_rate_tensor_new_valid_antisymmetric() {
    let omega = [[0.0, 1.0, 2.0], [-1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]];
    let t = RotationRateTensor::<f64>::new(omega).unwrap();
    assert_eq!(t.value(), &omega);
}

#[test]
fn test_rotation_rate_tensor_rejects_nonzero_diagonal() {
    let omega = [[1.0, 1.0, 2.0], [-1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]];
    let r = RotationRateTensor::<f64>::new(omega);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("diagonal") || msg.contains("antisymmetric"))
        }
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_rotation_rate_tensor_rejects_non_antisymmetric_off_diagonal() {
    // Ω_01 = 1.0 but Ω_10 = 1.0 (should be -1.0)
    let omega = [[0.0, 1.0, 2.0], [1.0, 0.0, 3.0], [-2.0, -3.0, 0.0]];
    let r = RotationRateTensor::<f64>::new(omega);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("antisymmetric")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_rotation_rate_tensor_rejects_non_finite() {
    let omega = [
        [0.0, f64::INFINITY, 0.0],
        [-f64::INFINITY, 0.0, 0.0],
        [0.0, 0.0, 0.0],
    ];
    assert!(RotationRateTensor::<f64>::new(omega).is_err());
}

#[test]
fn test_rotation_rate_tensor_default_is_zero() {
    assert_eq!(
        RotationRateTensor::<f64>::default().into_inner(),
        [[0.0; 3]; 3]
    );
}

#[test]
fn test_rotation_rate_tensor_new_unchecked_bypasses_check() {
    let omega = [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let t = RotationRateTensor::<f64>::new_unchecked(omega);
    assert_eq!(t.into_inner(), omega);
}

#[test]
fn test_rotation_rate_tensor_from_self_to_raw() {
    let omega = [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let t = RotationRateTensor::<f64>::new(omega).unwrap();
    let raw: [[f64; 3]; 3] = t.into();
    assert_eq!(raw, omega);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_rotation_rate_tensor_traits() {
    let omega = [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
    let a = RotationRateTensor::<f64>::new(omega).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// CauchyStress — symmetric
// =============================================================================

#[test]
fn test_cauchy_stress_new_valid_symmetric() {
    let sigma = [[100.0, 5.0, 3.0], [5.0, 200.0, 7.0], [3.0, 7.0, 300.0]];
    let t = CauchyStress::<f64>::new(sigma).unwrap();
    assert_eq!(t.value(), &sigma);
}

#[test]
fn test_cauchy_stress_rejects_asymmetric() {
    let sigma = [[100.0, 5.0, 3.0], [99.0, 200.0, 7.0], [3.0, 7.0, 300.0]];
    let r = CauchyStress::<f64>::new(sigma);
    assert!(r.is_err());
    match &r.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => assert!(msg.contains("symmetric")),
        _ => panic!("Expected PhysicalInvariantBroken"),
    }
}

#[test]
fn test_cauchy_stress_rejects_non_finite() {
    let sigma = [[0.0, 0.0, 0.0], [0.0, f64::NAN, 0.0], [0.0, 0.0, 0.0]];
    assert!(CauchyStress::<f64>::new(sigma).is_err());
}

#[test]
fn test_cauchy_stress_default_is_zero() {
    assert_eq!(CauchyStress::<f64>::default().into_inner(), [[0.0; 3]; 3]);
}

#[test]
fn test_cauchy_stress_new_unchecked_bypasses_check() {
    let sigma = [[1.0, 2.0, 3.0], [9.0, 5.0, 6.0], [3.0, 6.0, 9.0]];
    let t = CauchyStress::<f64>::new_unchecked(sigma);
    assert_eq!(t.into_inner(), sigma);
}

#[test]
fn test_cauchy_stress_from_self_to_raw() {
    let sigma = [[1.0, 0.0, 0.0], [0.0, 2.0, 0.0], [0.0, 0.0, 3.0]];
    let t = CauchyStress::<f64>::new(sigma).unwrap();
    let raw: [[f64; 3]; 3] = t.into();
    assert_eq!(raw, sigma);
}

#[test]
#[allow(clippy::clone_on_copy)]
fn test_cauchy_stress_traits() {
    let sigma = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    let a = CauchyStress::<f64>::new(sigma).unwrap();
    let b = a;
    let c = a.clone();
    assert_eq!(a, b);
    assert_eq!(a, c);
    let _ = format!("{:?}", a);
}

// =============================================================================
// Property test: any VelocityGradient decomposes as S + Ω
// =============================================================================

#[test]
fn test_velocity_gradient_decomposes_into_strain_and_rotation() {
    // Arbitrary finite gradient
    let g = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];
    let _grad = VelocityGradient::<f64>::new(g).unwrap();

    // Symmetric part S = 0.5*(G + G^T)
    let mut s = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            s[i][j] = 0.5 * (g[i][j] + g[j][i]);
        }
    }
    // Antisymmetric part Omega = 0.5*(G - G^T)
    let mut omega = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            omega[i][j] = 0.5 * (g[i][j] - g[j][i]);
        }
    }

    let strain = StrainRateTensor::<f64>::new(s).unwrap();
    let rotation = RotationRateTensor::<f64>::new(omega).unwrap();

    // Verify S + Omega == G
    let s_raw: [[f64; 3]; 3] = strain.into();
    let o_raw: [[f64; 3]; 3] = rotation.into();
    for i in 0..3 {
        for j in 0..3 {
            assert!((s_raw[i][j] + o_raw[i][j] - g[i][j]).abs() < 1e-12);
        }
    }
}
