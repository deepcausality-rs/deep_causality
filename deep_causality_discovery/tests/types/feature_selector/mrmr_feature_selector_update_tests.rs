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

    // The selected PAIR is determined; the order within it is not. `F2 = 3·F0` exactly, and Pearson
    // is invariant under a positive scaling, so both columns carry the identical F-statistic —
    // `38809/3` over the exact rationals, against `2306332/1303 ≈ 1770` for F1. Which of the two
    // tied columns is emitted first is decided by rounding, so asserting an order would be pinning
    // noise. Mirrors the sibling assertion in
    // `deep_causality_algorithms::mrmr::test_mrmr_select_features`.
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
