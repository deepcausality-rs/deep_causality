/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, CausalTensorError};

#[test]
fn test_reshape_success() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let reshaped = tensor.reshape(&[3, 2]).unwrap();

    assert_eq!(reshaped.shape(), &[3, 2]);
    assert_eq!(reshaped.as_slice(), &[1, 2, 3, 4, 5, 6]);
    // Indirectly test strides by checking element access
    assert_eq!(reshaped.get(&[0, 0]), Some(&1));
    assert_eq!(reshaped.get(&[0, 1]), Some(&2));
    assert_eq!(reshaped.get(&[1, 0]), Some(&3));
    assert_eq!(reshaped.get(&[2, 1]), Some(&6));
}

#[test]
fn test_reshape_to_vector() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let reshaped = tensor.reshape(&[6]).unwrap();
    assert_eq!(reshaped.shape(), &[6]);
    assert_eq!(reshaped.as_slice(), &[1, 2, 3, 4, 5, 6]);
}

#[test]
fn test_reshape_shape_mismatch() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let result = tensor.reshape(&[2, 2]);
    assert_eq!(result, Err(CausalTensorError::ShapeMismatch));
}

#[test]
fn test_ravel() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let original_data = tensor.as_slice().to_vec();
    let raveled = tensor.ravel();

    assert_eq!(raveled.shape(), &[6]);
    assert_eq!(raveled.as_slice(), original_data.as_slice());
}

#[test]
fn test_ravel_on_vector() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let raveled = tensor.ravel();
    assert_eq!(raveled.shape(), &[3]);
}

#[test]
fn test_ravel_on_scalar() {
    let tensor = CausalTensor::new(vec![42], vec![]).unwrap();
    let raveled = tensor.ravel();
    // A scalar has len 1, so ravel should produce a vector of len 1.
    assert_eq!(raveled.shape(), &[1]);
    assert_eq!(raveled.as_slice(), &[42]);
}

#[test]
fn test_permute_axes_2d_transpose() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    // Original tensor:
    // [[1, 2, 3],
    //  [4, 5, 6]]

    let permuted = tensor.permute_axes(&[1, 0]).unwrap();
    // Expected permuted tensor (transpose):
    // [[1, 4],
    //  [2, 5],
    //  [3, 6]]

    assert_eq!(permuted.shape(), &[3, 2]);
    assert_eq!(permuted.as_slice(), &[1, 2, 3, 4, 5, 6]); // Data is not reordered

    // Verify element access
    assert_eq!(permuted.get(&[0, 0]), Some(&1));
    assert_eq!(permuted.get(&[0, 1]), Some(&4));
    assert_eq!(permuted.get(&[1, 0]), Some(&2));
    assert_eq!(permuted.get(&[1, 1]), Some(&5));
    assert_eq!(permuted.get(&[2, 0]), Some(&3));
    assert_eq!(permuted.get(&[2, 1]), Some(&6));
}

#[test]
fn test_permute_axes_3d() {
    let data: Vec<i32> = (0..24).collect();
    let tensor = CausalTensor::new(data, vec![2, 3, 4]).unwrap();
    // Original tensor shape: (2, 3, 4)
    // Axes: 0, 1, 2

    // Permute to (3, 4, 2)
    let permuted = tensor.permute_axes(&[1, 2, 0]).unwrap();
    assert_eq!(permuted.shape(), &[3, 4, 2]);

    // Original: tensor[d0, d1, d2]
    // Permuted: permuted[d1, d2, d0] should be the same element
    assert_eq!(tensor.get(&[0, 0, 0]), Some(&0));
    assert_eq!(permuted.get(&[0, 0, 0]), Some(&0));

    assert_eq!(tensor.get(&[1, 2, 3]), Some(&23));
    assert_eq!(permuted.get(&[2, 3, 1]), Some(&23));

    assert_eq!(tensor.get(&[0, 1, 2]), Some(&6));
    assert_eq!(permuted.get(&[1, 2, 0]), Some(&6));
}

#[test]
fn test_permute_axes_identity() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let permuted = tensor.permute_axes(&[0, 1]).unwrap();

    assert_eq!(permuted.shape(), &[2, 3]);
    assert_eq!(permuted.as_slice(), tensor.as_slice());
    assert_eq!(permuted, tensor); // Should be identical
}

#[test]
fn test_permute_axes_invalid_len() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let result = tensor.permute_axes(&[0]);
    assert_eq!(result, Err(CausalTensorError::DimensionMismatch));

    let result_long = tensor.permute_axes(&[0, 1, 2]);
    assert_eq!(result_long, Err(CausalTensorError::DimensionMismatch));
}

#[test]
fn test_permute_axes_duplicate_axis() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let result = tensor.permute_axes(&[0, 0]);
    assert_eq!(
        result,
        Err(CausalTensorError::InvalidParameter(
            "Invalid axes permutation".to_string()
        ))
    );
}

#[test]
fn test_permute_axes_out_of_bounds_axis() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let result = tensor.permute_axes(&[0, 2]);
    assert_eq!(
        result,
        Err(CausalTensorError::InvalidParameter(
            "Invalid axes permutation".to_string()
        ))
    );
}

#[test]
fn test_permute_axes_1d() {
    let tensor = CausalTensor::new(vec![1, 2, 3], vec![3]).unwrap();
    let permuted = tensor.permute_axes(&[0]).unwrap();
    assert_eq!(permuted.shape(), &[3]);
    assert_eq!(permuted, tensor);
}

#[test]
fn test_permute_axes_scalar() {
    let tensor = CausalTensor::new(vec![42], vec![]).unwrap();
    // Permuting a 0-dim tensor
    let permuted = tensor.permute_axes(&[]).unwrap();
    assert_eq!(permuted.shape(), &[]);
    assert_eq!(permuted, tensor);
}