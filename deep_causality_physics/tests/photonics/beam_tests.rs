/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
