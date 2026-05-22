/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AlfvenSpeed, Conductivity, DebyeLength, Diffusivity, LarmorRadius, MagneticPressure,
    PlasmaBeta, PlasmaFrequency,
};

#[test]
fn test_alfven_speed() {
    let v = AlfvenSpeed::<f64>::new(100.0).unwrap();
    assert_eq!(v.value(), 100.0);
    assert!(AlfvenSpeed::<f64>::new(-1.0).is_err());
    assert!(AlfvenSpeed::<f64>::new(f64::NAN).is_err());
    assert!(AlfvenSpeed::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_plasma_beta() {
    let beta = PlasmaBeta::<f64>::new(0.5).unwrap();
    assert_eq!(beta.value(), 0.5);
    assert!(PlasmaBeta::<f64>::new(-0.1).is_err());
    assert!(PlasmaBeta::<f64>::new(f64::NAN).is_err());
    assert!(PlasmaBeta::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_magnetic_pressure() {
    let p = MagneticPressure::<f64>::new(1000.0).unwrap();
    assert_eq!(p.value(), 1000.0);
    assert!(MagneticPressure::<f64>::new(-10.0).is_err());
    assert!(MagneticPressure::<f64>::new(f64::NAN).is_err());
    assert!(MagneticPressure::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_larmor_radius() {
    let r = LarmorRadius::<f64>::new(1.0).unwrap();
    assert_eq!(r.value(), 1.0);
    assert!(LarmorRadius::<f64>::new(0.0).is_err()); // Must be positive
    assert!(LarmorRadius::<f64>::new(f64::NAN).is_err());
    assert!(LarmorRadius::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_debye_length() {
    let l = DebyeLength::<f64>::new(1e-6).unwrap();
    assert_eq!(l.value(), 1e-6);
    assert!(DebyeLength::<f64>::new(0.0).is_err());
    assert!(DebyeLength::<f64>::new(f64::NAN).is_err());
    assert!(DebyeLength::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_plasma_frequency() {
    let w = PlasmaFrequency::<f64>::new(1e9).unwrap();
    assert_eq!(w.value(), 1e9);
    assert!(PlasmaFrequency::<f64>::new(0.0).is_err());
    assert!(PlasmaFrequency::<f64>::new(f64::NAN).is_err());
    assert!(PlasmaFrequency::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_conductivity() {
    let s = Conductivity::<f64>::new(1e7).unwrap();
    assert_eq!(s.value(), 1e7);
    assert!(Conductivity::<f64>::new(0.0).is_err());
    assert!(Conductivity::<f64>::new(f64::NAN).is_err());
    assert!(Conductivity::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_diffusivity() {
    let eta = Diffusivity::<f64>::new(1.0).unwrap();
    assert_eq!(eta.value(), 1.0);
    assert!(Diffusivity::<f64>::new(-1.0).is_err());
    assert!(Diffusivity::<f64>::new(f64::NAN).is_err());
    assert!(Diffusivity::<f64>::new(f64::INFINITY).is_err());
}

// ===========================================================================
// new_unchecked tests
// ===========================================================================

#[test]
fn test_alfven_speed_new_unchecked() {
    let v = AlfvenSpeed::<f64>::new_unchecked(100.0);
    assert_eq!(v.value(), 100.0);
}

#[test]
fn test_plasma_beta_new_unchecked() {
    let beta = PlasmaBeta::<f64>::new_unchecked(0.5);
    assert_eq!(beta.value(), 0.5);
}

#[test]
fn test_magnetic_pressure_new_unchecked() {
    let p = MagneticPressure::<f64>::new_unchecked(1000.0);
    assert_eq!(p.value(), 1000.0);
}

#[test]
fn test_larmor_radius_new_unchecked() {
    let r = LarmorRadius::<f64>::new_unchecked(1.0);
    assert_eq!(r.value(), 1.0);
}

#[test]
fn test_debye_length_new_unchecked() {
    let l = DebyeLength::<f64>::new_unchecked(1e-6);
    assert_eq!(l.value(), 1e-6);
}

#[test]
fn test_plasma_frequency_new_unchecked() {
    let w = PlasmaFrequency::<f64>::new_unchecked(1e9);
    assert_eq!(w.value(), 1e9);
}

#[test]
fn test_conductivity_new_unchecked() {
    let s = Conductivity::<f64>::new_unchecked(1e7);
    assert_eq!(s.value(), 1e7);
}

#[test]
fn test_diffusivity_new_unchecked() {
    let eta = Diffusivity::<f64>::new_unchecked(1.0);
    assert_eq!(eta.value(), 1.0);
}

// ===========================================================================
// Default trait tests
// ===========================================================================

#[test]
fn test_alfven_speed_default() {
    let v: AlfvenSpeed<f64> = Default::default();
    assert_eq!(v.value(), 0.0);
}

#[test]
fn test_plasma_beta_default() {
    let beta: PlasmaBeta<f64> = Default::default();
    assert_eq!(beta.value(), 0.0);
}

#[test]
fn test_magnetic_pressure_default() {
    let p: MagneticPressure<f64> = Default::default();
    assert_eq!(p.value(), 0.0);
}

#[test]
fn test_diffusivity_default() {
    let eta: Diffusivity<f64> = Default::default();
    assert_eq!(eta.value(), 0.0);
}

#[test]
fn test_larmor_radius_default() {
    let r: LarmorRadius<f64> = Default::default();
    assert!(r.value() > 0.0);
}

#[test]
fn test_debye_length_default() {
    let l: DebyeLength<f64> = Default::default();
    assert!(l.value() > 0.0);
}

#[test]
fn test_plasma_frequency_default() {
    let w: PlasmaFrequency<f64> = Default::default();
    assert!(w.value() > 0.0);
}

#[test]
fn test_conductivity_default() {
    let s: Conductivity<f64> = Default::default();
    assert!(s.value() > 0.0);
}

// =============================================================================
// NaN/Infinity validation paths
// =============================================================================

#[test]
fn test_alfven_speed_new_nan_error() {
    assert!(AlfvenSpeed::<f64>::new(f64::NAN).is_err());
    assert!(AlfvenSpeed::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_plasma_beta_new_nan_error() {
    assert!(PlasmaBeta::<f64>::new(f64::NAN).is_err());
    assert!(PlasmaBeta::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_magnetic_pressure_new_nan_error() {
    assert!(MagneticPressure::<f64>::new(f64::NAN).is_err());
    assert!(MagneticPressure::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_larmor_radius_new_nan_error() {
    assert!(LarmorRadius::<f64>::new(f64::NAN).is_err());
    assert!(LarmorRadius::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_debye_length_new_nan_error() {
    assert!(DebyeLength::<f64>::new(f64::NAN).is_err());
    assert!(DebyeLength::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_plasma_frequency_new_nan_error() {
    assert!(PlasmaFrequency::<f64>::new(f64::NAN).is_err());
    assert!(PlasmaFrequency::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_conductivity_new_nan_error() {
    assert!(Conductivity::<f64>::new(f64::NAN).is_err());
    assert!(Conductivity::<f64>::new(f64::INFINITY).is_err());
}

#[test]
fn test_diffusivity_new_nan_error() {
    assert!(Diffusivity::<f64>::new(f64::NAN).is_err());
    assert!(Diffusivity::<f64>::new(f64::INFINITY).is_err());
}

// =============================================================================
// Trait coverage: Debug / Clone / Copy / PartialEq / PartialOrd
// =============================================================================

// =============================================================================
// From<X> for f64 conversion coverage
// =============================================================================

#[test]
fn test_mhd_scalars_into_f64() {
    let v: f64 = AlfvenSpeed::<f64>::new(100.0).unwrap().into();
    assert_eq!(v, 100.0);

    let v: f64 = PlasmaBeta::<f64>::new(0.5).unwrap().into();
    assert_eq!(v, 0.5);

    let v: f64 = MagneticPressure::<f64>::new(1000.0).unwrap().into();
    assert_eq!(v, 1000.0);

    let v: f64 = LarmorRadius::<f64>::new(1.0).unwrap().into();
    assert_eq!(v, 1.0);

    let v: f64 = DebyeLength::<f64>::new(1e-6).unwrap().into();
    assert_eq!(v, 1e-6);

    let v: f64 = PlasmaFrequency::<f64>::new(1e9).unwrap().into();
    assert_eq!(v, 1e9);

    let v: f64 = Conductivity::<f64>::new(1e7).unwrap().into();
    assert_eq!(v, 1e7);

    let v: f64 = Diffusivity::<f64>::new(1.0).unwrap().into();
    assert_eq!(v, 1.0);
}

#[test]
fn test_mhd_scalars_traits() {
    let a = AlfvenSpeed::<f64>::new(1.0).unwrap();
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a, a.clone());
    assert!(a < AlfvenSpeed::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", a);

    let pb = PlasmaBeta::<f64>::new(0.5).unwrap();
    assert!(pb < PlasmaBeta::<f64>::new(1.0).unwrap());
    let _ = format!("{:?}", pb);

    let mp = MagneticPressure::<f64>::new(100.0).unwrap();
    assert!(mp < MagneticPressure::<f64>::new(200.0).unwrap());
    let _ = format!("{:?}", mp);

    let lr = LarmorRadius::<f64>::new(1.0).unwrap();
    assert!(lr < LarmorRadius::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", lr);

    let dl = DebyeLength::<f64>::new(1.0).unwrap();
    assert!(dl < DebyeLength::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", dl);

    let pf = PlasmaFrequency::<f64>::new(1.0).unwrap();
    assert!(pf < PlasmaFrequency::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", pf);

    let c = Conductivity::<f64>::new(1.0).unwrap();
    assert!(c < Conductivity::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", c);

    let d = Diffusivity::<f64>::new(1.0).unwrap();
    assert!(d < Diffusivity::<f64>::new(2.0).unwrap());
    let _ = format!("{:?}", d);
}
