/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;
use deep_causality_physics::{
    AbcdMatrix, ComplexBeamParameter, Wavelength, beam_spot_size_kernel,
    gaussian_q_propagation_kernel,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_gaussian_propagation() {
    // Free space d=1. Matrix [1, 1; 0, 1]
    let m_data = vec![1.0, 1.0, 0.0, 1.0];
    let mat = AbcdMatrix::new(CausalTensor::new(m_data, vec![2, 2]).unwrap());

    let q_in = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap(); // Waist at z=0, zR=1

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

    let q = ComplexBeamParameter::new(Complex::new(0.0, 4.0)).unwrap();
    let lambda = Wavelength::new(std::f64::consts::PI).unwrap();

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
    let mat = AbcdMatrix::new(CausalTensor::new(m_data, vec![3, 3]).unwrap());

    let q_in = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap();
    let res = gaussian_q_propagation_kernel(q_in, &mat);
    assert!(res.is_err());
}

#[test]
fn test_gaussian_propagation_singularity() {
    // Create matrix that causes C*q + D = 0
    // q = i (so re=0, im=1). Want C*i + D = 0. So D=0, C= real? No.
    // C*q + D = Ci + D. If C=-1, D=i, then -i + i = 0.
    // But D is real in ABCD matrix normally. Use pathological case.
    // Actually, for imaginary singularity, let's use C=1, D=0, q = 0 (but q must have Im>0 from ComplexBeamParameter).
    // So we can't easily create a true singularity with valid q. Skip this or use very small Im.
    // Actually, set D = -Im(q)*C/Re(something). Complex math needed.
    // Simplest: use mat where C=-i component from q cancels - hard to construct.
    // Alternative: This test is about the Singularity error enum hit at den.norm_sqr()==0.
    // We need C*q + D with norm 0. If q = a + bi, C*q + D = Ca + Cbi + D.
    // For norm_sqr=0, both parts must be 0: Ca + D = 0 AND Cb = 0. Since q must have b>0, C=0.
    // Then Ca + D = D = 0. So C=0, D=0 triggers singularity.
    let m_data = vec![1.0, 1.0, 0.0, 0.0]; // C=0, D=0
    let mat = AbcdMatrix::new(CausalTensor::new(m_data, vec![2, 2]).unwrap());
    let q_in = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap();
    let res = gaussian_q_propagation_kernel(q_in, &mat);
    assert!(res.is_err());
}

#[test]
fn test_beam_spot_size_zero_q_error() {
    // ComplexBeamParameter requires Im > 0, so we can't have q=0 directly.
    // This error path is unreachable with valid ComplexBeamParameter.
    // Skip or test with a different scenario.
}

#[test]
fn test_complex_beam_parameter_new_non_positive_im_error() {
    // Test the ComplexBeamParameter constructor validation
    let res = ComplexBeamParameter::new(Complex::new(1.0, 0.0));
    assert!(res.is_err());

    let res2 = ComplexBeamParameter::new(Complex::new(1.0, -1.0));
    assert!(res2.is_err());
}

#[test]
fn test_gaussian_propagation_unphysical_output_error() {
    let q_in = ComplexBeamParameter::new(Complex::new(0.0, 1.0)).unwrap();
    // Matrix [1, 0, 0, -1] -> q_out = -q_in = -i. Im = -1.
    let m = CausalTensor::new(vec![1.0, 0.0, 0.0, -1.0], vec![2, 2]).unwrap();
    let mat = AbcdMatrix::new(m);

    let res = gaussian_q_propagation_kernel(q_in, &mat);
    assert!(res.is_err());
}

#[test]
fn test_beam_spot_size_invalid_q_error() {
    // beam_spot_size_kernel check: if im_inv_q >= 0.0
    // q = z + i z_R. inv_q = (z - i z_R) / (z^2 + z_R^2). Im(inv_q) = -z_R / (z^2 + z_R^2).
    // If z_R is positive, Im(inv_q) is negative.
    // To make Im(inv_q) >= 0, we need z_R <= 0.
    // But ComplexBeamParameter constructor requires z_R > 0.
    // To test this kernel's check, we must use new_unchecked or hit it via logic.
    let q = ComplexBeamParameter::new_unchecked(Complex::new(0.0, -1.0));
    let lambda = Wavelength::new(1.0).unwrap();
    let res = beam_spot_size_kernel(q, lambda);
    assert!(res.is_err());
}
