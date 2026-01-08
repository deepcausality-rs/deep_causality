/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    FeatureSelectError, FeatureSelector, FeatureSelectorConfig, MrmrConfig, MrmrFeatureSelector,
};
use deep_causality_tensor::CausalTensor;

#[test]
fn test_mrmr_feature_selector_with_option_f64_tensor() {
    let data = vec![
        // F0,  F1,  F2,  Target
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

    let mrmr_config = MrmrConfig::new(2, 3); // Select 2 features, target is column 3
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);

    let selector = MrmrFeatureSelector;
    let result_tensor = selector.select(tensor, &config).unwrap();

    // Expected selected features from deep_causality_algorithms::mrmr::select_features are [2, 0]
    // This means the new tensor should contain columns F2 and F0 in that order.
    let expected_data = vec![
        Some(3.0),
        Some(1.0), // F2, F0 for row 0
        Some(6.0),
        Some(2.0), // F2, F0 for row 1
        Some(9.0),
        Some(3.0), // F2, F0 for row 2
        Some(12.0),
        Some(4.0), // F2, F0 for row 3
    ];
    let expected_shape = vec![4, 2];
    let expected_tensor = CausalTensor::new(expected_data, expected_shape).unwrap();

    assert_eq!(result_tensor.as_slice(), expected_tensor.as_slice());
    assert_eq!(result_tensor.shape(), expected_tensor.shape());
}

#[test]
fn test_mrmr_feature_selector_with_option_f64_tensor_and_nans() {
    let data = vec![
        Some(1.0),
        Some(2.0),
        Some(3.0),
        Some(1.6),
        Some(2.0),
        None,
        Some(6.0),
        Some(3.5),
        Some(3.0),
        Some(6.2),
        Some(9.0),
        None,
        Some(4.0),
        Some(8.1),
        None,
        Some(7.5),
    ];
    let tensor = CausalTensor::new(data, vec![4, 4]).unwrap();

    let mrmr_config = MrmrConfig::new(2, 3); // Select 2 features, target is column 3
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);

    let selector = MrmrFeatureSelector;
    let result = selector.select(tensor, &config);

    assert!(result.is_err());
    if let Err(FeatureSelectError::MrmrError(e)) = result {
        assert_eq!(
            e,
            deep_causality_algorithms::mrmr::MrmrError::SampleTooSmall(3)
        );
    } else {
        panic!("Expected MrmrError::SampleTooSmall(3), got {:?}", result);
    }
}
