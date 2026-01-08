/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::feature_selection::mrmr::mrmr_error::MrmrError;
use deep_causality_num::{Float, FloatOption};
use deep_causality_tensor::CausalTensor;

/// Calculates the Pearson correlation coefficient between two columns of a `CausalTensor`.
///
/// This function is generic over any type `T` that implements `FloatOption<F>`,
/// where `F` is a float type (`f32` or `f64`). It handles missing data (represented
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
    T: FloatOption<F>,
    F: Float,
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

    let mut sum_a: f64 = 0.0;
    let mut sum_b: f64 = 0.0;
    let mut sum_sq_a: f64 = 0.0;
    let mut sum_sq_b: f64 = 0.0;
    let mut sum_prod: f64 = 0.0;
    let mut n: f64 = 0.0;

    for i in 0..n_rows {
        let a_option = tensor
            .get(&[i, col_a_idx])
            .ok_or_else(|| {
                MrmrError::CalculationError("Failed to get value from tensor".to_string())
            })?
            .to_option();
        let b_option = tensor
            .get(&[i, col_b_idx])
            .ok_or_else(|| {
                MrmrError::CalculationError("Failed to get value from tensor".to_string())
            })?
            .to_option();

        if let (Some(a_val), Some(b_val)) = (a_option, b_option) {
            // Convert to f64 for calculation to maintain precision
            let a = a_val.to_f64().ok_or_else(|| {
                MrmrError::CalculationError("Failed to cast float to f64".to_string())
            })?;
            let b = b_val.to_f64().ok_or_else(|| {
                MrmrError::CalculationError("Failed to cast float to f64".to_string())
            })?;
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
        return Err(MrmrError::SampleTooSmall(2));
    }

    let numerator = sum_prod - (sum_a * sum_b) / n;
    let denominator_a = sum_sq_a - (sum_a * sum_a) / n;
    let denominator_b = sum_sq_b - (sum_b * sum_b) / n;

    if denominator_a <= 0.0 || denominator_b <= 0.0 {
        return Ok((0.0, n));
    }

    Ok((numerator / (denominator_a.sqrt() * denominator_b.sqrt()), n))
}

/// Calculates the F-statistic between a feature and a target column.
///
/// This function is generic over any type `T` that implements `FloatOption<F>`,
/// where `F` is a float type (`f32` or `f64`). It uses `pearson_correlation`
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
    T: FloatOption<F>,
    F: Float,
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
