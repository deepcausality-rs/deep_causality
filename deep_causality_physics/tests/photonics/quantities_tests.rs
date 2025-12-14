/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    BeamWaist, ComplexBeamParameter, FocalLength, JonesVector, NumericalAperture, OpticalPower,
    Wavelength,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_focal_length() {
    let f = FocalLength::new(-0.5).unwrap();
    assert_eq!(f.value(), -0.5);
}

#[test]
fn test_optical_power() {
    let p = OpticalPower::new(2.0).unwrap();
    assert_eq!(p.value(), 2.0);
}

#[test]
fn test_wavelength() {
    let w = Wavelength::new(500e-9).unwrap();
    assert_eq!(w.value(), 500e-9);

    let err = Wavelength::new(-1.0);
    assert!(err.is_err());
}

#[test]
fn test_numerical_aperture() {
    let na = NumericalAperture::new(0.5).unwrap();
    assert_eq!(na.value(), 0.5);

    let err = NumericalAperture::new(0.0);
    assert!(err.is_err());
}

#[test]
fn test_beam_waist() {
    let w0 = BeamWaist::new(1e-3).unwrap();
    assert_eq!(w0.value(), 1e-3);

    let err = BeamWaist::new(-1.0);
    assert!(err.is_err());
}

#[test]
fn test_complex_beam_parameter() {
    let q = ComplexBeamParameter::new(Complex::new(1.0, 2.0)).unwrap();
    assert_eq!(q.value(), Complex::new(1.0, 2.0));

    // Im(q) must be positive
    let err = ComplexBeamParameter::new(Complex::new(1.0, -1.0));
    assert!(err.is_err());
}

#[test]
fn test_jones_vector() {
    let data = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 1.0)];
    let t = CausalTensor::new(data, vec![2]).unwrap();
    let j = JonesVector::new(t);
    assert_eq!(j.inner().shape(), vec![2]);
}

// ===========================================================================
// new_unchecked tests
// ===========================================================================

#[test]
fn test_wavelength_new_unchecked() {
    let w = Wavelength::new_unchecked(500e-9);
    assert_eq!(w.value(), 500e-9);
}

#[test]
fn test_numerical_aperture_new_unchecked() {
    let na = NumericalAperture::new_unchecked(0.5);
    assert_eq!(na.value(), 0.5);
}

#[test]
fn test_beam_waist_new_unchecked() {
    let w0 = BeamWaist::new_unchecked(1e-3);
    assert_eq!(w0.value(), 1e-3);
}

#[test]
fn test_complex_beam_parameter_new_unchecked() {
    let q = ComplexBeamParameter::new_unchecked(Complex::new(1.0, 2.0));
    assert_eq!(q.value(), Complex::new(1.0, 2.0));
}

// ===========================================================================
// Default tests
// ===========================================================================

#[test]
fn test_focal_length_default() {
    let f: FocalLength = Default::default();
    assert_eq!(f.value(), 0.0);
}

#[test]
fn test_optical_power_default() {
    let p: OpticalPower = Default::default();
    assert_eq!(p.value(), 0.0);
}

#[test]
fn test_wavelength_default() {
    let w: Wavelength = Default::default();
    assert_eq!(w.value(), 0.0);
}

#[test]
fn test_numerical_aperture_default() {
    let na: NumericalAperture = Default::default();
    assert_eq!(na.value(), 0.0);
}

#[test]
fn test_beam_waist_default() {
    let w0: BeamWaist = Default::default();
    assert_eq!(w0.value(), 0.0);
}

#[test]
fn test_complex_beam_parameter_default() {
    let q: ComplexBeamParameter = Default::default();
    assert_eq!(q.value(), Complex::new(0.0, 0.0));
}
