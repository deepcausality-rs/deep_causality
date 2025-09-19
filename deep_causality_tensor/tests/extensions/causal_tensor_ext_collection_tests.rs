/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_tensor::{CausalTensor, CausalTensorCollectionExt, CausalTensorError};

#[test]
fn test_stack_empty_slice() {
    let tensors: [CausalTensor<i32>; 0] = [];
    let result = tensors.stack(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CausalTensorError::EmptyTensor);
}

#[test]
fn test_stack_axis_out_of_bounds() {
    let tensor = CausalTensor::<i32>::new(vec![1, 2], vec![2]).unwrap();
    let tensors = [tensor];
    // axis 2 is out of bounds for a 1D tensor (ndim=1, so max axis is 1)
    let result = tensors.stack(2);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CausalTensorError::AxisOutOfBounds);
}

#[test]
fn test_stack_shape_mismatch() {
    let tensor1 = CausalTensor::<i32>::new(vec![1, 2], vec![2]).unwrap();
    let tensor2 = CausalTensor::<i32>::new(vec![3, 4, 5], vec![3]).unwrap();
    let tensors = [tensor1, tensor2];
    let result = tensors.stack(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), CausalTensorError::ShapeMismatch);
}

#[test]
fn test_stack_scalars() {
    let tensor1 = CausalTensor::<i32>::new(vec![1], vec![]).unwrap();
    let tensor2 = CausalTensor::<i32>::new(vec![2], vec![]).unwrap();
    let tensors = [tensor1, tensor2];

    // Stack along axis 0
    let result = tensors.stack(0).unwrap();
    assert_eq!(result.shape(), &[2]);
    assert_eq!(result.as_slice(), &[1, 2]);
}

#[test]
fn test_stack_vectors_axis_0() {
    let tensor1 = CausalTensor::<i32>::new(vec![1, 2], vec![2]).unwrap();
    let tensor2 = CausalTensor::<i32>::new(vec![3, 4], vec![2]).unwrap();
    let tensors = [tensor1, tensor2];

    // Stack along axis 0 -> shape [2, 2]
    let result = tensors.stack(0).unwrap();
    assert_eq!(result.shape(), &[2, 2]);
    assert_eq!(result.as_slice(), &[1, 2, 3, 4]);
}

#[test]
fn test_stack_vectors_axis_1() {
    let tensor1 = CausalTensor::<i32>::new(vec![1, 2], vec![2]).unwrap();
    let tensor2 = CausalTensor::<i32>::new(vec![3, 4], vec![2]).unwrap();
    let tensors = [tensor1, tensor2];

    // Stack along axis 1 -> shape [2, 2]
    let result = tensors.stack(1).unwrap();
    assert_eq!(result.shape(), &[2, 2]);
    // The data should be interleaved
    assert_eq!(result.as_slice(), &[1, 3, 2, 4]);
}

#[test]
fn test_stack_matrices_axis_0() {
    let tensor1 = CausalTensor::<i32>::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let tensor2 = CausalTensor::<i32>::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();
    let tensors = [tensor1, tensor2];

    // Stack along axis 0 -> shape [2, 2, 2]
    let result = tensors.stack(0).unwrap();
    assert_eq!(result.shape(), &[2, 2, 2]);
    assert_eq!(result.as_slice(), &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn test_stack_matrices_axis_1() {
    let tensor1 = CausalTensor::<i32>::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let tensor2 = CausalTensor::<i32>::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();
    let tensors = [tensor1, tensor2];

    // Stack along axis 1 -> shape [2, 2, 2]
    let result = tensors.stack(1).unwrap();
    assert_eq!(result.shape(), &[2, 2, 2]);
    assert_eq!(result.as_slice(), &[1, 2, 5, 6, 3, 4, 7, 8]);
}

#[test]
fn test_stack_matrices_axis_2() {
    let tensor1 = CausalTensor::<i32>::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let tensor2 = CausalTensor::<i32>::new(vec![5, 6, 7, 8], vec![2, 2]).unwrap();
    let tensors = [tensor1, tensor2];

    // Stack along axis 2 -> shape [2, 2, 2]
    let result = tensors.stack(2).unwrap();
    assert_eq!(result.shape(), &[2, 2, 2]);
    assert_eq!(result.as_slice(), &[1, 5, 2, 6, 3, 7, 4, 8]);
}

#[test]
fn test_stack_single_tensor() {
    let tensor = CausalTensor::<i32>::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let tensors = [tensor];

    // Stack along axis 0
    let result = tensors.stack(0).unwrap();
    assert_eq!(result.shape(), &[1, 2, 2]);
    assert_eq!(result.as_slice(), &[1, 2, 3, 4]);
}
