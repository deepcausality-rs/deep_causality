/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::feature_selection::mrmr::mrmr_error::MrmrError;
use crate::mrmr::mrmr_utils;
use deep_causality_data_structures::CausalTensor;

#[test]
fn test_pearson_correlation() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 4.0, 3.0, 2.0, 1.0];
    let shape = vec![2, 5];
    let tensor = CausalTensor::new(data, shape).unwrap();

    let corr = mrmr_utils::pearson_correlation(&tensor, 0, 4).unwrap();
    assert!((corr - (-1.0)).abs() < 1e-9);
}

#[test]
fn test_pearson_correlation_non_2d_tensor() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let shape = vec![4]; // 1D tensor
    let tensor = CausalTensor::new(data, shape).unwrap();

    let result = mrmr_utils::pearson_correlation(&tensor, 0, 1);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Input tensor must be 2-dimensional"
    );
}

#[test]
fn test_pearson_correlation_index_out_of_bounds() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let shape = vec![2, 2];
    let tensor = CausalTensor::new(data, shape).unwrap();

    // col_b_idx is 2, which is out of bounds for a 2-column tensor.
    let result = mrmr_utils::pearson_correlation(&tensor, 0, 2);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Column index out of bounds"
    );
}

#[test]
fn test_pearson_correlation_sample_too_small() {
    let data = vec![1.0, 2.0];
    let shape = vec![1, 2]; // 1 row, 2 columns
    let tensor = CausalTensor::new(data, shape).unwrap();

    let result = mrmr_utils::pearson_correlation(&tensor, 0, 1);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(2))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Sample size is too small. At least 2 samples are required."
    );
}

#[test]
fn test_f_statistic() {
    let data = vec![1.0, 2.0, 2.0, 4.0, 3.0, 6.0];
    let shape = vec![3, 2];
    let tensor = CausalTensor::new(data, shape).unwrap();

    // Correlation is 1.0, so F-statistic should be a large number.
    let f_stat = mrmr_utils::f_statistic(&tensor, 0, 1).unwrap();
    assert_eq!(f_stat, 1e12);
}

#[test]
fn test_f_statistic_sample_too_small() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let shape = vec![2, 2]; // 2 rows, less than the required 3
    let tensor = CausalTensor::new(data, shape).unwrap();

    let result = mrmr_utils::f_statistic(&tensor, 0, 1);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(3))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Sample size is too small. At least 3 samples are required."
    );
}

#[test]
fn test_impute_missing_values() {
    let data = vec![1.0, 2.0, f64::NAN, 4.0, 3.0, 6.0];
    let mut tensor = CausalTensor::new(data, vec![3, 2]).unwrap();

    mrmr_utils::impute_missing_values(&mut tensor);

    // Mean of column 0 is (1.0 + 3.0) / 2 = 2.0
    let imputed_val = tensor.get(&[1, 0]).unwrap();
    assert_eq!(*imputed_val, 2.0);

    // Ensure other values are unchanged
    assert_eq!(*tensor.get(&[0, 0]).unwrap(), 1.0);
    assert_eq!(*tensor.get(&[0, 1]).unwrap(), 2.0);
    assert_eq!(*tensor.get(&[1, 1]).unwrap(), 4.0);
}
