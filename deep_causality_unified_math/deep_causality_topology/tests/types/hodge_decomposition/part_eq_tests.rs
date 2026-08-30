/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;
use deep_causality_topology::HodgeDecomposition;

fn tensor_1d(data: Vec<f64>) -> CausalTensor<f64> {
    let n = data.len();
    CausalTensor::new(data, vec![n]).unwrap()
}

fn build(
    exact: Vec<f64>,
    co_exact: Vec<f64>,
    harmonic: Vec<f64>,
    grade: usize,
) -> HodgeDecomposition<f64> {
    HodgeDecomposition::new(
        tensor_1d(exact),
        tensor_1d(co_exact),
        tensor_1d(harmonic),
        grade,
    )
}

#[test]
fn test_equality_reflexivity() {
    let decomposition = build(vec![1.0, 2.0], vec![3.0], vec![4.0], 1);
    assert_eq!(decomposition, decomposition);
}

#[test]
fn test_equality_symmetry() {
    let a = build(vec![1.0], vec![2.0], vec![3.0], 1);
    let b = build(vec![1.0], vec![2.0], vec![3.0], 1);
    assert_eq!(a, b);
    assert_eq!(b, a);
}

#[test]
fn test_equality_transitivity() {
    let a = build(vec![1.0], vec![2.0], vec![3.0], 1);
    let b = build(vec![1.0], vec![2.0], vec![3.0], 1);
    let c = build(vec![1.0], vec![2.0], vec![3.0], 1);
    assert_eq!(a, b);
    assert_eq!(b, c);
    assert_eq!(a, c);
}

#[test]
fn test_inequality_when_grade_differs() {
    let a = build(vec![1.0], vec![2.0], vec![3.0], 1);
    let b = build(vec![1.0], vec![2.0], vec![3.0], 2);
    assert_ne!(a, b);
}

#[test]
fn test_inequality_when_exact_differs() {
    let a = build(vec![1.0], vec![2.0], vec![3.0], 1);
    let b = build(vec![9.0], vec![2.0], vec![3.0], 1);
    assert_ne!(a, b);
}

#[test]
fn test_inequality_when_co_exact_differs() {
    let a = build(vec![1.0], vec![2.0], vec![3.0], 1);
    let b = build(vec![1.0], vec![9.0], vec![3.0], 1);
    assert_ne!(a, b);
}

#[test]
fn test_inequality_when_harmonic_differs() {
    let a = build(vec![1.0], vec![2.0], vec![3.0], 1);
    let b = build(vec![1.0], vec![2.0], vec![9.0], 1);
    assert_ne!(a, b);
}

#[test]
fn test_inequality_when_dimensions_differ() {
    let a = build(vec![1.0, 2.0], vec![3.0], vec![4.0], 1);
    let b = build(vec![1.0], vec![3.0], vec![4.0], 1);
    assert_ne!(a, b);
}

#[test]
fn test_equality_at_f32_precision() {
    let a: HodgeDecomposition<f32> = HodgeDecomposition::new(
        CausalTensor::new(vec![1.0f32], vec![1]).unwrap(),
        CausalTensor::new(vec![2.0f32], vec![1]).unwrap(),
        CausalTensor::new(vec![3.0f32], vec![1]).unwrap(),
        1,
    );
    let b: HodgeDecomposition<f32> = HodgeDecomposition::new(
        CausalTensor::new(vec![1.0f32], vec![1]).unwrap(),
        CausalTensor::new(vec![2.0f32], vec![1]).unwrap(),
        CausalTensor::new(vec![3.0f32], vec![1]).unwrap(),
        1,
    );
    assert_eq!(a, b);
}
