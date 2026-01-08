/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};

#[test]
fn test_grade_projection() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    // 1 (gr0) + 2e1 (gr1) + 3e2 (gr1) + 4e12 (gr2)

    let g0 = v.grade_projection(0);
    assert_eq!(g0.data()[0], 1.0);
    assert_eq!(g0.data()[1], 0.0);

    let g1 = v.grade_projection(1);
    assert_eq!(g1.data()[1], 2.0);
    assert_eq!(g1.data()[2], 3.0);
    assert_eq!(g1.data()[0], 0.0);
}

#[test]
fn test_reversion() {
    let m = Metric::Euclidean(2);
    // A = 1 + 2e1 + 3e2 + 4e12
    // ~A = 1 + 2e1 + 3e2 - 4e12 (bivectors reverse sign)
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let rev = v.reversion();
    assert_eq!(rev.data(), &vec![1.0, 2.0, 3.0, -4.0]);
}

#[test]
fn test_squared_magnitude() {
    let m = Metric::Euclidean(2);
    // e1 -> mag^2 = 1
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    assert_eq!(e1.squared_magnitude(), 1.0);

    // e1 + e2 -> (e1+e2)(e1+e2) = 1 + e1e2 + e2e1 + 1 = 2
    let v = CausalMultiVector::new(vec![0.0, 1.0, 1.0, 0.0], m).unwrap();
    assert_eq!(v.squared_magnitude(), 2.0);
}

#[test]
fn test_inverse() {
    let m = Metric::Euclidean(2);
    // e1^-1 = e1 (since e1^2 = 1)
    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let inv = e1.inverse().unwrap();
    assert_eq!(inv.data(), e1.data());

    // Zero vector -> error
    let zero = CausalMultiVector::scalar(0.0, m);
    assert!(zero.inverse().is_err());
}

#[test]
fn test_dual() {
    let m = Metric::Euclidean(2);
    // I = e12
    // e1 * = e1 * I^-1 = e1 * (-e12) = -e1 e1 e2 = -e2
    // Wait: I = e1e2. I^2 = -1. I^-1 = -I = -e1e2.
    // e1 * (-e1e2) = -e1 e1 e2 = -e2.

    let e1 = CausalMultiVector::new(vec![0.0, 1.0, 0.0, 0.0], m).unwrap();
    let dual = e1.dual().unwrap();

    // e2 is index 2. -e2 means index 2 is -1.
    assert_eq!(dual.data()[2], -1.0);
}

#[test]
fn test_basis_shift_no_shift() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    let shifted = v.basis_shift(0);
    assert_eq!(shifted.data(), &vec![1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_basis_shift_with_shift() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    // Shift by 1: [1, 2, 3, 4] -> [2, 3, 4, 1]
    let shifted = v.basis_shift(1);
    assert_eq!(shifted.data(), &vec![2.0, 3.0, 4.0, 1.0]);

    // Shift by 2: [1, 2, 3, 4] -> [3, 4, 1, 2]
    let shifted = v.basis_shift(2);
    assert_eq!(shifted.data(), &vec![3.0, 4.0, 1.0, 2.0]);
}

#[test]
fn test_basis_shift_wrap_around() {
    let m = Metric::Euclidean(2);
    let v = CausalMultiVector::new(vec![1.0, 2.0, 3.0, 4.0], m).unwrap();
    // Shift by 4 (length of vector): [1, 2, 3, 4] -> [1, 2, 3, 4]
    let shifted = v.basis_shift(4);
    assert_eq!(shifted.data(), &vec![1.0, 2.0, 3.0, 4.0]);

    // Shift by 5: same as shift by 1
    let shifted = v.basis_shift(5);
    assert_eq!(shifted.data(), &vec![2.0, 3.0, 4.0, 1.0]);
}
