/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_discovery::{MissingValueImputer, PreprocessError};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_impute_mean_2d_tensor_check() {
    let tensor = CausalTensor::new(vec![1.0, f64::NAN, 3.0], vec![3]).unwrap(); // 1D tensor
    let result = MissingValueImputer::impute_mean(tensor);
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        PreprocessError::ImputeError(
            "Tensor must be 2-dimensional for column-wise imputation".to_string()
        )
    );
}

#[test]
fn test_impute_mean_empty_tensor() {
    let tensor = CausalTensor::new(vec![], vec![0, 0]).unwrap();
    let result = MissingValueImputer::impute_mean(tensor).unwrap();
    assert!(result.as_slice().is_empty());
    assert_eq!(result.shape(), &[0, 0]);
}

#[test]
fn test_impute_mean_single_nan() {
    let tensor = CausalTensor::new(vec![1.0, f64::NAN, 3.0, 4.0], vec![2, 2]).unwrap();
    let result = MissingValueImputer::impute_mean(tensor).unwrap();
    // Column 0: [1.0, 3.0] -> mean = 2.0
    // Column 1: [NaN, 4.0] -> mean = 4.0, NaN replaced by 4.0
    let expected = CausalTensor::new(vec![1.0, 4.0, 3.0, 4.0], vec![2, 2]).unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_impute_mean_multiple_nans_in_column() {
    let tensor =
        CausalTensor::new(vec![1.0, f64::NAN, 3.0, f64::NAN, 5.0, 6.0], vec![3, 2]).unwrap();
    let result = MissingValueImputer::impute_mean(tensor).unwrap();
    // Column 0: [1.0, 3.0, 5.0] -> mean = 3.0
    // Column 1: [NaN, NaN, 6.0] -> mean = 6.0, NaNs replaced by 6.0
    let expected = CausalTensor::new(vec![1.0, 6.0, 3.0, 6.0, 5.0, 6.0], vec![3, 2]).unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_impute_mean_no_nans() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    let result = MissingValueImputer::impute_mean(tensor).unwrap();
    let expected = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_impute_mean_column_all_nans() {
    let tensor = CausalTensor::new(vec![f64::NAN, 1.0, f64::NAN, 2.0], vec![2, 2]).unwrap();
    let result = MissingValueImputer::impute_mean(tensor).unwrap();
    // Column 0: [NaN, NaN] -> mean = 0.0 (default)
    // Column 1: [1.0, 2.0] -> mean = 1.5
    let expected = CausalTensor::new(vec![0.0, 1.0, 0.0, 2.0], vec![2, 2]).unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_impute_mean_mixed_columns() {
    let tensor = CausalTensor::new(
        vec![1.0, f64::NAN, 3.0, 4.0, f64::NAN, f64::NAN],
        vec![3, 2],
    )
    .unwrap();
    let result = MissingValueImputer::impute_mean(tensor).unwrap();
    // Column 0: [1.0, 3.0, NaN] -> mean = (1.0 + 3.0) / 2 = 2.0. NaN replaced by 2.0
    // Column 1: [NaN, 4.0, NaN] -> mean = 4.0. NaNs replaced by 4.0
    let expected = CausalTensor::new(vec![1.0, 4.0, 3.0, 4.0, 2.0, 4.0], vec![3, 2]).unwrap();
    assert_eq!(result.as_slice(), expected.as_slice());
}

#[test]
fn test_impute_mean_large_tensor() {
    let data = vec![
        1.0,
        2.0,
        f64::NAN,
        4.0,
        5.0,
        6.0,
        f64::NAN,
        8.0,
        9.0,
        10.0,
        11.0,
        12.0,
        13.0,
        f64::NAN,
        15.0,
        f64::NAN,
        17.0,
        18.0,
        19.0,
        20.0,
    ];
    let tensor = CausalTensor::new(data, vec![4, 5]).unwrap();
    let result = MissingValueImputer::impute_mean(tensor).unwrap();

    // Expected means for each column:
    // Col 0: [1.0, 6.0, 11.0, NaN] -> (1+6+11)/3 = 6.0. NaN replaced by 6.0
    // Col 1: [2.0, NaN, 12.0, 17.0] -> (2+12+17)/3 = 10.333... NaN replaced by 10.333...
    // Col 2: [NaN, 8.0, 13.0, 18.0] -> (8+13+18)/3 = 13.0. NaN replaced by 13.0
    // Col 3: [4.0, 9.0, NaN, 19.0] -> (4+9+19)/3 = 10.666... NaN replaced by 10.666...
    // Col 4: [5.0, 10.0, 15.0, 20.0] -> No NaNs

    let expected_data = vec![
        1.0,
        2.0,
        13.0,
        4.0,
        5.0,
        6.0,
        10.333333333333334,
        8.0,
        9.0,
        10.0,
        11.0,
        12.0,
        13.0,
        10.666666666666666,
        15.0,
        6.0,
        17.0,
        18.0,
        19.0,
        20.0,
    ];
    let expected = CausalTensor::new(expected_data, vec![4, 5]).unwrap();

    for i in 0..result.as_slice().len() {
        assert!((result.as_slice()[i] - expected.as_slice()[i]).abs() < f64::EPSILON);
    }
}
