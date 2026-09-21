/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_physics::{
    AbcdMatrix, ComplexBeamParameter, PhysicsErrorEnum, Wavelength, beam_spot_size_kernel,
    gaussian_q_propagation_kernel,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_gaussian_propagation() {
    // Free space d=1. Matrix [1, 1; 0, 1]
    let m_data = vec![1.0, 1.0, 0.0, 1.0];
    let mat = AbcdMatrix::<f64>::new(CausalTensor::new(m_data, vec![2, 2]).unwrap());

    let q_in = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap(); // Waist at z=0, zR=1

    let res = gaussian_q_propagation_kernel(q_in, &mat);
    assert!(res.is_ok());
    let q_out = res.unwrap().value();

    // q_out = (1*i + 1)/(0*i + 1) = 1 + i
    assert!((q_out.re - 1.0).abs() < 1e-10);
    assert!((q_out.im - 1.0).abs() < 1e-10);
}

#[test]
fn test_beam_spot_size() {
    // q = i * zR. zR = pi w0^2 / lambda.
    // Let lambda = pi. Then zR = w0^2. Let w0 = 2. zR = 4.
    // q = 4i.

    let q = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 4.0)).unwrap();
    let lambda = Wavelength::<f64>::new(std::f64::consts::PI).unwrap();

    let res = beam_spot_size_kernel(q, lambda);
    assert!(res.is_ok());
    let w = res.unwrap();
    assert!((w.value() - 2.0).abs() < 1e-10);
}

// ===========================================================================
// Error Path Tests
// ===========================================================================

#[test]
fn test_gaussian_propagation_wrong_matrix_shape() {
    // Matrix must be 2x2, using 3x3 instead
    let m_data = vec![1.0; 9];
    let mat = AbcdMatrix::<f64>::new(CausalTensor::new(m_data, vec![3, 3]).unwrap());

    let q_in = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap();
    let res = gaussian_q_propagation_kernel(q_in, &mat);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::DimensionMismatch { .. }
        ),
        "expected a DimensionMismatch refusal"
    );
}

#[test]
fn test_gaussian_propagation_singularity() {
    // The singularity guard fires when the denominator C q + D has zero norm. Writing
    // q = a + bi, that needs C a + D = 0 and C b = 0 at once. A valid `ComplexBeamParameter`
    // has b > 0, so C must be zero, and then D must be zero too. C = D = 0 is the only
    // singular ABCD matrix reachable from a well-formed q.
    let m_data = vec![1.0, 1.0, 0.0, 0.0]; // A = B = 1, C = D = 0
    let mat = AbcdMatrix::<f64>::new(CausalTensor::new(m_data, vec![2, 2]).unwrap());
    let q_in = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap();
    let res = gaussian_q_propagation_kernel(q_in, &mat);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::Singularity { .. }
        ),
        "expected a Singularity refusal"
    );
}

#[test]
fn test_beam_spot_size_zero_q_error() {
    // q = 0 has norm_sqr() == 0, tripping the Singularity guard in
    // beam_spot_size_kernel (beam.rs:88-90). A valid ComplexBeamParameter
    // requires Im > 0, so we construct the degenerate q via new_unchecked.
    let q = ComplexBeamParameter::<f64>::new_unchecked(Complex::new(0.0, 0.0));
    let lambda = Wavelength::<f64>::new(1.0).unwrap();
    let res = beam_spot_size_kernel(q, lambda);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::Singularity { .. }
        ),
        "expected a Singularity refusal"
    );
}

#[test]
fn test_complex_beam_parameter_new_non_positive_im_error() {
    // Test the ComplexBeamParameter constructor validation
    let res = ComplexBeamParameter::<f64>::new(Complex::new(1.0, 0.0));
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );

    let res2 = ComplexBeamParameter::<f64>::new(Complex::new(1.0, -1.0));
    assert!(
        matches!(
            res2.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_gaussian_propagation_unphysical_output_error() {
    let q_in = ComplexBeamParameter::<f64>::new(Complex::new(0.0, 1.0)).unwrap();
    // Matrix [1, 0, 0, -1] -> q_out = -q_in = -i. Im = -1.
    let m = CausalTensor::new(vec![1.0, 0.0, 0.0, -1.0], vec![2, 2]).unwrap();
    let mat = AbcdMatrix::<f64>::new(m);

    let res = gaussian_q_propagation_kernel(q_in, &mat);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}

#[test]
fn test_beam_spot_size_invalid_q_error() {
    // beam_spot_size_kernel check: if im_inv_q >= 0.0
    // q = z + i z_R. inv_q = (z - i z_R) / (z^2 + z_R^2). Im(inv_q) = -z_R / (z^2 + z_R^2).
    // If z_R is positive, Im(inv_q) is negative.
    // To make Im(inv_q) >= 0, we need z_R <= 0.
    // But ComplexBeamParameter constructor requires z_R > 0.
    // To test this kernel's check, we must use new_unchecked or hit it via logic.
    let q = ComplexBeamParameter::<f64>::new_unchecked(Complex::new(0.0, -1.0));
    let lambda = Wavelength::<f64>::new(1.0).unwrap();
    let res = beam_spot_size_kernel(q, lambda);
    assert!(
        matches!(
            res.as_ref().unwrap_err().0,
            PhysicsErrorEnum::PhysicalInvariantBroken { .. }
        ),
        "expected a PhysicalInvariantBroken refusal"
    );
}
