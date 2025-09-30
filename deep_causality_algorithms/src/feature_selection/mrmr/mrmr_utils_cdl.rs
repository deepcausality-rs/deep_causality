/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_tensor::CausalTensor;

use crate::feature_selection::mrmr::mrmr_error::MrmrError;

/// Calculates the Pearson correlation coefficient between two columns of a `CausalTensor<Option<f64>>`.
///
/// This function handles missing data by performing pairwise deletion. For each pair of values
/// in the two columns, if either value is missing (i.e., is `None`), the pair
/// is excluded from the calculation. The sample size `n` is dynamically adjusted to reflect
/// the number of complete pairs.
///
/// # Arguments
///
/// * `tensor` - A reference to a 2-dimensional `CausalTensor<Option<f64>>`.
/// * `col_a_idx` - The column index of the first variable.
/// * `col_b_idx` - The column index of the second variable.
///
/// # Returns
///
/// A `Result` containing:
/// * `Ok(f64)` - The calculated Pearson correlation coefficient.
/// * `Err(MrmrError)` - An error if the input is invalid, sample size is too small, or an uncertainty error occurs.
///
pub(super) fn pearson_correlation_cdl(
    tensor: &CausalTensor<Option<f64>>,
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

    let mut sum_a: f64 = 0.0;
    let mut sum_b: f64 = 0.0;
    let mut sum_sq_a: f64 = 0.0;
    let mut sum_sq_b: f64 = 0.0;
    let mut sum_prod: f64 = 0.0;
    let mut n: f64 = 0.0;

    for i in 0..n_rows {
        let a_option = tensor.get(&[i, col_a_idx]).unwrap();
        let b_option = tensor.get(&[i, col_b_idx]).unwrap();

        if let (Some(a), Some(b)) = (a_option, b_option) {
            sum_a += a;
            sum_b += b;
            sum_sq_a += a * a;
            sum_sq_b += b * b;
            sum_prod += a * b;
            n += 1.0;
        }
    }

    if n < 2.0 {
        // Correlation is not well-defined for less than 2 samples.
        return Err(MrmrError::FeatureScoreError(format!(
            "Pearson correlation for columns {} and {} requires at least 2 valid samples, but found {}.",
            col_a_idx, col_b_idx, n as usize
        )));
    }

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
/// This function uses `pearson_correlation_cdl` to handle `Option<f64>` values,
/// thereby correctly managing missing data via pairwise deletion.
///
/// # Arguments
///
/// * `tensor` - A reference to a 2-dimensional `CausalTensor<Option<f64>>`.
/// * `feature_idx` - The column index of the feature variable.
/// * `target_idx` - The column index of the target variable.
///
/// # Returns
///
/// A `Result` containing:
/// * `Ok(f64)` - The calculated F-statistic.
/// * `Err(MrmrError)` - An error if the sample size is too small or an uncertainty error occurs.
///
pub(super) fn f_statistic_cdl(
    tensor: &CausalTensor<Option<f64>>,
    feature_idx: usize,
    target_idx: usize,
) -> Result<f64, MrmrError> {
    // Note: The effective number of rows `n` is determined inside pearson_correlation_cdl.
    // We need a preliminary check here to ensure there's enough data to even attempt the calculation.
    if tensor.shape()[0] < 3 {
        return Err(MrmrError::SampleTooSmall(3));
    }

    let r = pearson_correlation_cdl(tensor, feature_idx, target_idx)?;
    let r2 = r.powi(2);

    // The dynamic `n` from pearson_correlation is not available here.
    // We must re-calculate it to ensure the F-statistic is accurate.
    let mut n = 0.0;
    for i in 0..tensor.shape()[0] {
        let a_option = tensor.get(&[i, feature_idx]).unwrap();
        let b_option = tensor.get(&[i, target_idx]).unwrap();

        if a_option.is_some() && b_option.is_some() {
            n += 1.0;
        }
    }

    if n < 3.0 {
        // F-statistic requires n-2 > 0.
        return Err(MrmrError::SampleTooSmall(3));
    }

    if (1.0 - r2).abs() < 1e-9 {
        // Correlation is 1 or -1, implying infinite relevance.
        return Ok(1e12);
    }

    let f_stat = (n - 2.0) * r2 / (1.0 - r2);
    Ok(f_stat)
}
