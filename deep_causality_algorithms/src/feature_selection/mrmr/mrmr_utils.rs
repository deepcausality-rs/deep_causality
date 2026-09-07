/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::feature_selection::mrmr::mrmr_error::MrmrError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_stats::{StatsErrorEnum, pearson_pairwise_complete};
use deep_causality_tensor::CausalTensor;

/// Calculates the Pearson correlation coefficient between two columns of a `CausalTensor`.
///
/// This function is generic over any type `T` that implements `Into<Option<F>>`,
/// where `F` is a real-field scalar. It handles missing data (represented
/// by `None` or `NaN` values) using pairwise deletion.
///
/// # Arguments
///
/// * `tensor` - A reference to a 2-dimensional `CausalTensor<T>`.
/// * `col_a_idx` - The column index of the first variable.
/// * `col_b_idx` - The column index of the second variable.
///
/// # Returns
///
/// A `Result` containing:
/// * `Ok((f64, f64))` - A tuple containing the calculated Pearson correlation coefficient
///   and the number of valid pairs (`n`) used in the calculation.
/// * `Err(MrmrError)` - An error if the input is invalid or sample size is too small.
///
pub(super) fn pearson_correlation<T, F>(
    tensor: &CausalTensor<T>,
    col_a_idx: usize,
    col_b_idx: usize,
) -> Result<(f64, f64), MrmrError>
where
    T: Copy + Into<Option<F>>,
    F: RealField + FromPrimitive,
{
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

    let mut a = Vec::with_capacity(n_rows);
    let mut b = Vec::with_capacity(n_rows);
    for row in 0..n_rows {
        let left: Option<F> = (*tensor.get(&[row, col_a_idx]).ok_or_else(|| {
            MrmrError::CalculationError("Failed to get value from tensor".to_string())
        })?)
        .into();
        let right: Option<F> = (*tensor.get(&[row, col_b_idx]).ok_or_else(|| {
            MrmrError::CalculationError("Failed to get value from tensor".to_string())
        })?)
        .into();
        a.push(left.filter(|value| !value.is_nan()));
        b.push(right.filter(|value| !value.is_nan()));
    }
    let (r, n) = pearson_pairwise_complete(&a, &b).map_err(|error| match error.kind() {
        StatsErrorEnum::EmptyInput(_) | StatsErrorEnum::InsufficientSamples(_) => {
            MrmrError::SampleTooSmall(2)
        }
        _ => MrmrError::CalculationError(error.to_string()),
    })?;
    let r = r.to_f64().ok_or_else(|| {
        MrmrError::CalculationError("Failed to convert correlation to f64".to_string())
    })?;
    Ok((r, n as f64))
}

/// Calculates the F-statistic between a feature and a target column.
///
/// This function is generic over any type `T` that implements `Into<Option<F>>`,
/// where `F` is a real-field scalar. It uses `pearson_correlation`
/// to handle missing data via pairwise deletion.
///
/// # Arguments
///
/// * `tensor` - A reference to a 2-dimensional `CausalTensor<T>`.
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
pub(super) fn f_statistic<T, F>(
    tensor: &CausalTensor<T>,
    feature_idx: usize,
    target_idx: usize,
) -> Result<f64, MrmrError>
where
    T: Copy + Into<Option<F>>,
    F: RealField + FromPrimitive,
{
    let (r, n) = pearson_correlation(tensor, feature_idx, target_idx)?;

    if n < 3.0 {
        // F-statistic requires n-2 > 0.
        return Err(MrmrError::SampleTooSmall(3));
    }

    let r2 = r.powi(2);

    if (1.0 - r2).abs() < 1e-9 {
        // Correlation is 1 or -1, implying infinite relevance.
        return Ok(1e12);
    }

    let f_stat = (n - 2.0) * r2 / (1.0 - r2);
    Ok(f_stat)
}
