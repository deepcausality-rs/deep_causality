/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr;
use deep_causality_algorithms::mrmr::MrmrError;
use deep_causality_tensor::CausalTensor;

#[test]
fn test_mrmr_select_features() {
    let data = vec![
        // F0,  F1,  F2,  Target
        1.0, 2.0, 3.0, 1.6, 2.0, 4.1, 6.0, 3.5, 3.0, 6.2, 9.0, 5.5, 4.0, 8.1, 12.0, 7.5,
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    let selected_features_with_scores = mrmr::mrmr_features_selector(&tensor, 2, 3).unwrap();

    // Extract indices for assertion
    let selected_features: Vec<usize> = selected_features_with_scores
        .iter()
        .map(|(idx, _score)| *idx)
        .collect();

    // Based on calculation, F2 is most relevant, then F0 is chosen due to lower redundancy.
    assert_eq!(selected_features, vec![2, 0]);

    // Verify scores are valid numbers and normalized
    for (_, score) in selected_features_with_scores {
        assert!(score.is_finite());
        assert!((0.0..=1.0).contains(&score));
    }
}

#[test]
fn test_select_features_non_2d_tensor() {
    let tensor = CausalTensor::new(vec![1.0; 4], vec![4]).unwrap();
    let result = mrmr::mrmr_features_selector(&tensor, 1, 0);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Input tensor must be 2-dimensional"
    );
}

#[test]
fn test_select_features_sample_too_small() {
    let tensor = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap(); // 2 rows < 3
    let result = mrmr::mrmr_features_selector(&tensor, 1, 1);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(3))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Sample size is too small. At least 3 samples are required."
    );
}

#[test]
fn test_select_features_invalid_num_features_zero() {
    let tensor = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = mrmr::mrmr_features_selector(&tensor, 0, 2); // num_features = 0
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Invalid number of features requested"
    );
}

#[test]
fn test_select_features_invalid_num_features_too_large() {
    let tensor = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = mrmr::mrmr_features_selector(&tensor, 3, 2); // num_features = 3, n_cols = 3
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Invalid number of features requested"
    );
}

#[test]
fn test_select_features_target_col_out_of_bounds() {
    let tensor = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = mrmr::mrmr_features_selector(&tensor, 1, 3); // target_col = 3, n_cols = 3
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Target column index out of bounds"
    );
}

#[test]
fn test_select_features_zero_relevance_and_redundancy() {
    let data = vec![
        // F0 (relevant), F1 (irrelevant/uncorrelated), F2 (relevant but less than F0), Target
        1.0, 1.0, 4.0, 1.0, 2.0, -1.0, 3.0, 2.0, 3.0, -1.0, 2.0, 3.0, 4.0, 1.0, 1.1,
        4.0, // F2 is now slightly noisy
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    // Select 2 features, target is col 3
    let result = mrmr::mrmr_features_selector(&tensor, 2, 3);

    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    assert!(result.unwrap_err().to_string().contains("NaN"));
}

#[test]
fn test_select_features_highly_relevant_no_redundancy() {
    let data = vec![
        // F0 (high rel), F1 (low rel, orthogonal), F2 (high rel, redundant), Target
        1.0, 1.0, 1.1, 3.0, 1.0, -1.0, 0.9, 1.0, -1.0, 1.0, -1.1, -1.0, -1.0, -1.0, -0.9, -3.0,
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    // Select 2 features, target is col 3
    let result = mrmr::mrmr_features_selector(&tensor, 2, 3);

    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    assert!(result.unwrap_err().to_string().contains("infinite"));
}

#[test]
fn test_select_features_nan_score_error() {
    // Create a tensor where F0 and F1 have zero relevance to the target,
    // and F1 has zero redundancy with F0.
    // This should lead to relevance = 0.0 and redundancy = 0.0 for F1,
    // causing mRMR score to be NaN (0.0 / 0.0).
    let data = vec![
        // F0,  F1,  F2,  Target
        1.0, 1.0, 5.0, 1.0, // F0 and F1 are constant, F2 varies, Target varies
        1.0, 1.0, 6.0, 2.0, 1.0, 1.0, 7.0, 3.0, 1.0, 1.0, 8.0, 4.0,
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

    // Select 2 features, target is col 3.
    // F0 will be selected first (relevance 0.0).
    // Then F1 will have relevance 0.0 and redundancy 0.0 with F0.
    let result = mrmr::mrmr_features_selector(&tensor, 2, 3);

    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    assert!(result.unwrap_err().to_string().contains("NaN"));
}

#[test]
fn test_select_features_infinite_score_error() {
    // Create a tensor where a feature's F-statistic might become infinite
    // This is harder to reliably trigger with simple data, but can happen if
    // the 'within-group' variance is zero.
    // For now, let's simulate a scenario where redundancy is 0 and relevance is high.
    // The current algo handles this by returning f64::MAX, but the new spec says it should error.
    let data = vec![
        // F0 (high rel), F1 (low rel, orthogonal), F2 (high rel, redundant), Target
        1.0, 1.0, 1.1, 3.0, 1.0, -1.0, 0.9, 1.0, -1.0, 1.0, -1.1, -1.0, -1.0, -1.0, -0.9, -3.0,
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

    // Select 2 features, target is col 3.
    // F0 is selected first.
    // F1 is orthogonal to F0, so its redundancy with F0 is 0.
    // If F1's relevance is high, its mRMR score would be infinite (relevance / 0).
    let result = mrmr::mrmr_features_selector(&tensor, 2, 3);

    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    assert!(result.unwrap_err().to_string().contains("infinite"));
}
