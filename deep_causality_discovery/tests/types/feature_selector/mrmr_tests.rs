/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_discovery::{
    FeatureSelectError, FeatureSelector, FeatureSelectorConfig, MrmrConfig, MrmrFeatureSelector,
};
use deep_causality_tensor::CausalTensor;

// The Miri gate is gone with the order assertion it protected: F0 and F2 are not "nearly" tied but
// exactly tied, so no precision setting decides between them and there is nothing left for soft-float
// emulation to flip.
#[test]
fn test_mrmr_feature_selector_select() {
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

    // The selected PAIR is determined; the order within it is not. `F2 = 3·F0` exactly, and Pearson
    // is invariant under a positive scaling, so both columns carry the identical F-statistic —
    // `38809/3` over the exact rationals, against `2306332/1303 ≈ 1770` for F1.
    assert_eq!(result_tensor.shape(), &[4, 2]);

    let columns: Vec<Vec<Option<f64>>> = (0..2)
        .map(|c| {
            (0..4)
                .map(|r| result_tensor.as_slice()[r * 2 + c])
                .collect()
        })
        .collect();
    let f0 = vec![Some(1.0), Some(2.0), Some(3.0), Some(4.0)];
    let f2 = vec![Some(3.0), Some(6.0), Some(9.0), Some(12.0)];
    assert!(
        columns.contains(&f0) && columns.contains(&f2),
        "expected the tied pair {{F0, F2}} in either order, got {columns:?}"
    );
}

#[test]
fn test_mrmr_feature_selector_select_error_too_few_features() {
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

    // Requesting more features than available (excluding target)
    let mrmr_config = MrmrConfig::new(4, 3); // 4 features requested, only 3 non-target features available
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);

    let selector = MrmrFeatureSelector;
    let result = selector.select(tensor, &config);

    assert!(result.is_err());
    // The underlying mrmr::select_features returns MrmrError::InvalidInput
    // which is converted to FeatureSelectError::MrmrError
    if let Err(e) = result {
        assert!(matches!(e, FeatureSelectError::MrmrError(_)));
        assert!(
            e.to_string()
                .contains("Invalid number of features requested")
        );
    }
}

#[test]
fn test_mrmr_feature_selector_select_error_invalid_target_col() {
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

    // Target column out of bounds
    let mrmr_config = MrmrConfig::new(2, 4); // target_col = 4, n_cols = 4
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);

    let selector = MrmrFeatureSelector;
    let result = selector.select(tensor, &config);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, FeatureSelectError::MrmrError(_)));
        assert!(e.to_string().contains("Target column index out of bounds"));
    }
}

#[test]
fn test_mrmr_feature_selector_select_error_non_2d_tensor() {
    let tensor = CausalTensor::new(vec![Some(1.0); 4], vec![4]).unwrap(); // 1D tensor

    let mrmr_config = MrmrConfig::new(1, 0);
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);

    let selector = MrmrFeatureSelector;
    let result = selector.select(tensor, &config);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, FeatureSelectError::MrmrError(_)));
        assert!(e.to_string().contains("Input tensor must be 2-dimensional"));
    }
}

#[test]
fn test_mrmr_feature_selector_select_error_sample_too_small() {
    let tensor = CausalTensor::new(vec![Some(1.0); 4], vec![2, 2]).unwrap(); // 2 rows < 3

    let mrmr_config = MrmrConfig::new(1, 1);
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);

    let selector = MrmrFeatureSelector;
    let result = selector.select(tensor, &config);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(matches!(e, FeatureSelectError::MrmrError(_)));
        assert!(
            e.to_string()
                .contains("Sample size is too small. At least 3 samples are required.")
        );
    }
}
