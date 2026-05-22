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

#[test]
fn test_new_constructs_with_prescribed_components() {
    let exact = tensor_1d(vec![1.0, 2.0, 3.0]);
    let co_exact = tensor_1d(vec![0.5, 0.25, 0.125]);
    let harmonic = tensor_1d(vec![0.0, 0.0, 0.0]);
    let grade = 1;

    let decomposition =
        HodgeDecomposition::new(exact.clone(), co_exact.clone(), harmonic.clone(), grade);

    assert_eq!(decomposition.exact(), &exact);
    assert_eq!(decomposition.co_exact(), &co_exact);
    assert_eq!(decomposition.harmonic(), &harmonic);
    assert_eq!(decomposition.grade(), grade);
}

#[test]
fn test_new_accepts_grade_zero() {
    let zero_tensor = tensor_1d(vec![0.0]);
    let decomposition =
        HodgeDecomposition::new(zero_tensor.clone(), zero_tensor.clone(), zero_tensor, 0);
    assert_eq!(decomposition.grade(), 0);
}

#[test]
fn test_new_accepts_large_grade_value() {
    let zero_tensor = tensor_1d(vec![0.0]);
    let decomposition = HodgeDecomposition::new(
        zero_tensor.clone(),
        zero_tensor.clone(),
        zero_tensor,
        usize::MAX,
    );
    assert_eq!(decomposition.grade(), usize::MAX);
}

#[test]
fn test_new_clone_is_independent() {
    let exact = tensor_1d(vec![1.0]);
    let co_exact = tensor_1d(vec![2.0]);
    let harmonic = tensor_1d(vec![3.0]);
    let decomposition = HodgeDecomposition::new(exact, co_exact, harmonic, 1);

    let cloned = decomposition.clone();
    assert_eq!(cloned.exact(), decomposition.exact());
    assert_eq!(cloned.co_exact(), decomposition.co_exact());
    assert_eq!(cloned.harmonic(), decomposition.harmonic());
    assert_eq!(cloned.grade(), decomposition.grade());
}

#[test]
fn test_new_at_f32_precision() {
    let exact: CausalTensor<f32> = CausalTensor::new(vec![1.0f32], vec![1]).unwrap();
    let co_exact: CausalTensor<f32> = CausalTensor::new(vec![0.0f32], vec![1]).unwrap();
    let harmonic: CausalTensor<f32> = CausalTensor::new(vec![0.0f32], vec![1]).unwrap();
    let decomposition: HodgeDecomposition<f32> =
        HodgeDecomposition::new(exact, co_exact, harmonic, 1);
    assert_eq!(decomposition.grade(), 1);
}
