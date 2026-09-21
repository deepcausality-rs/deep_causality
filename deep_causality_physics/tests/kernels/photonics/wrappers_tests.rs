/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    AbcdMatrix, ComplexBeamParameter, IndexOfRefraction, JonesVector, Length, RayAngle, RayHeight,
    StokesVector, Wavelength, beam_spot_size, beam_spot_size_kernel, degree_of_polarization,
    gaussian_q_propagation, grating_equation, jones_rotation, lens_maker, lens_maker_kernel,
    ray_transfer, ray_transfer_kernel, single_slit_irradiance, single_slit_irradiance_kernel,
    snells_law, snells_law_kernel, stokes_from_jones, stokes_from_jones_kernel,
};
use deep_causality_tensor::CausalTensor;

// ============================================================================
// Ray Optics Wrappers
// ============================================================================

#[test]
fn test_wrapper_ray_transfer() {
    // The identity matrix on a default (zero) ray leaves every implementation's answer at zero,
    // so it discriminates nothing. A thin lens of focal length f has ABCD = [[1, 0], [-1/f, 1]],
    // which leaves the height alone and bends the angle by -h/f: with h = 2 and f = 4,
    // (h, a) = (2, 0.1) goes to (2, 0.1 - 0.5) = (2, -0.4).
    let f = 4.0_f64;
    let m = AbcdMatrix::<f64>::new(
        CausalTensor::new(vec![1.0, 0.0, -1.0 / f, 1.0], vec![2, 2]).unwrap(),
    );
    let h = RayHeight::<f64>::new(2.0).unwrap();
    let a = RayAngle::<f64>::new(0.1).unwrap();

    let result = ray_transfer(&m, h, a);
    let (h_out, a_out) = result.value_cloned().unwrap();
    assert!(
        (h_out.value() - 2.0).abs() < 1e-12,
        "height = {}",
        h_out.value()
    );
    assert!(
        (a_out.value() + 0.4).abs() < 1e-12,
        "angle = {}",
        a_out.value()
    );
}

#[test]
fn test_wrapper_ray_transfer_error() {
    let m = AbcdMatrix::<f64>::new(CausalTensor::new(vec![1.0], vec![1]).unwrap());
    let h = RayHeight::<f64>::default();
    let a = RayAngle::<f64>::default();

    let result = ray_transfer(&m, h, a);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Dimension Mismatch"),
        "expected a Dimension Mismatch refusal, got {err}"
    );
}

