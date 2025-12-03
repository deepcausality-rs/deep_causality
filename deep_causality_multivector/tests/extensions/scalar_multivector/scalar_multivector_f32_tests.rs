/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVectorError, MultiVector};

#[test]
fn test_grade_projection_zero() {
    let scalar = 5.0f32;
    assert_eq!(scalar.grade_projection(0), 5.0f32);
}

#[test]
fn test_grade_projection_non_zero() {
    let scalar = 5.0f32;
    assert_eq!(scalar.grade_projection(1), 0.0f32);
    assert_eq!(scalar.grade_projection(2), 0.0f32);
}

#[test]
fn test_reversion() {
    let scalar = 5.0f32;
    assert_eq!(scalar.reversion(), 5.0f32);
}

#[test]
fn test_squared_magnitude() {
    let scalar = 5.0f32;
    assert_eq!(scalar.squared_magnitude(), 25.0f32);
    let scalar_neg = -3.0f32;
    assert_eq!(scalar_neg.squared_magnitude(), 9.0f32);
}

#[test]
fn test_inverse_non_zero() {
    let scalar = 5.0f32;
    assert_eq!(scalar.inverse().unwrap(), 0.2f32);
    let scalar_neg = -2.0f32;
    assert_eq!(scalar_neg.inverse().unwrap(), -0.5f32);
}

#[test]
fn test_inverse_zero() {
    let scalar = 0.0f32;
    let err = scalar.inverse().unwrap_err();
    assert_eq!(err, CausalMultiVectorError::zero_magnitude());
}

#[test]
fn test_dual() {
    let scalar = 5.0f32;
    assert_eq!(scalar.dual().unwrap(), 5.0f32);
}

#[test]
fn test_geometric_product() {
    let s1 = 2.0f32;
    let s2 = 3.0f32;
    assert_eq!(s1.geometric_product(&s2), 6.0f32);
}

#[test]
fn test_outer_product() {
    let s1 = 2.0f32;
    let s2 = 3.0f32;
    assert_eq!(s1.outer_product(&s2), 6.0f32);
}

#[test]
fn test_inner_product() {
    let s1 = 2.0f32;
    let s2 = 3.0f32;
    assert_eq!(s1.inner_product(&s2), 6.0f32);
}

#[test]
fn test_commutator_lie() {
    let s1 = 2.0f32;
    let s2 = 3.0f32;
    assert_eq!(s1.commutator_lie(&s2), 0.0f32);
}

#[test]
fn test_commutator_geometric() {
    let s1 = 2.0f32;
    let s2 = 3.0f32;
    assert_eq!(s1.commutator_geometric(&s2), 0.0f32);
}

#[test]
fn test_basis_shift() {
    let scalar = 5.0f32;
    assert_eq!(scalar.basis_shift(0), 5.0f32);
    assert_eq!(scalar.basis_shift(10), 5.0f32);
}
