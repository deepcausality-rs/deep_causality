/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AlfvenSpeed, Conductivity, DebyeLength, Diffusivity, LarmorRadius, MagneticPressure,
    PlasmaBeta, PlasmaFrequency,
};

#[test]
fn test_alfven_speed() {
    let v = AlfvenSpeed::new(100.0).unwrap();
    assert_eq!(v.value(), 100.0);
    assert!(AlfvenSpeed::new(-1.0).is_err());
}

#[test]
fn test_plasma_beta() {
    let beta = PlasmaBeta::new(0.5).unwrap();
    assert_eq!(beta.value(), 0.5);
    assert!(PlasmaBeta::new(-0.1).is_err());
}

#[test]
fn test_magnetic_pressure() {
    let p = MagneticPressure::new(1000.0).unwrap();
    assert_eq!(p.value(), 1000.0);
    assert!(MagneticPressure::new(-10.0).is_err());
}

#[test]
fn test_larmor_radius() {
    let r = LarmorRadius::new(1.0).unwrap();
    assert_eq!(r.value(), 1.0);
    assert!(LarmorRadius::new(0.0).is_err()); // Must be positive
}

#[test]
fn test_debye_length() {
    let l = DebyeLength::new(1e-6).unwrap();
    assert_eq!(l.value(), 1e-6);
    assert!(DebyeLength::new(0.0).is_err());
}

#[test]
fn test_plasma_frequency() {
    let w = PlasmaFrequency::new(1e9).unwrap();
    assert_eq!(w.value(), 1e9);
    assert!(PlasmaFrequency::new(0.0).is_err());
}

#[test]
fn test_conductivity() {
    let s = Conductivity::new(1e7).unwrap();
    assert_eq!(s.value(), 1e7);
    assert!(Conductivity::new(0.0).is_err());
}

#[test]
fn test_diffusivity() {
    let eta = Diffusivity::new(1.0).unwrap();
    assert_eq!(eta.value(), 1.0);
    assert!(Diffusivity::new(-1.0).is_err());
}

// ===========================================================================
// new_unchecked tests
// ===========================================================================

#[test]
fn test_alfven_speed_new_unchecked() {
    let v = AlfvenSpeed::new_unchecked(100.0);
    assert_eq!(v.value(), 100.0);
}

#[test]
fn test_plasma_beta_new_unchecked() {
    let beta = PlasmaBeta::new_unchecked(0.5);
    assert_eq!(beta.value(), 0.5);
}

#[test]
fn test_magnetic_pressure_new_unchecked() {
    let p = MagneticPressure::new_unchecked(1000.0);
    assert_eq!(p.value(), 1000.0);
}

#[test]
fn test_larmor_radius_new_unchecked() {
    let r = LarmorRadius::new_unchecked(1.0);
    assert_eq!(r.value(), 1.0);
}

#[test]
fn test_debye_length_new_unchecked() {
    let l = DebyeLength::new_unchecked(1e-6);
    assert_eq!(l.value(), 1e-6);
}

#[test]
fn test_plasma_frequency_new_unchecked() {
    let w = PlasmaFrequency::new_unchecked(1e9);
    assert_eq!(w.value(), 1e9);
}

#[test]
fn test_conductivity_new_unchecked() {
    let s = Conductivity::new_unchecked(1e7);
    assert_eq!(s.value(), 1e7);
}

#[test]
fn test_diffusivity_new_unchecked() {
    let eta = Diffusivity::new_unchecked(1.0);
    assert_eq!(eta.value(), 1.0);
}

// ===========================================================================
// Default trait tests
// ===========================================================================

#[test]
fn test_alfven_speed_default() {
    let v: AlfvenSpeed = Default::default();
    assert_eq!(v.value(), 0.0);
}

#[test]
fn test_plasma_beta_default() {
    let beta: PlasmaBeta = Default::default();
    assert_eq!(beta.value(), 0.0);
}

#[test]
fn test_magnetic_pressure_default() {
    let p: MagneticPressure = Default::default();
    assert_eq!(p.value(), 0.0);
}

#[test]
fn test_diffusivity_default() {
    let eta: Diffusivity = Default::default();
    assert_eq!(eta.value(), 0.0);
}
