/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::EffectValue;
use deep_causality_num::Complex;
use deep_causality_physics::{
    beam_spot_size, degree_of_polarization, gaussian_q_propagation, grating_equation,
    jones_rotation, lens_maker, ray_transfer, single_slit_irradiance, snells_law, stokes_from_jones,
    AbcdMatrix, ComplexBeamParameter, IndexOfRefraction, JonesVector, Length, RayAngle, RayHeight,
    StokesVector, Wavelength,
};
use deep_causality_tensor::CausalTensor;

// ============================================================================
// Ray Optics Wrappers
// ============================================================================

#[test]
fn test_wrapper_ray_transfer() {
    let m = AbcdMatrix::new(CausalTensor::identity(&[2, 2]).unwrap());
    let h = RayHeight::default();
    let a = RayAngle::default();

    let result = ray_transfer(&m, h, a);
    assert!(result.is_ok());
}

#[test]
fn test_wrapper_ray_transfer_free_space() {
    // Free space propagation matrix: [[1, d], [0, 1]]
    let d = 0.1; // 10 cm
    let m = AbcdMatrix::new(CausalTensor::new(vec![1.0, d, 0.0, 1.0], vec![2, 2]).unwrap());
    let h = RayHeight::new(0.01).unwrap(); // 1 cm height
    let a = RayAngle::new(0.1).unwrap(); // 0.1 rad angle

    let result = ray_transfer(&m, h, a);
    assert!(result.is_ok());

    if let EffectValue::Value((h_out, _a_out)) = result.value() {
        // h' = h + d*a = 0.01 + 0.1*0.1 = 0.02
        assert!((h_out.value() - 0.02).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_snells_law() {
    let n1 = IndexOfRefraction::new(1.0).unwrap();
    let n2 = IndexOfRefraction::new(1.5).unwrap();
    let theta1 = RayAngle::new(0.3).unwrap();

    let result = snells_law(n1, n2, theta1);
    assert!(result.is_ok());

    if let EffectValue::Value(theta2) = result.value() {
        // n1*sin(theta1) = n2*sin(theta2)
        // sin(theta2) = 1.0*sin(0.3) / 1.5 = 0.1973
        // theta2 ≈ 0.198 rad
        assert!(theta2.value() < theta1.value()); // Refracted towards normal
    }
}

#[test]
fn test_wrapper_snells_law_tir() {
    // Total internal reflection case
    let n1 = IndexOfRefraction::new(1.5).unwrap();
    let n2 = IndexOfRefraction::new(1.0).unwrap();
    let theta1 = RayAngle::new(1.0).unwrap(); // Beyond critical angle

    let result = snells_law(n1, n2, theta1);
    // Should error for TIR
    assert!(result.is_err());
}

#[test]
fn test_wrapper_lens_maker() {
    let n = IndexOfRefraction::new(1.5).unwrap();
    let r1 = 0.1; // 10 cm radius
    let r2 = -0.1; // -10 cm (convex-convex)

    let result = lens_maker(n, r1, r2);
    assert!(result.is_ok());

    if let EffectValue::Value(power) = result.value() {
        // P = (n-1)(1/r1 - 1/r2) = 0.5 * (10 - (-10)) = 10 diopters
        assert!(power.value() > 0.0);
    }
}

// ============================================================================
// Polarization Wrappers
// ============================================================================

#[test]
fn test_wrapper_stokes_from_jones() {
    // Horizontally polarized light: [1, 0]
    let j = JonesVector::new(CausalTensor::new(vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)], vec![2]).unwrap());

    let result = stokes_from_jones(&j);
    assert!(result.is_ok());

    if let EffectValue::Value(stokes) = result.value() {
        let s = stokes.inner();
        // S0 = |Ex|² + |Ey|² = 1
        // S1 = |Ex|² - |Ey|² = 1
        // S2 = 2*Re(Ex*Ey*) = 0
        // S3 = -2*Im(Ex*Ey*) = 0
        assert!((s.data()[0] - 1.0).abs() < 1e-10); // S0
        assert!((s.data()[1] - 1.0).abs() < 1e-10); // S1
    }
}

#[test]
fn test_wrapper_jones_rotation() {
    // Identity Jones matrix
    let jones = CausalTensor::new(
        vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(0.0, 0.0),
            Complex::new(1.0, 0.0),
        ],
        vec![2, 2],
    )
    .unwrap();
    let angle = RayAngle::new(std::f64::consts::PI / 4.0).unwrap(); // 45 degrees

    let result = jones_rotation(&jones, angle);
    assert!(result.is_ok());

    if let EffectValue::Value(rotated) = result.value() {
        assert_eq!(rotated.shape(), &[2, 2]);
    }
}