#[test]
fn test_wrapper_ray_transfer_free_space() {
    // Free space propagation matrix: [[1, d], [0, 1]]
    let d = 0.1; // 10 cm
    let m = AbcdMatrix::<f64>::new(CausalTensor::new(vec![1.0, d, 0.0, 1.0], vec![2, 2]).unwrap());
    let h = RayHeight::<f64>::new(0.01).unwrap(); // 1 cm height
    let a = RayAngle::<f64>::new(0.1).unwrap(); // 0.1 rad angle

    let result = ray_transfer(&m, h, a);
    assert!(result.is_ok());

    if let Some((h_out, _a_out)) = result.value() {
        // h' = h + d*a = 0.01 + 0.1*0.1 = 0.02
        assert!((h_out.value() - 0.02).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_snells_law() {
    let n1 = IndexOfRefraction::<f64>::new(1.0).unwrap();
    let n2 = IndexOfRefraction::<f64>::new(1.5).unwrap();
    let theta1 = RayAngle::<f64>::new(0.3).unwrap();

    let result = snells_law(n1, n2, theta1);
    assert!(result.is_ok());

    if let Some(theta2) = result.value() {
        // n1*sin(theta1) = n2*sin(theta2)
        // sin(theta2) = 1.0*sin(0.3) / 1.5 = 0.1973
        // theta2 ≈ 0.198 rad
        assert!(theta2.value() < theta1.value()); // Refracted towards normal
    }
}

#[test]
fn test_wrapper_snells_law_tir() {
    // Total internal reflection case
    let n1 = IndexOfRefraction::<f64>::new(1.5).unwrap();
    let n2 = IndexOfRefraction::<f64>::new(1.0).unwrap();
    let theta1 = RayAngle::<f64>::new(1.0).unwrap(); // Beyond critical angle

    let result = snells_law(n1, n2, theta1);
    // Should error for TIR
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Physical Invariant Broken"),
        "expected a Physical Invariant Broken refusal, got {err}"
    );
}

#[test]
fn test_wrapper_lens_maker() {
    let n = IndexOfRefraction::<f64>::new(1.5).unwrap();
    let r1 = 0.1; // 10 cm radius
    let r2 = -0.1; // -10 cm (convex-convex)

    let result = lens_maker(n, r1, r2);
    assert!(result.is_ok());

    if let Some(power) = result.value() {
        // P = (n-1)(1/r1 - 1/r2) = 0.5 * (10 - (-10)) = 10 diopters
        assert!(power.value() > 0.0);
    }
}

#[test]
fn test_wrapper_lens_maker_error() {
    let n = IndexOfRefraction::<f64>::new(1.5).unwrap();
    let result = lens_maker(n, 0.0, 0.1);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Singularity"),
        "expected a Singularity refusal, got {err}"
    );
}

// ============================================================================
// Polarization Wrappers
// ============================================================================

#[test]
fn test_wrapper_stokes_from_jones() {
    // Horizontally polarized light: [1, 0]
    let j = JonesVector::<f64>::new(
        CausalTensor::new(
            vec![Complex::new(1.0, 0.0), Complex::new(0.0, 0.0)],
            vec![2],
        )
        .unwrap(),
    );

    let result = stokes_from_jones(&j);
    assert!(result.is_ok());

    if let Some(stokes) = result.value() {
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
fn test_wrapper_stokes_from_jones_error() {
    let j =
        JonesVector::<f64>::new(CausalTensor::new(vec![Complex::new(1.0, 0.0)], vec![1]).unwrap());
    let result = stokes_from_jones(&j);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Dimension Mismatch"),
        "expected a Dimension Mismatch refusal, got {err}"
    );
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
    let angle = RayAngle::<f64>::new(std::f64::consts::PI / 4.0).unwrap(); // 45 degrees

    let result = jones_rotation(&jones, angle);
    assert!(result.is_ok());

    if let Some(rotated) = result.value() {
        assert_eq!(rotated.shape(), &[2, 2]);
    }
}

#[test]
fn test_wrapper_jones_rotation_error() {
    let m = CausalTensor::new(vec![Complex::new(1.0, 0.0)], vec![1]).unwrap();
    let a = RayAngle::<f64>::default();
    let result = jones_rotation(&m, a);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Dimension Mismatch"),
        "expected a Dimension Mismatch refusal, got {err}"
    );
}

#[test]
fn test_wrapper_degree_of_polarization() {
    // Fully polarized light
    let stokes =
        StokesVector::<f64>::new(CausalTensor::new(vec![1.0, 1.0, 0.0, 0.0], vec![4]).unwrap())
            .unwrap();

    let result = degree_of_polarization(&stokes);
    assert!(result.is_ok());

    if let Some(dop) = result.value() {
        // DOP = sqrt(S1² + S2² + S3²) / S0 = 1.0 for fully polarized
        assert!((dop.value() - 1.0).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_degree_of_polarization_partial() {
    // Partially polarized light
    let stokes =
        StokesVector::<f64>::new(CausalTensor::new(vec![1.0, 0.5, 0.0, 0.0], vec![4]).unwrap())
            .unwrap();

    let result = degree_of_polarization(&stokes);
    assert!(result.is_ok());

    if let Some(dop) = result.value() {
        // DOP = 0.5 / 1.0 = 0.5
        assert!((dop.value() - 0.5).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_degree_of_polarization_error() {
    let s =
        StokesVector::<f64>::new(CausalTensor::new(vec![-1.0, 0.0, 0.0, 0.0], vec![4]).unwrap())
            .unwrap();
    let result = degree_of_polarization(&s);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Physical Invariant Broken"),
        "expected a Physical Invariant Broken refusal, got {err}"
    );
}

// ============================================================================
// Gaussian Beam Wrappers
// ============================================================================

#[test]
fn test_wrapper_gaussian_q_propagation() {
    // q = z_R * i at waist
    let q_in = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap();
    // Free space propagation matrix
    let m =
        AbcdMatrix::<f64>::new(CausalTensor::new(vec![1.0, 0.5, 0.0, 1.0], vec![2, 2]).unwrap());

    let result = gaussian_q_propagation(q_in, &m);
    assert!(result.is_ok());

    if let Some(q_out) = result.value() {
        // q' = (A*q + B) / (C*q + D) = (1*i + 0.5) / (0*i + 1) = 0.5 + i
        assert!((q_out.value().re - 0.5).abs() < 1e-10);
        assert!((q_out.value().im - 1.0).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_gaussian_q_propagation_error() {
    let q = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap();
    let m = AbcdMatrix::<f64>::new(CausalTensor::new(vec![1.0], vec![1]).unwrap());
    let result = gaussian_q_propagation(q, &m);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Dimension Mismatch"),
        "expected a Dimension Mismatch refusal, got {err}"
    );
}

#[test]
fn test_wrapper_beam_spot_size() {
    let q = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap();
    let w = Wavelength::<f64>::new(1e-6).unwrap(); // 1 μm

    let result = beam_spot_size(q, w);
    assert!(result.is_ok());

    if let Some(spot) = result.value() {
        // At waist: w0 = sqrt(λ * z_R / π)
        assert!(spot.value() > 0.0);
    }
}

#[test]
fn test_wrapper_beam_spot_size_error() {
    let q = ComplexBeamParameter::<f64>::new_unchecked(Complex::new(1.0, 0.0));
    let w = Wavelength::<f64>::new(1e-6).unwrap();
    let result = beam_spot_size(q, w);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Physical Invariant Broken"),
        "expected a Physical Invariant Broken refusal, got {err}"
    );
}

// ============================================================================
// Diffraction Wrappers
// ============================================================================

#[test]
fn test_wrapper_single_slit_irradiance() {
    let i0 = 1.0; // Initial intensity
    let slit_width = Length::new(1e-4).unwrap(); // 100 μm
    let theta = RayAngle::<f64>::new(0.01).unwrap(); // 0.01 rad
    let wavelength = Wavelength::<f64>::new(500e-9).unwrap(); // 500 nm

    let result = single_slit_irradiance(i0, slit_width, theta, wavelength);
    assert!(result.is_ok());

    if let Some(intensity) = result.value() {
        // Should be less than I0 for non-zero angle
        assert!(*intensity < i0);
        assert!(*intensity >= 0.0);
    }
}

#[test]
fn test_wrapper_single_slit_irradiance_center() {
    let i0 = 1.0;
    let slit_width = Length::new(1e-4).unwrap();
    let theta = RayAngle::<f64>::new(0.0).unwrap(); // Center: maximum
    let wavelength = Wavelength::<f64>::new(500e-9).unwrap();

    let result = single_slit_irradiance(i0, slit_width, theta, wavelength);
    assert!(result.is_ok());

    if let Some(intensity) = result.value() {
        // At center, I = I0
        assert!((intensity - i0).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_single_slit_irradiance_error() {
    let i0 = -1.0;
    let l = Length::new(1.0).unwrap();
    let a = RayAngle::<f64>::default();
    let w = Wavelength::<f64>::new(1e-6).unwrap();
    let result = single_slit_irradiance(i0, l, a, w);
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Physical Invariant Broken"),
        "expected a Physical Invariant Broken refusal, got {err}"
    );
}

#[test]
fn test_wrapper_grating_equation() {
    let pitch = Length::new(1e-6).unwrap(); // 1 μm grating period
    let order = 1; // First order
    let incidence = RayAngle::<f64>::new(0.0).unwrap(); // Normal incidence
    let wavelength = Wavelength::<f64>::new(500e-9).unwrap(); // 500 nm

    let result = grating_equation(pitch, order, incidence, wavelength);
    assert!(result.is_ok());

    if let Some(angle) = result.value() {
        // sin(θ) = m * λ / d = 1 * 500e-9 / 1e-6 = 0.5
        // θ ≈ 0.524 rad
        assert!((angle.value().sin() - 0.5).abs() < 1e-6);
    }
}

#[test]
fn test_wrapper_grating_equation_zero_order() {
    let pitch = Length::new(1e-6).unwrap();
    let order = 0; // Zero order = specular reflection
    let incidence = RayAngle::<f64>::new(0.3).unwrap();
    let wavelength = Wavelength::<f64>::new(500e-9).unwrap();

    let result = grating_equation(pitch, order, incidence, wavelength);
    assert!(result.is_ok());

    if let Some(angle) = result.value() {
        // m=0: sin(θ_out) = sin(θ_in)
        assert!((angle.value() - 0.3).abs() < 1e-10);
    }
}

#[test]
fn test_wrapper_grating_equation_error_evanescent() {
    // High order that would result in evanescent wave
    let pitch = Length::new(1e-6).unwrap();
    let order = 5; // Too high
    let incidence = RayAngle::<f64>::new(0.0).unwrap();
    let wavelength = Wavelength::<f64>::new(800e-9).unwrap();

    let result = grating_equation(pitch, order, incidence, wavelength);
    // sin(θ) = 5 * 800e-9 / 1e-6 = 4.0 > 1, should error
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = result.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Physical Invariant Broken"),
        "expected a Physical Invariant Broken refusal, got {err}"
    );
}

// ============================================================================
// Combined Test (Original)
// ============================================================================

#[test]
fn test_wrappers_combined() {
    // Ray Transfer
    let m = AbcdMatrix::<f64>::new(CausalTensor::identity(&[2, 2]).unwrap());
    let h = RayHeight::<f64>::default();
    let a = RayAngle::<f64>::default();
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = ray_transfer(&m, h, a);
    assert_eq!(
        effect.value_cloned().unwrap(),
        ray_transfer_kernel(&m, h, a).unwrap(),
        "ray_transfer must carry the value its kernel produced"
    );

    // Snells
    let n1 = IndexOfRefraction::<f64>::new(1.0).unwrap();
    let n2 = IndexOfRefraction::<f64>::new(1.5).unwrap();
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = snells_law(n1, n2, a);
    assert_eq!(
        effect.value_cloned().unwrap(),
        snells_law_kernel(n1, n2, a).unwrap(),
        "snells_law must carry the value its kernel produced"
    );

    // Lens
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = lens_maker(n2, 1.0, -1.0);
    assert_eq!(
        effect.value_cloned().unwrap(),
        lens_maker_kernel(n2, 1.0, -1.0).unwrap(),
        "lens_maker must carry the value its kernel produced"
    );

    // Jones
    let j = JonesVector::<f64>::new(
        CausalTensor::new(vec![Complex::new(1.0, 0.0); 2], vec![2]).unwrap(),
    );
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = stokes_from_jones(&j);
    assert_eq!(
        effect.value_cloned().unwrap(),
        stokes_from_jones_kernel(&j).unwrap(),
        "stokes_from_jones must carry the value its kernel produced"
    );

    // Beam
    let q = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap();
    let w = Wavelength::<f64>::new(1e-6).unwrap();
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = beam_spot_size(q, w);
    assert_eq!(
        effect.value_cloned().unwrap(),
        beam_spot_size_kernel(q, w).unwrap(),
        "beam_spot_size must carry the value its kernel produced"
    );

    // Diffraction
    let l = Length::new(1.0).unwrap();
    // Delegation, not merely success: `assert!(x.is_ok())` alone passed even when a wrapper
    // discarded its kernel's answer and returned a constant.
    let effect = single_slit_irradiance(1.0, l, a, w);
    assert_eq!(
        effect.value_cloned().unwrap(),
        single_slit_irradiance_kernel(1.0, l, a, w).unwrap(),
        "single_slit_irradiance must carry the value its kernel produced"
    );
}
