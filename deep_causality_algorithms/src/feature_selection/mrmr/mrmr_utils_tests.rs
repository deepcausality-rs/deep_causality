/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::feature_selection::mrmr::mrmr_error::MrmrError;
use crate::mrmr::mrmr_utils;
use deep_causality_tensor::CausalTensor;

#[test]
fn pearson_large_offset_retains_affine_correlation() {
    // A positive affine relationship has correlation one, independent of offset.
    let tensor = CausalTensor::new(
        vec![1e12, 3.0, 1e12 + 1.0, 5.0, 1e12 + 2.0, 7.0, 1e12 + 3.0, 9.0],
        vec![4, 2],
    )
    .unwrap();
    let (r, n) = mrmr_utils::pearson_correlation(&tensor, 0, 1).unwrap();
    assert!((r - 1.0).abs() < 1e-14);
    assert_eq!(n, 4.0);
}

#[test]
fn pearson_infinity_is_a_calculation_error() {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 2.0, f64::INFINITY], vec![2, 2]).unwrap();
    assert!(matches!(
        mrmr_utils::pearson_correlation(&tensor, 0, 1),
        Err(MrmrError::CalculationError(_))
    ));
}

#[test]
fn pearson_drops_a_nan_pair_rather_than_refusing_the_column() {
    // mRMR's policy is **pairwise deletion**: a row with a missing or `NaN` entry drops out and
    // the remaining rows are correlated. The `NaN` is converted to an absence before the call, so
    // the statistics crate — which refuses a non-finite observation, as
    // `pearson_infinity_is_a_calculation_error` above shows — never sees it.
    //
    // Without that conversion this column would be a `CalculationError` and mRMR would lose a
    // feature to one bad cell. Nothing else in the suite had a `NaN`, so the conversion was
    // unpinned even though the infinity case was.
    let tensor = CausalTensor::new(
        vec![1.0, 2.0, 2.0, 4.0, f64::NAN, 99.0, 4.0, 8.0],
        vec![4, 2],
    )
    .unwrap();

    let (r, n) = mrmr_utils::pearson_correlation(&tensor, 0, 1).unwrap();
    assert_eq!(n, 3.0, "the NaN row is deleted, leaving three pairs");
    // The surviving pairs are (1,2), (2,4), (4,8) — exactly proportional, so r = 1.
    assert!((r - 1.0).abs() < 1e-14, "expected r = 1, got {r}");
}

#[test]
fn test_pearson_correlation() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 5.0, 4.0, 3.0, 2.0, 1.0];
    let shape = vec![2, 5];
    let tensor = CausalTensor::new(data, shape).unwrap();

    let (corr, _) = mrmr_utils::pearson_correlation(&tensor, 0, 4).unwrap();
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
fn test_f_statistic_non_2d_tensor() {
    let data = vec![1.0, 2.0, 3.0, 4.0];
    let shape = vec![4]; // 1D tensor
    let tensor = CausalTensor::new(data, shape).unwrap();

    let result = mrmr_utils::f_statistic(&tensor, 0, 1);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Input tensor must be 2-dimensional"
    );
}

#[test]
fn test_f_statistic_index_out_of_bounds() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let shape = vec![3, 2];
    let tensor = CausalTensor::new(data, shape).unwrap();

    // col_b_idx is 2, which is out of bounds for a 2-column tensor.
    let result = mrmr_utils::f_statistic(&tensor, 0, 2);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Column index out of bounds"
    );
}
