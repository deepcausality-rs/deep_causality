/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;

#[test]
fn test_from_t_for_causal_tensor() {
    let data = 42;
    let tensor = CausalTensor::from(data);
    assert_eq!(tensor.data(), &[42]);
    assert!(tensor.shape().is_empty());
}

#[test]
fn test_from_ref_t_for_causal_tensor() {
    let data = 42;
    let tensor: CausalTensor<i32> = CausalTensor::from(&data);
    assert_eq!(tensor.data(), &[42]);
    assert!(tensor.shape().is_empty());
}

#[test]
fn test_from_ref_causal_tensor_for_causal_tensor() {
    let original_tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let cloned_tensor = CausalTensor::from(original_tensor.clone());
    assert_eq!(cloned_tensor.data(), &[1, 2, 3]);
    assert_eq!(cloned_tensor.shape(), &[3]);
    assert_eq!(original_tensor, cloned_tensor);
}
