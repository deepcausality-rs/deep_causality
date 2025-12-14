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
