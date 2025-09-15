/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_data_structures::CausalTensor;

#[test]
fn test_display_non_empty_tensor() {
    let tensor = CausalTensor::new(vec![1, 2, 3, 4, 5, 6], vec![2, 3]).unwrap();
    let display_str = tensor.to_string();
    // Strides for shape [2, 3] are [3, 1]
    assert_eq!(
        display_str,
        "CausalTensor { data: [1, 2, 3, 4, 5, 6], shape: [2, 3], strides: [3, 1] }"
    );
}

#[test]
fn test_display_empty_tensor_shape_zero() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![0]).unwrap();
    let display_str = tensor.to_string();
    // Strides for shape [0] is [1] because product of trailing dims is 1.
    assert_eq!(
        display_str,
        "CausalTensor { data: [], shape: [0], strides: [1] }"
    );
}

#[test]
fn test_display_empty_tensor_shape_non_zero() {
    let tensor: CausalTensor<i32> = CausalTensor::new(vec![], vec![2, 0, 3]).unwrap();
    let display_str = tensor.to_string();
    // Strides for shape [2, 0, 3] are [0, 3, 1]
    assert_eq!(
        display_str,
        "CausalTensor { data: [], shape: [2, 0, 3], strides: [0, 3, 1] }"
    );
}

#[test]
fn test_display_single_element_tensor() {
    let tensor = CausalTensor::new(vec![42], vec![1, 1, 1]).unwrap();
    let display_str = tensor.to_string();
    // Strides for shape [1, 1, 1] are [1, 1, 1]
    assert_eq!(
        display_str,
        "CausalTensor { data: [42], shape: [1, 1, 1], strides: [1, 1, 1] }"
    );
}

#[test]
fn test_display_scalar_tensor() {
    let tensor = CausalTensor::new(vec![42], vec![]).unwrap();
    let display_str = tensor.to_string();
    // Strides for shape [] are []
    assert_eq!(
        display_str,
        "CausalTensor { data: [42], shape: [], strides: [] }"
    );
}

#[test]
fn test_display_complex_shape() {
    let data = (0..24).collect::<Vec<_>>();
    let tensor = CausalTensor::new(data, vec![2, 3, 4]).unwrap();
    let display_str = tensor.to_string();

    // The new Display impl truncates data longer than 10 items.
    let expected_str = "CausalTensor { data: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, ...], shape: [2, 3, 4], strides: [12, 4, 1] }";

    assert_eq!(display_str, expected_str);
}
