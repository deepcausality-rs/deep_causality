/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::feature_selection::mrmr::mrmr_error::MrmrError;
use deep_causality_tensor::CausalTensor;

/// Calculates the Pearson correlation coefficient between two columns of a `CausalTensor`.
///
/// The Pearson correlation coefficient measures the linear correlation between two sets of data.
/// It ranges from -1 to +1, where +1 is a total positive linear correlation, 0 is no linear
/// correlation, and -1 is a total negative linear correlation.
///
/// # Arguments
///
/// * `tensor` - A reference to a 2-dimensional `CausalTensor<f64>`.
/// * `col_a_idx` - The column index of the first variable.
/// * `col_b_idx` - The column index of the second variable.
///
/// # Returns
///
/// A `Result` containing:
/// * `Ok(f64)` - The calculated Pearson correlation coefficient.
/// * `Err(MrmrError)` - An error if the input is invalid or sample size is too small.
///
/// # Errors
///
/// This function can return the following `MrmrError` variants:
/// * `MrmrError::InvalidInput` - If the tensor is not 2-dimensional or column indices are out of bounds.
/// * `MrmrError::SampleTooSmall` - If the number of rows in the tensor is less than 2, which is insufficient for correlation calculation.
///
pub(super) fn pearson_correlation(
    tensor: &CausalTensor<f64>,
    col_a_idx: usize,
    col_b_idx: usize,
) -> Result<f64, MrmrError> {
    let shape = tensor.shape();
    if shape.len() != 2 {
        return Err(MrmrError::InvalidInput(
            "Input tensor must be 2-dimensional".to_string(),
        ));
    }

    let n_rows = shape[0];
    let n_cols = shape[1];

    if col_a_idx >= n_cols || col_b_idx >= n_cols {
        return Err(MrmrError::InvalidInput(
            "Column index out of bounds".to_string(),
        ));
    }

    if n_rows < 2 {
        // Correlation is not well-defined for less than 2 samples.
        return Err(MrmrError::SampleTooSmall(2));
    }

    let mut sum_a = 0.0;
    let mut sum_b = 0.0;
    let mut sum_sq_a = 0.0;
    let mut sum_sq_b = 0.0;
    let mut sum_prod = 0.0;

    for i in 0..n_rows {
        let a = *tensor.get(&[i, col_a_idx]).unwrap();
        let b = *tensor.get(&[i, col_b_idx]).unwrap();
        sum_a += a;
        sum_b += b;
        sum_sq_a += a * a;
        sum_sq_b += b * b;
        sum_prod += a * b;
    }

    let n = n_rows as f64;
    let numerator = sum_prod - (sum_a * sum_b) / n;
    let denominator_a = sum_sq_a - (sum_a * sum_a) / n;
    let denominator_b = sum_sq_b - (sum_b * sum_b) / n;

    if denominator_a <= 0.0 || denominator_b <= 0.0 {
        return Ok(0.0);
    }

    Ok(numerator / (denominator_a.sqrt() * denominator_b.sqrt()))
}

/// Calculates the F-statistic between a feature and a target column.
///
/// The F-statistic is used as a measure of relevance in mRMR algorithms.
/// It quantifies the ratio of variance between group means to variance within groups.
/// A higher F-statistic indicates a stronger relationship between the feature and the target.
///
/// # Arguments
///
/// * `tensor` - A reference to a 2-dimensional `CausalTensor<f64>`.
/// * `feature_idx` - The column index of the feature variable.
/// * `target_idx` - The column index of the target variable.
///
/// # Returns
///
/// A `Result` containing:
/// * `Ok(f64)` - The calculated F-statistic.
/// *  Ok(1e12) (a large number) if the correlation is perfect (1 or -1), to avoid division by zero and handle infinite relevance in mRMR scoring.
/// * `Err(MrmrError)` - An error if the sample size is too small.
///
/// # Errors
///
/// This function can return the following `MrmrError` variants:
/// * `MrmrError::SampleTooSmall` - If the number of rows in the tensor is less than 3, which is insufficient for F-statistic calculation.
/// * `MrmrError::InvalidInput` - Propagated from `pearson_correlation` if column indices are out of bounds or tensor is not 2D.
///
pub(super) fn f_statistic(
    tensor: &CausalTensor<f64>,
    feature_idx: usize,
    target_idx: usize,
) -> Result<f64, MrmrError> {
    let n_rows = tensor.shape()[0];
    if n_rows < 3 {
        // F-statistic is not well-defined for less than 3 samples (n-2 must be > 0)
        return Err(MrmrError::SampleTooSmall(3));
    }

    let r = pearson_correlation(tensor, feature_idx, target_idx)?;
    let r2 = r.powi(2);

    if (1.0 - r2).abs() < 1e-9 {
        // Correlation is 1 or -1, leading to division by zero.
        // This implies infinite relevance. To avoid Inf - Inf issues, return a large number.
        return Ok(1e12);
    }

    let f_stat = (n_rows as f64 - 2.0) * r2 / (1.0 - r2);
    Ok(f_stat)
}

/// Handles missing data in a CausalTensor by imputing column means.
/// NaN values are replaced with the mean of their respective columns.
pub(super) fn impute_missing_values(tensor: &mut CausalTensor<f64>) {
    let shape = tensor.shape();
    let n_rows = shape[0];
    let n_cols = shape[1];

    for col_idx in 0..n_cols {
        let mut sum = 0.0;
        let mut count = 0;
        for row_idx in 0..n_rows {
            if let Some(val) = tensor.get(&[row_idx, col_idx])
                && !val.is_nan()
            {
                sum += *val;
                count += 1;
            }
        }

        let mean = if count > 0 { sum / count as f64 } else { 0.0 };

        for row_idx in 0..n_rows {
            if let Some(val) = tensor.get_mut(&[row_idx, col_idx])
                && val.is_nan()
            {
                *val = mean;
            }
        }
    }
}
