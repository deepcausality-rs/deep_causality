/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr;
use deep_causality_algorithms::mrmr::MrmrError;
use deep_causality_tensor::CausalTensor;

// Skipped under Miri: this test asserts the exact selection order [F2, F0].
// MRMR picks the most-relevant feature first, then the least-redundant. When
// two candidates have nearly equal scores (as here), Miri's soft-float
// emulation drifts the comparison by ~1 ULP and flips the order to [F0, F2].
// The selected set is correct in both cases; only the ordering is unstable
// at the precision boundary. Test is correct and passes under normal CI.
#[cfg_attr(miri, ignore)]
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
fn test_first_feature_relevance_not_finite() {
    // An infinite value in the target column makes EVERY feature's relevance
    // (F-statistic) non-finite, so the very first feature scanned trips the
    // first-feature `relevance.is_finite()` guard. `f64::INFINITY` is not `NaN`,
    // so `FloatOption::to_option` passes it through into the Pearson sums (NaN
    // mapping only catches NaN), producing a non-finite F-statistic.
    let data = vec![
        // F0,  F1,  Target
        1.0,
        2.0,
        f64::INFINITY,
        2.0,
        4.0,
        2.0,
        3.0,
        6.0,
        3.0,
        4.0,
        8.0,
        4.0,
    ];
    let tensor = CausalTensor::new(data, vec![4, 3]).unwrap();
    let result = mrmr::mrmr_features_selector(&tensor, 1, 2);
    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Relevance score for feature")
    );
}

#[test]
fn test_iteration_relevance_not_finite() {
    // The first feature is finite/relevant and is selected; a *later* feature has
    // a non-finite relevance because the target value it correlates against is
    // infinite only on a row where that feature also varies. Here F0 is cleanly
    // relevant (selected first), and F1's relevance becomes non-finite due to the
    // infinite target entry, tripping the iteration-loop relevance guard.
    let data = vec![
        // F0,    F1,  Target
        1.0,
        5.0,
        1.0,
        2.0,
        9.0,
        2.0,
        3.0,
        4.0,
        3.0,
        4.0,
        7.0,
        f64::INFINITY,
    ];
    let tensor = CausalTensor::new(data, vec![4, 3]).unwrap();
    // Request both features; F0 selected first (finite), F1 evaluated in the loop.
    let result = mrmr::mrmr_features_selector(&tensor, 2, 2);
    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("not finite"),
        "expected a non-finite score error, got: {msg}"
    );
}

#[test]
fn test_iteration_correlation_not_finite() {
    // First feature finite/selected; for a later feature the *correlation* with an
    // already-selected feature is non-finite (an infinite value in the candidate
    // feature column, distinct from the target), tripping the redundancy/
    // correlation finiteness guard inside the selection loop.
    let data = vec![
        // F0,    F1 (relevant target proxy), F2 (has inf), Target
        1.0,
        1.0,
        2.0,
        1.0,
        2.0,
        2.0,
        3.0,
        2.0,
        3.0,
        3.0,
        f64::INFINITY,
        3.0,
        4.0,
        4.0,
        9.0,
        4.0,
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    // Target col 3. F1 is perfectly relevant -> selected first. F2 has an infinite
    // entry so its correlation/redundancy with F1 (or its own relevance) is
    // non-finite, exercising a finiteness guard in the iteration loop.
    let result = mrmr::mrmr_features_selector(&tensor, 3, 3);
    assert!(matches!(result, Err(MrmrError::FeatureScoreError(_))));
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("not finite"),
        "expected a non-finite score error, got: {msg}"
    );
}

#[test]
fn test_normalization_skipped_when_max_score_non_positive() {
    // When the single selected feature has zero relevance (a constant feature vs a
    // varying target gives F-statistic 0), `max_score` is 0.0, so the
    // `if max_score > 0.0` normalization branch is skipped (its false arm), and
    // the raw (zero) score is returned unmodified.
    let data = vec![
        // F0 (constant -> zero relevance), Target (varies)
        5.0, 1.0, 5.0, 2.0, 5.0, 3.0, 5.0, 4.0,
    ];
    let tensor = CausalTensor::new(data, vec![4, 2]).unwrap();
    // Only one feature available (F0); select it. Its relevance is 0.0.
    let result = mrmr::mrmr_features_selector(&tensor, 1, 1).unwrap();
    assert_eq!(result.len(), 1);
    let (idx, score) = result.features()[0];
    assert_eq!(idx, 0);
    // Normalization skipped: the score stays at its raw 0.0 value (not divided).
    assert_eq!(score, 0.0);
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
