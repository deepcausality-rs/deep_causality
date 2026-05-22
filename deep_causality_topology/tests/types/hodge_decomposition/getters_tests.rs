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

fn sample() -> HodgeDecomposition<f64> {
    HodgeDecomposition::new(
        tensor_1d(vec![1.0, 2.0]),
        tensor_1d(vec![3.0, 4.0]),
        tensor_1d(vec![5.0, 6.0]),
        2,
    )
}

#[test]
fn test_exact_returns_borrowed_view() {
    let decomposition = sample();
    let view: &CausalTensor<f64> = decomposition.exact();
    assert_eq!(view.as_slice(), &[1.0, 2.0]);
}

#[test]
fn test_co_exact_returns_borrowed_view() {
    let decomposition = sample();
    let view: &CausalTensor<f64> = decomposition.co_exact();
    assert_eq!(view.as_slice(), &[3.0, 4.0]);
}

#[test]
fn test_harmonic_returns_borrowed_view() {
    let decomposition = sample();
    let view: &CausalTensor<f64> = decomposition.harmonic();
    assert_eq!(view.as_slice(), &[5.0, 6.0]);
}

#[test]
fn test_grade_returns_value() {
    let decomposition = sample();
    assert_eq!(decomposition.grade(), 2);
}

#[test]
fn test_getters_return_stable_references_across_multiple_calls() {
    let decomposition = sample();
    let view_a = decomposition.exact() as *const _;
    let view_b = decomposition.exact() as *const _;
    assert_eq!(view_a, view_b);
}

#[test]
fn test_getters_match_construction_order() {
    let a = tensor_1d(vec![10.0]);
    let b = tensor_1d(vec![20.0]);
    let c = tensor_1d(vec![30.0]);
    let decomposition = HodgeDecomposition::new(a, b, c, 7);

    assert_eq!(decomposition.exact().as_slice(), &[10.0]);
    assert_eq!(decomposition.co_exact().as_slice(), &[20.0]);
    assert_eq!(decomposition.harmonic().as_slice(), &[30.0]);
    assert_eq!(decomposition.grade(), 7);
}
