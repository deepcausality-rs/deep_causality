/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashSet;

use deep_causality_tensor::CausalTensor;

use crate::feature_selection::mrmr::MrmrError;
use crate::feature_selection::mrmr::mrmr_utils_cdl;

/// Selects features using the mRMR (Maximum Relevance, Minimum Redundancy) algorithm for data with missing values.
///
/// This implementation is specifically designed to work with `CausalTensor<Option<f64>>`,
/// making it suitable for datasets with missing values, such as medical records. Instead of
/// imputing missing data, this function uses pairwise selection. For any statistical calculation
/// between two variables, only the rows where both variables are present (not `None`) are used.
///
/// # Note
///
/// If multiple features have the same maximum relevance or mRMR score, the selection order is not
/// guaranteed due to the internal use of a `HashSet`. The feature that is encountered first
/// during iteration will be chosen.
///
/// # Arguments
///
/// * `tensor` - A reference to a 2-dimensional `CausalTensor<Option<f64>>`.
/// * `num_features` - The desired number of features to select.
/// * `target_col` - The column index of the target variable within the `tensor`.
///
/// # Returns
///
/// A `Result` containing:
/// * `Ok(Vec<usize>)` - A vector of `usize` representing the indices of the selected features.
/// * `Err(MrmrError)` - An error if the input is invalid or calculations fail.
///
pub fn mrmr_features_selector_cdl(
    tensor: &CausalTensor<Option<f64>>,
    num_features: usize,
    target_col: usize,
) -> Result<Vec<usize>, MrmrError> {
    let shape = tensor.shape();
    if shape.len() != 2 {
        return Err(MrmrError::InvalidInput(
            "Input tensor must be 2-dimensional".to_string(),
        ));
    }

    let n_cols = shape[1];

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

    let mut all_features: HashSet<usize> = (0..n_cols).collect();
    all_features.remove(&target_col);

    if num_features > all_features.len() {
        return Err(MrmrError::NotEnoughFeatures);
    }

    let mut selected_features: Vec<usize> = Vec::with_capacity(num_features);

    // First feature selection based on relevance only
    let mut first_feature = 0;
    let mut max_relevance = -1.0;

    for &feature_idx in &all_features {
        let relevance = mrmr_utils_cdl::f_statistic_cdl(tensor, feature_idx, target_col)?;
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
            let relevance = mrmr_utils_cdl::f_statistic_cdl(tensor, feature_idx, target_col)?;

            let mut redundancy = 0.0;
            for &selected_idx in &selected_features {
                redundancy +=
                    mrmr_utils_cdl::pearson_correlation_cdl(tensor, feature_idx, selected_idx)?
                        .abs();
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
