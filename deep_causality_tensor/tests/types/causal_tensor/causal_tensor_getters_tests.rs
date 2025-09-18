/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;

#[test]
fn test_get_from_2d() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    assert_eq!(tensor.get(&[0, 0]), Some(&1));
    assert_eq!(tensor.get(&[0, 1]), Some(&2));
    assert_eq!(tensor.get(&[0, 2]), Some(&3));
    assert_eq!(tensor.get(&[1, 0]), Some(&4));
    assert_eq!(tensor.get(&[1, 1]), Some(&5));
    assert_eq!(tensor.get(&[1, 2]), Some(&6));
}

#[test]
fn test_get_from_scalar() {
    let tensor = CausalTensor::new(vec![42], vec![]).unwrap();
    assert_eq!(tensor.get(&[]), Some(&42));
}

#[test]
fn test_get_out_of_bounds() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    // Dimension index out of bounds
    assert_eq!(tensor.get(&[2, 0]), None);
    assert_eq!(tensor.get(&[0, 2]), None);
}

#[test]
fn test_get_dimension_mismatch() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    // Wrong number of dimensions in index
    assert_eq!(tensor.get(&[0]), None);
    assert_eq!(tensor.get(&[0, 0, 0]), None);
}

#[test]
fn test_get_mut() {
    let mut tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();

    // Get and modify
    let val = tensor.get_mut(&[0, 1]).unwrap();
    *val = 20;
    assert_eq!(tensor.get(&[0, 1]), Some(&20));

    // Verify underlying data
    assert_eq!(tensor.as_slice(), &[1, 20, 3, 4]);
}

#[test]
fn test_get_mut_out_of_bounds() {
    let mut tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    assert_eq!(tensor.get_mut(&[2, 0]), None);
}

#[test]
fn test_get_mut_dimension_mismatch() {
    let mut tensor = CausalTensor::new(vec![1, 2, 3, 4], vec![2, 2]).unwrap();
    assert_eq!(tensor.get_mut(&[0]), None);
}

#[test]
fn test_get_from_3d() {
    let data = (0..24).collect::<Vec<_>>();
    let tensor = CausalTensor::new(data, vec![2, 3, 4]).unwrap();
    assert_eq!(tensor.get(&[1, 1, 1]), Some(&17));
}

#[test]
fn test_get_mut_from_3d() {
    let data = (0..24).collect::<Vec<_>>();
    let mut tensor = CausalTensor::new(data, vec![2, 3, 4]).unwrap();
    *tensor.get_mut(&[1, 1, 1]).unwrap() = 100;
    assert_eq!(tensor.get(&[1, 1, 1]), Some(&100));
}
