/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{DataCleaner, OptionNoneDataCleaner};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_option_none_data_cleaner_nan_to_none() {
    let raw_data = vec![1.0, 2.0, f64::NAN, 4.0, f64::NAN, 6.0, f64::NAN, 8.0, 9.0];
    let raw_tensor = CausalTensor::new(raw_data, vec![3, 3]).unwrap();

    let cleaner = OptionNoneDataCleaner;
    let cleaned_tensor = cleaner.process(raw_tensor).unwrap();

    let expected_data = vec![
        Some(1.0),
        Some(2.0),
        None,
        Some(4.0),
        None,
        Some(6.0),
        None,
        Some(8.0),
        Some(9.0),
    ];
    let expected_tensor = CausalTensor::new(expected_data, vec![3, 3]).unwrap();

    assert_eq!(cleaned_tensor.as_slice(), expected_tensor.as_slice());
    assert_eq!(cleaned_tensor.shape(), expected_tensor.shape());
}

#[test]
fn test_option_none_data_cleaner_no_nan() {
    let raw_data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let raw_tensor = CausalTensor::new(raw_data, vec![3, 3]).unwrap();

    let cleaner = OptionNoneDataCleaner;
    let cleaned_tensor = cleaner.process(raw_tensor).unwrap();

    let expected_data = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(4.0),
        Some(5.0),
        Some(6.0),
        Some(7.0),
        Some(8.0),
        Some(9.0),
    ];
    let expected_tensor = CausalTensor::new(expected_data, vec![3, 3]).unwrap();

    assert_eq!(cleaned_tensor.as_slice(), expected_tensor.as_slice());
    assert_eq!(cleaned_tensor.shape(), expected_tensor.shape());
}

#[test]
fn test_option_none_data_cleaner_all_nan() {
    let raw_data = vec![
        f64::NAN,
        f64::NAN,
        f64::NAN,
        f64::NAN,
        f64::NAN,
        f64::NAN,
        f64::NAN,
        f64::NAN,
        f64::NAN,
    ];
    let raw_tensor = CausalTensor::new(raw_data, vec![3, 3]).unwrap();

    let cleaner = OptionNoneDataCleaner;
    let cleaned_tensor = cleaner.process(raw_tensor).unwrap();

    let expected_data = vec![None, None, None, None, None, None, None, None, None];
    let expected_tensor = CausalTensor::new(expected_data, vec![3, 3]).unwrap();

    assert_eq!(cleaned_tensor.as_slice(), expected_tensor.as_slice());
    assert_eq!(cleaned_tensor.shape(), expected_tensor.shape());
}

#[test]
fn test_option_none_data_cleaner_empty_tensor() {
    let raw_data = vec![];
    let raw_tensor = CausalTensor::new(raw_data, vec![0, 0]).unwrap();

    let cleaner = OptionNoneDataCleaner;
    let cleaned_tensor = cleaner.process(raw_tensor).unwrap();

    let expected_data: Vec<Option<f64>> = vec![];
    let expected_tensor = CausalTensor::new(expected_data, vec![0, 0]).unwrap();

    assert_eq!(cleaned_tensor.as_slice(), expected_tensor.as_slice());
    assert_eq!(cleaned_tensor.shape(), expected_tensor.shape());
}
