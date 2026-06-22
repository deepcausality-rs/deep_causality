/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::DifferentialForm;

// `DifferentialForm::new` delegates to `from_tensor`; exercise it directly so
// the public constructor body is covered.
#[test]
fn test_new_delegates_to_from_tensor() {
    let tensor = CausalTensor::from_vec(vec![1.0, 2.0, 3.0], &[3]);
    let form: DifferentialForm<f64> = DifferentialForm::new(1, 3, tensor);

    assert_eq!(form.degree(), 1);
    assert_eq!(form.dim(), 3);
    assert_eq!(form.coefficients().as_slice(), &[1.0, 2.0, 3.0]);
}

// `coefficients_mut` returns a mutable handle to the coefficient tensor.
#[test]
fn test_coefficients_mut_returns_mutable_handle() {
    let tensor = CausalTensor::from_vec(vec![1.0, 2.0], &[2]);
    let mut form: DifferentialForm<f64> = DifferentialForm::new(1, 2, tensor);

    let coeffs = form.coefficients_mut();
    // The returned reference points at the same tensor the form holds.
    assert_eq!(coeffs.as_slice(), &[1.0, 2.0]);
    assert_eq!(form.coefficients().len(), 2);
}
