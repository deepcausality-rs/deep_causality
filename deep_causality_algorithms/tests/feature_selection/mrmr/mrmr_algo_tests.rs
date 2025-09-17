/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr;
use deep_causality_algorithms::mrmr::MrmrError;
use deep_causality_data_structures::CausalTensor;

#[test]
fn test_mrmr_select_features() {
    let data = vec![
        // F0,  F1,  F2,  Target
        1.0, 2.0, 3.0, 1.6, 2.0, 4.1, 6.0, 3.5, 3.0, 6.2, 9.0, 5.5, 4.0, 8.1, 12.0, 7.5,
    ];
    let mut tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    let selected_features = mrmr::select_features(&mut tensor, 2, 3).unwrap();

    // Based on calculation, F2 is most relevant, then F0 is chosen due to lower redundancy.
    assert_eq!(selected_features, vec![2, 0]);
}

#[test]
fn test_select_features_non_2d_tensor() {
    let mut tensor = CausalTensor::new(vec![1.0; 4], vec![4]).unwrap();
    let result = mrmr::select_features(&mut tensor, 1, 0);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Input tensor must be 2-dimensional"
    );
}

#[test]
fn test_select_features_sample_too_small() {
    let mut tensor = CausalTensor::new(vec![1.0; 4], vec![2, 2]).unwrap(); // 2 rows < 3
    let result = mrmr::select_features(&mut tensor, 1, 1);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(3))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Sample size is too small. At least 3 samples are required."
    );
}

#[test]
fn test_select_features_invalid_num_features_zero() {
    let mut tensor = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = mrmr::select_features(&mut tensor, 0, 2); // num_features = 0
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Invalid number of features requested"
    );
}

#[test]
fn test_select_features_invalid_num_features_too_large() {
    let mut tensor = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = mrmr::select_features(&mut tensor, 3, 2); // num_features = 3, n_cols = 3
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid input: Invalid number of features requested"
    );
}

#[test]
fn test_select_features_target_col_out_of_bounds() {
    let mut tensor = CausalTensor::new(vec![1.0; 9], vec![3, 3]).unwrap();
    let result = mrmr::select_features(&mut tensor, 1, 3); // target_col = 3, n_cols = 3
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
    let mut tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    // Select 2 features, target is col 3
    let selected_features = mrmr::select_features(&mut tensor, 2, 3).unwrap();

    // F0 is selected first (perfect relevance).
    // F2 has slightly lower relevance due to noise.
    // Then, F2 is chosen over F1 because F1 has a score of 0 (relevance=0, redundancy=0),
    // while F2 has a positive score.
    assert_eq!(selected_features, vec![0, 2]);
}

#[test]
fn test_select_features_highly_relevant_no_redundancy() {
    let data = vec![
        // F0 (high rel), F1 (low rel, orthogonal), F2 (high rel, redundant), Target
        1.0, 1.0, 1.1, 3.0, 1.0, -1.0, 0.9, 1.0, -1.0, 1.0, -1.1, -1.0, -1.0, -1.0, -0.9, -3.0,
    ];
    let mut tensor = CausalTensor::new(data, vec![4, 4]).unwrap();
    // Select 2 features, target is col 3
    let selected_features = mrmr::select_features(&mut tensor, 2, 3).unwrap();

    // F0 is selected first (highest relevance: F-stat approx 8.0).
    // F2 has slightly lower relevance (F-stat approx 7.6)
    // F1 has lowest relevance (F-stat approx 0.5)
    //
    // For the second feature:
    // Candidate F1 has redundancy of 0 with F0, so its score is f64::MAX.
    // Candidate F2 has redundancy of ~0.995 with F0, so its score is ~7.6.
    // F1 is chosen.
    assert_eq!(selected_features, vec![0, 1]);
}
