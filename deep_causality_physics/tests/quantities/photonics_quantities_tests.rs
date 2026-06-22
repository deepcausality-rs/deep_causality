/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    AbcdMatrix, BeamWaist, ComplexBeamParameter, FocalLength, JonesVector, NumericalAperture,
    OpticalPower, RayAngle, RayHeight, StokesVector, Wavelength,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_focal_length() {
    let f = FocalLength::<f64>::new(-0.5).unwrap();
    assert_eq!(f.value(), -0.5);
}

#[test]
fn test_optical_power() {
    let p = OpticalPower::<f64>::new(2.0).unwrap();
    assert_eq!(p.value(), 2.0);
}

#[test]
fn test_wavelength() {
    let w = Wavelength::<f64>::new(500e-9).unwrap();
    assert_eq!(w.value(), 500e-9);

    let err = Wavelength::<f64>::new(-1.0);
    assert!(err.is_err());
}

#[test]
fn test_numerical_aperture() {
    let na = NumericalAperture::<f64>::new(0.5).unwrap();
    assert_eq!(na.value(), 0.5);

    let err = NumericalAperture::<f64>::new(0.0);
    assert!(err.is_err());
}

#[test]
fn test_beam_waist() {
    let w0 = BeamWaist::<f64>::new(1e-3).unwrap();
    assert_eq!(w0.value(), 1e-3);

    let err = BeamWaist::<f64>::new(-1.0);
    assert!(err.is_err());
}

#[test]
fn test_complex_beam_parameter() {
    let q = ComplexBeamParameter::<f64>::new(Complex::new(1.0, 2.0)).unwrap();
    assert_eq!(q.value(), Complex::new(1.0, 2.0));

    // Im(q) must be positive
    let err = ComplexBeamParameter::<f64>::new(Complex::new(1.0, -1.0));
    assert!(err.is_err());
}

#[test]
fn test_jones_vector() {
    let data = vec![Complex::new(1.0, 0.0), Complex::new(0.0, 1.0)];
    let t = CausalTensor::new(data, vec![2]).unwrap();
    let j = JonesVector::<f64>::new(t);
    assert_eq!(j.inner().shape(), vec![2]);
}

// ===========================================================================
// new_unchecked tests
// ===========================================================================

#[test]
fn test_wavelength_new_unchecked() {
    let w = Wavelength::<f64>::new_unchecked(500e-9);
    assert_eq!(w.value(), 500e-9);
}

#[test]
fn test_numerical_aperture_new_unchecked() {
    let na = NumericalAperture::<f64>::new_unchecked(0.5);
    assert_eq!(na.value(), 0.5);
}

#[test]
fn test_beam_waist_new_unchecked() {
    let w0 = BeamWaist::<f64>::new_unchecked(1e-3);
    assert_eq!(w0.value(), 1e-3);
}

#[test]
fn test_complex_beam_parameter_new_unchecked() {
    let q = ComplexBeamParameter::<f64>::new_unchecked(Complex::new(1.0, 2.0));
    assert_eq!(q.value(), Complex::new(1.0, 2.0));
}

// ===========================================================================
// Default tests
// ===========================================================================

#[test]
fn test_focal_length_default() {
    let f: FocalLength<f64> = Default::default();
    assert_eq!(f.value(), 0.0);
}

#[test]
fn test_optical_power_default() {
    let p: OpticalPower<f64> = Default::default();
    assert_eq!(p.value(), 0.0);
}

#[test]
fn test_wavelength_default() {
    let w: Wavelength<f64> = Default::default();
    assert_eq!(w.value(), 0.0);
}

#[test]
fn test_numerical_aperture_default() {
    let na: NumericalAperture<f64> = Default::default();
    assert_eq!(na.value(), 0.0);
}

#[test]
fn test_beam_waist_default() {
    let w0: BeamWaist<f64> = Default::default();
    assert_eq!(w0.value(), 0.0);
}

#[test]
fn test_complex_beam_parameter_default() {
    let q: ComplexBeamParameter<f64> = Default::default();
    assert_eq!(q.value(), Complex::new(0.0, 0.0));
}

