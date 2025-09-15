/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::feature_selection::mrmr::mrmr_error::MrmrError;
use crate::mrmr::mrmr_utils;
use deep_causality_data_structures::CausalTensor;
use std::collections::HashSet;

/// Selects features using the mRMR (Maximum Relevance, Minimum Redundancy) algorithm.
///
/// This implementation uses the FCQ (F-statistic, Correlation, Quotient) variant, which aims
/// to find a subset of features that are maximally relevant to the target variable and minimally
/// redundant among themselves. The algorithm iteratively selects features based on a score
/// that balances relevance and redundancy.
///
/// Missing values in the input `CausalTensor` are handled by column-mean imputation prior to feature selection.
///
/// # Arguments
///
/// * `tensor` - A mutable reference to a 2-dimensional `CausalTensor<f64>` containing the features and the target variable.
/// * `num_features` - The desired number of features to select.
/// * `target_col` - The column index of the target variable within the `tensor`.
///
///  Missing values (NaN) in the input tensor will be imputed.
///
/// # Returns
///
/// A `Result` containing:
/// * `Ok(Vec<usize>)` - A vector of `usize` representing the indices of the selected features, ranked by their mRMR score.
/// * `Err(MrmrError)` - An error if the input is invalid, sample size is too small, or other calculation issues occur.
///
/// # Errors
///
/// This function can return the following `MrmrError` variants:
/// * `MrmrError::InvalidInput` - If the tensor is not 2-dimensional, `num_features` is invalid, or `target_col` is out of bounds.
/// * `MrmrError::SampleTooSmall` - If the number of rows in the tensor is less than 3, which is insufficient for F-statistic calculation.
/// * `MrmrError::CalculationError` - For numerical issues during statistical calculations (e.g., in `pearson_correlation` or `f_statistic`).
/// * `MrmrError::NotEnoughFeatures` - If the number of features requested is greater than the available features (excluding the target).
///
/// # Examples
///
/// ```
/// use deep_causality_data_structures::CausalTensor;
/// use deep_causality_algorithms::mrmr::select_features;
///
/// let data = vec![
///     1.0, 2.0, 3.0, 1.6,
///     2.0, 4.1, 6.0, 3.5,
///     3.0, 6.2, 9.0, 5.5,
///     4.0, 8.1, 12.0, 7.5,
/// ];
/// let mut tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
///
/// // Select 2 features, with the target variable in column 3.
/// let selected_features = select_features(&mut tensor, 2, 3).unwrap();
/// // The exact output may vary slightly based on floating-point precision and data, but for this example,
/// // it typically selects features 2 and 0 (indices of the original columns).
/// assert_eq!(selected_features.len(), 2);
/// // assert_eq!(selected_features, vec![2, 0]); // Example expected output
/// ```
pub fn select_features(
    tensor: &mut CausalTensor<f64>,
    num_features: usize,
    target_col: usize,
) -> Result<Vec<usize>, MrmrError> {
    let shape = tensor.shape();
    if shape.len() != 2 {
        return Err(MrmrError::InvalidInput(
            "Input tensor must be 2-dimensional".to_string(),
        ));
    }

    let n_rows = shape[0];
    let n_cols = shape[1];

    if n_rows < 3 {
        return Err(MrmrError::SampleTooSmall(3));
    }

    if num_features == 0 || num_features >= n_cols {
        return Err(MrmrError::InvalidInput(
            "Invalid number of features requested".to_string(),
        ));
    }

    if target_col >= n_cols {
        return Err(MrmrError::InvalidInput(
            "Target column index out of bounds".to_string(),
        ));
    }

    mrmr_utils::impute_missing_values(tensor);

    let mut all_features: HashSet<usize> = (0..n_cols).collect();
    all_features.remove(&target_col);

    let mut selected_features: Vec<usize> = Vec::with_capacity(num_features);

    // First feature selection based on relevance only
    let mut first_feature = 0;
    let mut max_relevance = -1.0;

    for &feature_idx in &all_features {
        let relevance = mrmr_utils::f_statistic(tensor, feature_idx, target_col)?;
        if relevance > max_relevance {
            max_relevance = relevance;
            first_feature = feature_idx;
        }
    }

    selected_features.push(first_feature);
    all_features.remove(&first_feature);

    // Iterative selection of remaining features
    while selected_features.len() < num_features {
        let mut best_feature = 0;
        let mut max_mrmr_score = -1.0;

        for &feature_idx in &all_features {
            let relevance = mrmr_utils::f_statistic(tensor, feature_idx, target_col)?;

            let mut redundancy = 0.0;
            for &selected_idx in &selected_features {
                redundancy +=
                    mrmr_utils::pearson_correlation(tensor, feature_idx, selected_idx)?.abs();
            }
            redundancy /= selected_features.len() as f64;

            let mrmr_score = if redundancy == 0.0 {
                if relevance == 0.0 {
                    0.0 // Neither relevant nor redundant
                } else {
                    f64::MAX // Highly relevant, no redundancy
                }
            } else {
                relevance / redundancy
            };

            if mrmr_score > max_mrmr_score {
                max_mrmr_score = mrmr_score;
                best_feature = feature_idx;
            }
        }

        selected_features.push(best_feature);
        all_features.remove(&best_feature);
    }

    Ok(selected_features)
}
