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
fn test_display_format_carries_grade_and_component_lengths() {
    let decomposition = HodgeDecomposition::new(
        tensor_1d(vec![1.0, 2.0, 3.0]),
        tensor_1d(vec![4.0, 5.0]),
        tensor_1d(vec![6.0]),
        1,
    );
    let s = format!("{}", decomposition);
    assert_eq!(
        s,
        "HodgeDecomposition(grade=1, exact_len=3, co_exact_len=2, harmonic_len=1)"
    );
}

#[test]
fn test_display_format_at_grade_zero() {
    let decomposition = HodgeDecomposition::new(
        tensor_1d(vec![0.0]),
        tensor_1d(vec![0.0]),
        tensor_1d(vec![0.0]),
        0,
    );
    let s = format!("{}", decomposition);
    assert!(s.contains("grade=0"));
}

#[test]
fn test_debug_format_is_non_empty() {
    let decomposition = HodgeDecomposition::new(
        tensor_1d(vec![1.0]),
        tensor_1d(vec![1.0]),
        tensor_1d(vec![1.0]),
        2,
    );
    let s = format!("{:?}", decomposition);
    assert!(s.contains("HodgeDecomposition"));
}

#[test]
fn test_display_format_at_f32_precision() {
    let exact: CausalTensor<f32> = CausalTensor::new(vec![1.0f32, 2.0], vec![2]).unwrap();
    let co_exact: CausalTensor<f32> = CausalTensor::new(vec![0.0f32], vec![1]).unwrap();
    let harmonic: CausalTensor<f32> = CausalTensor::new(vec![0.0f32], vec![1]).unwrap();
    let decomposition: HodgeDecomposition<f32> =
        HodgeDecomposition::new(exact, co_exact, harmonic, 3);
    let s = format!("{}", decomposition);
    assert_eq!(
        s,
        "HodgeDecomposition(grade=3, exact_len=2, co_exact_len=1, harmonic_len=1)"
    );
}
