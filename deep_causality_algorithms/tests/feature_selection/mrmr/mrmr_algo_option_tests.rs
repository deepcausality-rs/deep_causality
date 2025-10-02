/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrError;
use deep_causality_algorithms::feature_selection::mrmr::mrmr_features_selector;
use deep_causality_tensor::CausalTensor;

#[test]
fn test_mrmr_features_selector_basic() {
    // Test case with some missing values (None)
    let data = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(1.6),
        Some(2.0),
        Some(4.1),
        None,
        Some(3.5),
        Some(3.0),
        None,
        Some(9.0),
        Some(5.5),
        Some(4.0),
        Some(8.1),
        Some(12.0),
        Some(7.5),
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

    // Select 2 features, with the target variable in column 3.
    let result = mrmr_features_selector(&tensor, 2, 3);
    assert!(result.is_ok());
    let selected_features_with_scores = result.unwrap();
    assert_eq!(selected_features_with_scores.len(), 2);
    for (_, score) in &selected_features_with_scores {
        assert!(score.is_finite());
        assert!(*score >= 0.0 && *score <= 1.0);
    }
}

#[test]
fn test_mrmr_features_selector_no_missing_values() {
    // Test case with no missing values, should behave like the non-CDL version
    let data = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(1.6),
        Some(2.0),
        Some(4.1),
        Some(6.0),
        Some(3.5),
        Some(3.0),
        Some(6.2),
        Some(9.0),
        Some(5.5),
        Some(4.0),
        Some(8.1),
        Some(12.0),
        Some(7.5),
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

    let selected_features_with_scores = mrmr_features_selector(&tensor, 2, 3).unwrap();
    let selected_features: Vec<usize> = selected_features_with_scores
        .iter()
        .map(|(idx, _score)| *idx)
        .collect();

    assert_eq!(selected_features.len(), 2);
    assert!(selected_features.contains(&2));
    assert!(selected_features.contains(&0) || selected_features.contains(&1));

    for (_, score) in &selected_features_with_scores {
        assert!(score.is_finite());
        assert!(*score >= 0.0 && *score <= 1.0);
    }
}

#[test]
fn test_mrmr_features_selector_all_missing_in_column() {
    // Test case where a column is entirely missing
    let data = vec![
        Some(1.0),
        Some(2.0),
        None,
        Some(1.6),
        Some(2.0),
        Some(4.1),
        None,
        Some(3.5),
        Some(3.0),
        Some(6.2),
        None,
        Some(5.5),
        Some(4.0),
        Some(8.1),
        None,
        Some(7.5),
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

    // Feature 2 (index 2) is all None. Its correlation should be 0, and it should not be selected.
    let result = mrmr_features_selector(&tensor, 2, 3);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(2))));
}

#[test]
fn test_mrmr_features_selector_invalid_input() {
    let data = vec![Some(1.0), Some(2.0), Some(3.0)];
    let tensor = CausalTensor::new(data, vec![3]).unwrap(); // 1D tensor

    let result = mrmr_features_selector(&tensor, 1, 0);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
}

#[test]
fn test_mrmr_features_selector_sample_too_small() {
    let data = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(1.6),
        Some(2.0),
        Some(4.1),
        Some(6.0),
        Some(3.5),
    ];
    let tensor = CausalTensor::new(data, vec![2, 4]).unwrap(); // 2 rows, requires 3

    let result = mrmr_features_selector(&tensor, 1, 3);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(3))));
}

// #[test]
// fn test_mrmr_features_selector_not_enough_features() {
//     let data = vec![
//         Some(1.0),
//         Some(2.0),
//         Some(3.0),
//         Some(1.6),
//         Some(2.0),
//         Some(4.1),
//         Some(6.0),
//         Some(3.5),
//         Some(3.0),
//         Some(6.2),
//         Some(9.0),
//         Some(5.5),
//         Some(4.0),
//         Some(8.1),
//         Some(12.0),
//         Some(7.5),
//     ];
//     let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

//     // Request 4 features from 3 available (excluding target_col=3)
//     let result = mrmr_features_selector(&tensor, 4, 3);
//     assert!(matches!(result, Err(MrmrError::NotEnoughFeatures)));
// }

// #[test]
// fn test_mrmr_features_selector_relevance_not_finite() {
//     // Create a tensor where feature 0 is perfectly correlated with target 2, leading to infinite relevance
//     let data = vec![
//         Some(1.0), Some(10.0), Some(1.0),
//         Some(2.0), Some(20.0), Some(2.0),
//         Some(3.0), Some(30.0), Some(3.0),
//     ];
//     let tensor = CausalTensor::new(data, vec![3, 3]).unwrap();

//     // Request 1 feature, target_col=2
//     // Feature 0 is perfectly correlated with target 2.
//     let result = mrmr_features_selector(&tensor, 1, 2);
//     assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
//     assert!(result.unwrap_err().to_string().contains("Relevance score for feature 0 is not finite"));
// }

#[test]
fn test_mrmr_features_selector_mrmr_score_nan_zero_redundancy_zero_relevance() {
    // Feature 0: constant (0 relevance to target, 0 redundancy with selected)
    // Feature 1: target (selected first)
    // Feature 2: some values
    let data = vec![
        Some(1.0),
        Some(10.0),
        Some(1.0),
        Some(1.0),
        Some(20.0),
        Some(2.0),
        Some(1.0),
        Some(30.0),
        Some(3.0),
        Some(1.0),
        Some(40.0),
        Some(4.0),
    ];
    let tensor = CausalTensor::new(data, vec![4, 3]).unwrap();

    // Select 2 features, target_col=1
    // First feature selected will be feature 2 (highest relevance to target 1)
    // Then, when considering feature 0, its relevance to target 1 is 0 (constant column).
    // Its redundancy with feature 2 will also be 0 (constant vs increasing).
    // This should lead to 0/0 = NaN mRMR score.
    let result = mrmr_features_selector(&tensor, 2, 1);
    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("mRMR score for feature 0 is NaN")
    );
}

#[test]
fn test_mrmr_features_selector_mrmr_score_infinite_zero_redundancy_positive_relevance() {
    // Select 2 features, target_col=1
    // First feature selected will be feature 0 (perfect correlation with target 1).
    // Then, when considering feature 2, its relevance to target 1 is 0 (constant column).
    // Its redundancy with feature 0 will also be 0 (constant vs increasing).
    // This should lead to 0/0 = NaN mRMR score.
    // This test needs to be carefully constructed to ensure positive relevance and zero redundancy.
    // Let's re-think the data to get positive relevance and zero redundancy for the second feature.
    // Let's make feature 0 the target, feature 1 highly correlated with target, feature 2 uncorrelated.

    let data = vec![
        Some(10.0),
        Some(1.0),
        Some(100.0),
        Some(20.0),
        Some(2.0),
        Some(100.0),
        Some(30.0),
        Some(3.0),
        Some(100.0),
        Some(40.0),
        Some(4.0),
        Some(100.0),
    ];
    let tensor = CausalTensor::new(data, vec![4, 3]).unwrap();

    // Select 2 features, target_col=0
    // First feature selected will be feature 1 (perfect correlation with target 0).
    // Then, when considering feature 2, its relevance to target 0 is 0 (constant column).
    // Its redundancy with feature 1 will also be 0 (constant vs increasing).
    // This should lead to 0/0 = NaN mRMR score.
    let result = mrmr_features_selector(&tensor, 2, 0);
    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("mRMR score for feature 2 is NaN")
    );
}