#[test]
fn test_ray_height_default() {
    let y: RayHeight<f64> = Default::default();
    assert_eq!(y.value(), 0.0);
}

#[test]
fn test_ray_angle_default() {
    let theta: RayAngle<f64> = Default::default();
    assert_eq!(theta.value(), 0.0);
}

#[test]
fn test_abcd_matrix_default() {
    let m: AbcdMatrix<f64> = Default::default();
    assert!(m.inner().is_empty());
}

#[test]
fn test_jones_vector_default() {
    let j: JonesVector<f64> = Default::default();
    assert!(j.inner().is_empty());
}

#[test]
fn test_stokes_vector_default() {
    let s: StokesVector<f64> = Default::default();
    assert!(s.inner().is_empty());
}

#[test]
fn test_ray_height_new() {
    let y = RayHeight::<f64>::new(10.0).unwrap();
    assert_eq!(y.value(), 10.0);
}

#[test]
fn test_ray_angle_new() {
    let a = RayAngle::<f64>::new(0.5).unwrap();
    assert_eq!(a.value(), 0.5);
}

// =============================================================================
// Trait coverage: Debug / Clone / Copy / PartialEq / PartialOrd
// =============================================================================

#[test]
fn test_photonics_scalars_traits() {
    let f = FocalLength::<f64>::new(1.0).unwrap();
    assert_eq!(f, f.clone());
    let _ = format!("{:?}", f);

    let p = OpticalPower::<f64>::new(2.0).unwrap();
    assert_eq!(p, p.clone());
    let _ = format!("{:?}", p);

    let w = Wavelength::<f64>::new(500e-9).unwrap();
    assert_eq!(w, w.clone());
    assert!(w < Wavelength::<f64>::new(600e-9).unwrap());
    let _ = format!("{:?}", w);

    let na = NumericalAperture::<f64>::new(0.5).unwrap();
    assert_eq!(na, na.clone());
    let _ = format!("{:?}", na);

    let bw = BeamWaist::<f64>::new(1e-6).unwrap();
    assert_eq!(bw, bw.clone());
    let _ = format!("{:?}", bw);

    let rh = RayHeight::<f64>::new(0.01).unwrap();
    assert_eq!(rh, rh.clone());
    let _ = format!("{:?}", rh);

    let ra = RayAngle::<f64>::new(0.1).unwrap();
    assert_eq!(ra, ra.clone());
    let _ = format!("{:?}", ra);
}

// =============================================================================
// From<X> for f64 conversions
// photonics/mod.rs:31-33, 57-59, 91-93, 125-127, 159-161, 185-187, 211-213
// =============================================================================

#[test]
fn test_focal_length_into_f64() {
    let v: f64 = FocalLength::<f64>::new(-0.5).unwrap().into();
    assert!((v - (-0.5)).abs() < 1e-10);
}

#[test]
fn test_optical_power_into_f64() {
    let v: f64 = OpticalPower::<f64>::new(2.0).unwrap().into();
    assert!((v - 2.0).abs() < 1e-10);
}

#[test]
fn test_wavelength_into_f64() {
    let v: f64 = Wavelength::<f64>::new(500e-9).unwrap().into();
    assert!((v - 500e-9).abs() < 1e-18);
}

#[test]
fn test_numerical_aperture_into_f64() {
    let v: f64 = NumericalAperture::<f64>::new(0.65).unwrap().into();
    assert!((v - 0.65).abs() < 1e-10);
}

#[test]
fn test_beam_waist_into_f64() {
    let v: f64 = BeamWaist::<f64>::new(1e-3).unwrap().into();
    assert!((v - 1e-3).abs() < 1e-12);
}

#[test]
fn test_ray_height_into_f64() {
    let v: f64 = RayHeight::<f64>::new(0.02).unwrap().into();
    assert!((v - 0.02).abs() < 1e-10);
}

#[test]
fn test_ray_angle_into_f64() {
    let v: f64 = RayAngle::<f64>::new(0.1).unwrap().into();
    assert!((v - 0.1).abs() < 1e-10);
}
