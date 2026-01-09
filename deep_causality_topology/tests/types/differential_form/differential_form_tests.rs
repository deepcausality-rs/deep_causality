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

// =============================================================================
// Constructor coverage
// =============================================================================

#[test]
fn test_from_generator() {
    let form: DifferentialForm<f64> = DifferentialForm::from_generator(1, 3, |i| (i + 1) as f64);
    // 1-form in 3D has 3 components
    assert_eq!(form.num_components(), 3);
    assert_eq!(form.coefficients().as_slice(), &[1.0, 2.0, 3.0]);
}

#[test]
fn test_zero_form_high_degree() {
    // k-form where k > dim should have 0 components via binomial
    let form: DifferentialForm<f64> = DifferentialForm::zero(5, 3); // 5-form in 3D
    // C(3, 5) = 0, but we use max(1) so it has 1 component
    assert!(!form.coefficients().is_empty());
}

// =============================================================================
// Getter coverage
// =============================================================================

#[test]
fn test_is_top_form() {
    let not_top: DifferentialForm<f64> = DifferentialForm::constant(1, 3, 1.0);
    assert!(!not_top.is_top_form(), "1-form in 3D is not top form");

    let top: DifferentialForm<f64> = DifferentialForm::constant(3, 3, 1.0);
    assert!(top.is_top_form(), "3-form in 3D is top form");
}

#[test]
fn test_num_components_various() {
    // 0-form in any dimension: C(n, 0) = 1
    let scalar: DifferentialForm<f64> = DifferentialForm::constant(0, 4, 1.0);
    assert_eq!(scalar.num_components(), 1);

    // 1-form in n dimensions: C(n, 1) = n
    let covector: DifferentialForm<f64> = DifferentialForm::constant(1, 5, 1.0);
    assert_eq!(covector.num_components(), 5);

    // 2-form in 4D: C(4, 2) = 6
    let two_form: DifferentialForm<f64> = DifferentialForm::constant(2, 4, 1.0);
    assert_eq!(two_form.num_components(), 6);
}

#[test]
fn test_get_valid_index() {
    let form = DifferentialForm::from_coefficients(1, 3, vec![10.0, 20.0, 30.0]);

    assert_eq!(form.get(0), Some(&10.0));
    assert_eq!(form.get(1), Some(&20.0));
    assert_eq!(form.get(2), Some(&30.0));
}

#[test]
fn test_get_out_of_bounds() {
    let form = DifferentialForm::from_coefficients(1, 2, vec![1.0, 2.0]);

    assert_eq!(form.get(5), None, "Out of bounds should return None");
}

// Note: coefficients_mut test removed - CausalTensor doesn't expose as_slice_mut

// =============================================================================
// Operation edge cases
// =============================================================================

#[test]
fn test_map_type_conversion() {
    let float_form = DifferentialForm::from_coefficients(1, 2, vec![1.0, 2.0]);

    // Map to integers (rounded)
    let int_form: DifferentialForm<i32> = float_form.map(|x| *x as i32);

    assert_eq!(int_form.coefficients().as_slice(), &[1, 2]);
}

#[test]
#[should_panic(expected = "Form degrees must match")]
fn test_add_mismatched_degree() {
    let a = DifferentialForm::from_coefficients(0, 2, vec![1.0]);
    let b = DifferentialForm::from_coefficients(1, 2, vec![1.0, 2.0]);

    let _ = a.add(&b); // Should panic
}

#[test]
#[should_panic(expected = "Form dimensions must match")]
fn test_add_mismatched_dim() {
    let a = DifferentialForm::from_coefficients(1, 2, vec![1.0, 2.0]);
    let b = DifferentialForm::from_coefficients(1, 3, vec![1.0, 2.0, 3.0]);

    let _ = a.add(&b); // Should panic
}

#[test]
fn test_scale_zero() {
    let form = DifferentialForm::from_coefficients(1, 2, vec![5.0, 10.0]);
    let scaled = form.scale(0.0);

    assert!(scaled.coefficients().as_slice().iter().all(|&x| x == 0.0));
}
