/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashSet;

use deep_causality_tensor::CausalTensor;

use crate::feature_selection::mrmr::MrmrError;
use crate::feature_selection::mrmr::mrmr_utils_cdl;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Selects features using the mRMR (Maximum Relevance, Minimum Redundancy) algorithm for data with missing values.
///
/// This implementation is specifically designed to work with `CausalTensor<Option<f64>>`,
/// making it suitable for datasets with missing values, such as medical records. Instead of
/// imputing missing data, this function uses pairwise selection. For any statistical calculation
/// between two variables, only the rows where both variables are present (not `None`) are used.
///
/// When compiled with the `parallel` feature flag, the main feature selection loops are parallelized using `rayon`
/// to accelerate computation on multi-core systems.
///
/// The algorithm iteratively selects features based on a score that balances relevance and redundancy.
/// The scoring mechanism is as follows:
/// 1. **First Feature**: The feature with the highest relevance (F-statistic) to the target variable is selected. Its score is this F-statistic.
/// 2. **Subsequent Features**: For the remaining features, an mRMR score is calculated as `Relevance / Redundancy`. The feature with the highest mRMR score is chosen. Its score is this mRMR value.
///
/// The score is normalized as percentage within the range of 0...1.
///
/// Since the mRMR score is calculated as Relevance / Redundancy (both of which are non-negative),
/// the final mRMR score itself will always positive It quantifies the strength of
/// relevance and redundancy, not the direction of correlation between features.
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
/// * `Ok(Vec<(usize, f64)>)` - A vector of `(feature_index, score)` tuples, representing the indices of the selected features and their corresponding mRMR scores, ranked by their selection order.
/// * `Err(MrmrError)` - An error if the input is invalid or calculations fail.
///
/// # Errors
///
/// This function can return the following `MrmrError` variants:
/// * `MrmrError::InvalidInput` - If the tensor is not 2-dimensional, `num_features` is invalid, or `target_col` is out of bounds.
/// * `MrmrError::NotEnoughFeatures` - If the number of features requested is greater than the available features (excluding the target).
/// * `MrmrError::FeatureScoreError` - If a calculated relevance, redundancy, or mRMR score is `NaN` or `Infinity`.
///
/// # Examples
///
/// ```
/// use deep_causality_tensor::CausalTensor;
/// use deep_causality_algorithms::mrmr::mrmr_features_selector_cdl;
///
/// let data = vec![
///     Some(1.0), Some(2.0), Some(3.0), Some(1.6),
///     Some(2.0), Some(4.1), Some(6.0), Some(3.5),
///     Some(3.0), Some(6.2), Some(9.0), Some(5.5),
///     Some(4.0), Some(8.1), Some(12.0), Some(7.5),
/// ];
/// let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
///
/// // Select 2 features, with the target variable in column 3.
/// let selected_features_with_scores = mrmr_features_selector_cdl(&tensor, 2, 3).unwrap();
/// // The exact output may vary slightly based on floating-point precision and data,
/// // but for this example, it typically selects features 2 and 0 (indices of the original columns).
/// assert_eq!(selected_features_with_scores.len(), 2);
/// // assert_eq!(selected_features_with_scores[0].0, 2); // Example expected output for index
/// // assert!(selected_features_with_scores[0].1.is_finite()); // Example expected output for score
/// ```
pub fn mrmr_features_selector_cdl(
    tensor: &CausalTensor<Option<f64>>,
    num_features: usize,
    target_col: usize,
) -> Result<Vec<(usize, f64)>, MrmrError> {
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

    let mut selected_features_with_scores: Vec<(usize, f64)> = Vec::with_capacity(num_features);

    // First feature selection based on relevance only
    #[cfg(feature = "parallel")]
    let (first_feature, max_relevance) = {
        let features: Vec<usize> = all_features.iter().copied().collect();
        features
            .into_par_iter()
            .map(|feature_idx| {
                let relevance = mrmr_utils_cdl::f_statistic_cdl(tensor, feature_idx, target_col)?;
                if !relevance.is_finite() {
                    Err(MrmrError::FeatureScoreError(format!(
                        "Relevance score for feature {} is not finite: {}",
                        feature_idx, relevance
                    )))
                } else {
                    Ok((feature_idx, relevance))
                }
            })
            .reduce(
                || Ok((0, -1.0)),
                |acc, res| {
                    let acc = acc?;
                    let res = res?;
                    if res.1 > acc.1 { Ok(res) } else { Ok(acc) }
                },
            )?
    };

    #[cfg(not(feature = "parallel"))]
    let (first_feature, max_relevance) = {
        let mut first_feature = 0;
        let mut max_relevance = -1.0;

        for &feature_idx in &all_features {
            let relevance = mrmr_utils_cdl::f_statistic_cdl(tensor, feature_idx, target_col)?;
            if !relevance.is_finite() {
                return Err(MrmrError::FeatureScoreError(format!(
                    "Relevance score for feature {} is not finite: {}",
                    feature_idx, relevance
                )));
            }
            if relevance > max_relevance {
                max_relevance = relevance;
                first_feature = feature_idx;
            }
        }
        (first_feature, max_relevance)
    };

    selected_features_with_scores.push((first_feature, max_relevance));
    all_features.remove(&first_feature);

    // Iterative selection of remaining features
    while selected_features_with_scores.len() < num_features {
        #[cfg(feature = "parallel")]
        let (best_feature, best_feature_score) = {
            let features: Vec<usize> = all_features.iter().copied().collect();
            features
                .into_par_iter()
                .map(|feature_idx| {
                    let relevance = mrmr_utils_cdl::f_statistic_cdl(tensor, feature_idx, target_col)?;
                    if !relevance.is_finite() {
                        return Err(MrmrError::FeatureScoreError(format!(
                            "Relevance score for feature {} is not finite: {}",
                            feature_idx, relevance
                        )));
                    }

                    let selected_indices: Vec<usize> = selected_features_with_scores
                        .iter()
                        .map(|(idx, _)| *idx)
                        .collect();

                    let redundancy: f64 = selected_indices
                        .par_iter()
                        .map(|&selected_idx| {
                            mrmr_utils_cdl::pearson_correlation_cdl(tensor, feature_idx, selected_idx)
                                .map(|(corr, _)| corr.abs())
                        })
                        .sum::<Result<f64, _>>()?;

                    let redundancy = redundancy / selected_indices.len() as f64;

                    let mrmr_score = if redundancy == 0.0 {
                        if relevance == 0.0 {
                            return Err(MrmrError::FeatureScoreError(format!(
                                "mRMR score for feature {} is NaN (relevance {} / redundancy {}).",
                                feature_idx, relevance, redundancy
                            )));
                        } else {
                            return Err(MrmrError::FeatureScoreError(format!(
                                "mRMR score for feature {} is infinite (relevance {} / redundancy {}).",
                                feature_idx, relevance, redundancy
                            )));
                        }
                    } else {
                        relevance / redundancy
                    };

                    if !mrmr_score.is_finite() {
                        return Err(MrmrError::FeatureScoreError(format!(
                            "mRMR score for feature {} is not finite: {}",
                            feature_idx, mrmr_score
                        )));
                    }

                    Ok((feature_idx, mrmr_score))
                })
                .reduce(
                    || Ok((0, -1.0)),
                    |acc, res| {
                        let acc = acc?;
                        let res = res?;
                        if res.1 > acc.1 {
                            Ok(res)
                        } else {
                            Ok(acc)
                        }
                    },
                )?
        };

        #[cfg(not(feature = "parallel"))]
        let (best_feature, best_feature_score) = {
            let mut best_feature = 0;
            let mut max_mrmr_score = -1.0;

            for &feature_idx in &all_features {
                let relevance = mrmr_utils_cdl::f_statistic_cdl(tensor, feature_idx, target_col)?;
                if !relevance.is_finite() {
                    return Err(MrmrError::FeatureScoreError(format!(
                        "Relevance score for feature {} is not finite: {}",
                        feature_idx, relevance
                    )));
                }

                let mut redundancy = 0.0;
                let selected_indices: Vec<usize> = selected_features_with_scores
                    .iter()
                    .map(|(idx, _)| *idx)
                    .collect();

                for &selected_idx in &selected_indices {
                    redundancy +=
                        mrmr_utils_cdl::pearson_correlation_cdl(tensor, feature_idx, selected_idx)
                            .map(|(corr, _)| corr.abs())?
                }
                redundancy /= selected_indices.len() as f64;

                let mrmr_score = if redundancy == 0.0 {
                    if relevance == 0.0 {
                        return Err(MrmrError::FeatureScoreError(format!(
                            "mRMR score for feature {} is NaN (relevance {} / redundancy {}).",
                            feature_idx, relevance, redundancy
                        )));
                    } else {
                        return Err(MrmrError::FeatureScoreError(format!(
                            "mRMR score for feature {} is infinite (relevance {} / redundancy {}).",
                            feature_idx, relevance, redundancy
                        )));
                    }
                } else {
                    relevance / redundancy
                };

                if !mrmr_score.is_finite() {
                    return Err(MrmrError::FeatureScoreError(format!(
                        "mRMR score for feature {} is not finite: {}",
                        feature_idx, mrmr_score
                    )));
                }

                if mrmr_score > max_mrmr_score {
                    max_mrmr_score = mrmr_score;
                    best_feature = feature_idx;
                }
            }
            (best_feature, max_mrmr_score)
        };

        selected_features_with_scores.push((best_feature, best_feature_score));
        all_features.remove(&best_feature);
    }

    // Normalization step
    let max_score = selected_features_with_scores
        .iter()
        .map(|(_, score)| *score)
        .fold(f64::MIN, |acc, score| acc.max(score));

    if max_score > 0.0 {
        for (_, score) in &mut selected_features_with_scores {
            *score /= max_score;
        }
    }

    Ok(selected_features_with_scores)
}
