/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::{CausalTensor, Tensor};

// --- mod.rs constructors: from_shape_fn / from_slice / zeros ---

#[test]
fn test_from_shape_fn_2d() {
    // value at (i, j) = i * 10 + j
    let t = CausalTensor::from_shape_fn(&[2, 3], |idx| (idx[0] * 10 + idx[1]) as i32);
    assert_eq!(t.shape(), &[2, 3]);
    assert_eq!(t.as_slice(), &[0, 1, 2, 10, 11, 12]);
}

#[test]
fn test_from_shape_fn_empty_shape_zero_elements() {
    // A shape with a zero dimension produces zero elements (early return path).
    let t = CausalTensor::from_shape_fn(&[0, 4], |_idx| 1.0_f64);
    assert_eq!(t.shape(), &[0, 4]);
    assert!(t.is_empty());
}

#[test]
fn test_from_shape_fn_scalar() {
    let t = CausalTensor::from_shape_fn(&[], |_idx| 7_i32);
    assert_eq!(t.shape(), &[] as &[usize]);
    assert_eq!(t.as_slice(), &[7]);
}

#[test]
fn test_from_slice() {
    let data = [1, 2, 3, 4];
    let t = CausalTensor::from_slice(&data, &[2, 2]);
    assert_eq!(t.shape(), &[2, 2]);
    assert_eq!(t.as_slice(), &[1, 2, 3, 4]);
}

#[test]
fn test_zeros() {
    let t = CausalTensor::<f64>::zeros(&[2, 2]);
    assert_eq!(t.shape(), &[2, 2]);
    assert_eq!(t.as_slice(), &[0.0; 4]);
}

// --- to/mod.rs: from_vec / into_vec / to_vec ---

#[test]
fn test_from_vec() {
    let t = CausalTensor::from_vec(vec![1, 2, 3], &[3]);
    assert_eq!(t.shape(), &[3]);
    assert_eq!(t.as_slice(), &[1, 2, 3]);
}

#[test]
fn test_into_vec() {
    let t = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    let v = t.into_vec();
    assert_eq!(v, vec![1, 2, 3, 4]);
}

#[test]
fn test_to_vec() {
    let t = CausalTensor::new(vec![9, 8, 7], vec![3]).unwrap();
    let v = t.to_vec();
    assert_eq!(v, vec![9, 8, 7]);
}

// --- api/mod.rs: matmul / norm_l2 / norm_sq ---

#[test]
fn test_matmul_2d() {
    // [[1, 2], [3, 4]] x [[5, 6], [7, 8]] = [[19, 22], [43, 50]]
    let a = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let b = CausalTensor::new(vec![5.0, 6.0, 7.0, 8.0], vec![2, 2]).unwrap();
    let c = a.matmul(&b).unwrap();
    assert_eq!(c.shape(), &[2, 2]);
    assert_eq!(c.as_slice(), &[19.0, 22.0, 43.0, 50.0]);
}

#[test]
fn test_norm_l2() {
    // sqrt(3^2 + 4^2) = 5
    let t = CausalTensor::new(vec![3.0_f64, 4.0], vec![2]).unwrap();
    let n: f64 = t.norm_l2();
    assert!((n - 5.0).abs() < 1e-12);
}

#[test]
fn test_norm_sq() {
    // 3^2 + 4^2 = 25
    let t = CausalTensor::new(vec![3.0_f64, 4.0], vec![2]).unwrap();
    let n: f64 = t.norm_sq();
    assert!((n - 25.0).abs() < 1e-12);
}

// --- tensor_shape: reshape of a non-contiguous (permuted) tensor ---

#[test]
fn test_reshape_non_contiguous() {
    // Build a 2x3 tensor, transpose it (now non-contiguous strided view),
    // then reshape. The reshape must materialize data in logical order.
    let base = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let transposed = base.permute_axes(&[1, 0]).unwrap(); // logical shape [3, 2]
    // Logical row-major order of the transpose is [1, 4, 2, 5, 3, 6].
    let reshaped = transposed.reshape(&[2, 3]).unwrap();
    assert_eq!(reshaped.shape(), &[2, 3]);
    assert_eq!(reshaped.as_slice(), &[1, 4, 2, 5, 3, 6]);
}

#[test]
fn test_reshape_non_contiguous_to_vector() {
    let base = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let transposed = base.permute_axes(&[1, 0]).unwrap();
    let flat = transposed.reshape(&[6]).unwrap();
    assert_eq!(flat.shape(), &[6]);
    assert_eq!(flat.as_slice(), &[1, 4, 2, 5, 3, 6]);
}
