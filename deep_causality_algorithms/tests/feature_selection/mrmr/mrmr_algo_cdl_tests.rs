/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrError;
use deep_causality_algorithms::feature_selection::mrmr::mrmr_features_selector_cdl;
use deep_causality_tensor::CausalTensor;

#[test]
fn test_mrmr_features_selector_cdl_basic() {
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
    let result = mrmr_features_selector_cdl(&tensor, 2, 3);
    // dbg!(&result);
    assert!(result.is_ok());
}

#[test]
fn test_mrmr_features_selector_cdl_no_missing_values() {
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

    let selected_features = mrmr_features_selector_cdl(&tensor, 2, 3).unwrap();

    assert_eq!(selected_features.len(), 2);
    assert!(selected_features.contains(&2));
    assert!(selected_features.contains(&0) || selected_features.contains(&1));
}

#[test]
fn test_mrmr_features_selector_cdl_all_missing_in_column() {
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
    let result = mrmr_features_selector_cdl(&tensor, 2, 3);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(2))));
}

#[test]
fn test_mrmr_features_selector_cdl_invalid_input() {
    let data = vec![Some(1.0), Some(2.0), Some(3.0)];
    let tensor = CausalTensor::new(data, vec![3]).unwrap(); // 1D tensor

    let result = mrmr_features_selector_cdl(&tensor, 1, 0);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
}

#[test]
fn test_mrmr_features_selector_cdl_sample_too_small() {
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

    let result = mrmr_features_selector_cdl(&tensor, 1, 3);
    assert!(matches!(result, Err(MrmrError::SampleTooSmall(3))));
}

#[test]
fn test_mrmr_features_selector_cdl_not_enough_features() {
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

    // Request 4 features from 3 available (excluding target_col=3)
    let result = mrmr_features_selector_cdl(&tensor, 4, 3);
    assert!(matches!(result, Err(MrmrError::InvalidInput(_))));
}