#[test]
fn test_wrapper_degree_of_polarization() {
    // Fully polarized light
    let stokes = StokesVector::new(CausalTensor::new(vec![1.0, 1.0, 0.0, 0.0], vec![4]).unwrap());

    let result = degree_of_polarization(&stokes);
    assert!(result.is_ok());

    if let EffectValue::Value(dop) = result.value() {
        // DOP = sqrt(S1² + S2² + S3²) / S0 = 1.0 for fully polarized
        assert!((dop.value() - 1.0).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_degree_of_polarization_partial() {
    // Partially polarized light
    let stokes = StokesVector::new(CausalTensor::new(vec![1.0, 0.5, 0.0, 0.0], vec![4]).unwrap());

    let result = degree_of_polarization(&stokes);
    assert!(result.is_ok());

    if let EffectValue::Value(dop) = result.value() {
        // DOP = 0.5 / 1.0 = 0.5
        assert!((dop.value() - 0.5).abs() < 1e-10);
    }
}

// ============================================================================
// Gaussian Beam Wrappers
// ============================================================================

#[test]
fn test_wrapper_gaussian_q_propagation() {
    // q = z_R * i at waist
    let q_in = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap();
    // Free space propagation matrix
    let m = AbcdMatrix::new(CausalTensor::new(vec![1.0, 0.5, 0.0, 1.0], vec![2, 2]).unwrap());

    let result = gaussian_q_propagation(q_in, &m);
    assert!(result.is_ok());

    if let EffectValue::Value(q_out) = result.value() {
        // q' = (A*q + B) / (C*q + D) = (1*i + 0.5) / (0*i + 1) = 0.5 + i
        assert!((q_out.value().re - 0.5).abs() < 1e-10);
        assert!((q_out.value().im - 1.0).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_beam_spot_size() {
    let q = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap();
    let w = Wavelength::new(1e-6).unwrap(); // 1 μm

    let result = beam_spot_size(q, w);
    assert!(result.is_ok());

    if let EffectValue::Value(spot) = result.value() {
        // At waist: w0 = sqrt(λ * z_R / π)
        assert!(spot.value() > 0.0);
    }
}

// ============================================================================
// Diffraction Wrappers
// ============================================================================

#[test]
fn test_wrapper_single_slit_irradiance() {
    let i0 = 1.0; // Initial intensity
    let slit_width = Length::new(1e-4).unwrap(); // 100 μm
    let theta = RayAngle::new(0.01).unwrap(); // 0.01 rad
    let wavelength = Wavelength::new(500e-9).unwrap(); // 500 nm

    let result = single_slit_irradiance(i0, slit_width, theta, wavelength);
    assert!(result.is_ok());

    if let EffectValue::Value(intensity) = result.value() {
        // Should be less than I0 for non-zero angle
        assert!(*intensity < i0);
        assert!(*intensity >= 0.0);
    }
}

#[test]
fn test_wrapper_single_slit_irradiance_center() {
    let i0 = 1.0;
    let slit_width = Length::new(1e-4).unwrap();
    let theta = RayAngle::new(0.0).unwrap(); // Center: maximum
    let wavelength = Wavelength::new(500e-9).unwrap();

    let result = single_slit_irradiance(i0, slit_width, theta, wavelength);
    assert!(result.is_ok());

    if let EffectValue::Value(intensity) = result.value() {
        // At center, I = I0
        assert!((intensity - i0).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_grating_equation() {
    let pitch = Length::new(1e-6).unwrap(); // 1 μm grating period
    let order = 1; // First order
    let incidence = RayAngle::new(0.0).unwrap(); // Normal incidence
    let wavelength = Wavelength::new(500e-9).unwrap(); // 500 nm

    let result = grating_equation(pitch, order, incidence, wavelength);
    assert!(result.is_ok());

    if let EffectValue::Value(angle) = result.value() {
        // sin(θ) = m * λ / d = 1 * 500e-9 / 1e-6 = 0.5
        // θ ≈ 0.524 rad
        assert!((angle.value().sin() - 0.5).abs() < 1e-6);
    }
}

#[test]
fn test_wrapper_grating_equation_zero_order() {
    let pitch = Length::new(1e-6).unwrap();
    let order = 0; // Zero order = specular reflection
    let incidence = RayAngle::new(0.3).unwrap();
    let wavelength = Wavelength::new(500e-9).unwrap();

    let result = grating_equation(pitch, order, incidence, wavelength);
    assert!(result.is_ok());

    if let EffectValue::Value(angle) = result.value() {
        // m=0: sin(θ_out) = sin(θ_in)
        assert!((angle.value() - 0.3).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_grating_equation_error_evanescent() {
    // High order that would result in evanescent wave
    let pitch = Length::new(1e-6).unwrap();
    let order = 5; // Too high
    let incidence = RayAngle::new(0.0).unwrap();
    let wavelength = Wavelength::new(800e-9).unwrap();

    let result = grating_equation(pitch, order, incidence, wavelength);
    // sin(θ) = 5 * 800e-9 / 1e-6 = 4.0 > 1, should error
    assert!(result.is_err());
}

// ============================================================================
// Combined Test (Original)
// ============================================================================

#[test]
fn test_wrappers_combined() {
    // Ray Transfer
    let m = AbcdMatrix::new(CausalTensor::identity(&[2, 2]).unwrap());
    let h = RayHeight::default();
    let a = RayAngle::default();
    assert!(ray_transfer(&m, h, a).is_ok());

    // Snells
    let n1 = IndexOfRefraction::new(1.0).unwrap();
    let n2 = IndexOfRefraction::new(1.5).unwrap();
    assert!(snells_law(n1, n2, a).is_ok());

    // Lens
    assert!(lens_maker(n2, 1.0, -1.0).is_ok());

    // Jones
    let j = JonesVector::new(CausalTensor::new(vec![Complex::new(1.0, 0.0); 2], vec![2]).unwrap());
    assert!(stokes_from_jones(&j).is_ok());

    // Beam
    let q = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap();
    let w = Wavelength::new(1e-6).unwrap();
    assert!(beam_spot_size(q, w).is_ok());

    // Diffraction
    let l = Length::new(1.0).unwrap();
    assert!(single_slit_irradiance(1.0, l, a, w).is_ok());
}

