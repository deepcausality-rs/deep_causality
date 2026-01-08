/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_topology::DifferentialForm;

#[test]
fn test_constant_form() {
    let form: DifferentialForm<f64> = DifferentialForm::constant(0, 4, 1.0);
    assert_eq!(form.degree(), 0);
    assert_eq!(form.dim(), 4);
    assert_eq!(form.coefficients().as_slice()[0], 1.0);
    assert!(form.is_scalar());
    assert!(!form.is_covector());
}

#[test]
fn test_from_coefficients() {
    let form = DifferentialForm::from_coefficients(1, 3, vec![1.0, 2.0, 3.0]);
    assert_eq!(form.degree(), 1);
    assert_eq!(form.coefficients().as_slice(), &[1.0, 2.0, 3.0]);
    assert!(form.is_covector());
}

#[test]
fn test_form_add() {
    let a = DifferentialForm::from_coefficients(1, 2, vec![1.0, 2.0]);
    let b = DifferentialForm::from_coefficients(1, 2, vec![3.0, 4.0]);
    let c = a.add(&b);
    assert_eq!(c.coefficients().as_slice(), &[4.0, 6.0]);
}

#[test]
fn test_form_scale() {
    let a = DifferentialForm::from_coefficients(1, 2, vec![1.0, 2.0]);
    let b = a.scale(2.0);
    assert_eq!(b.coefficients().as_slice(), &[2.0, 4.0]);
}

#[test]
fn test_zero_form() {
    let form: DifferentialForm<f64> = DifferentialForm::zero(2, 3); // 3D 2-form
    // Dim 3, degree 2 -> C(3,2) = 3 components
    assert_eq!(form.num_components(), 3);
    assert!(form.coefficients().as_slice().iter().all(|&x| x == 0.0));
}

#[test]
fn test_map() {
    let a = DifferentialForm::from_coefficients(1, 1, vec![1.0, 2.0]);
    let b = a.map(|x| x * 2.0);
    assert_eq!(b.coefficients().as_slice(), &[2.0, 4.0]);
}

#[test]
fn test_from_tensor() {
    // Test the unconstrained constructor
    let tensor = deep_causality_tensor::CausalTensor::from_vec(vec![1, 2, 3], &[3]);
    let form = DifferentialForm::from_tensor(1, 3, tensor);
    assert_eq!(form.coefficients().as_slice(), &[1, 2, 3]);
}
