/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::{MrmrError, MrmrResult};
use deep_causality_discovery::{CDL, CdlConfig, WithCleanedData};
use deep_causality_tensor::CausalTensor;

fn create_cdl_with_cleaned_data() -> CDL<WithCleanedData> {
    // 2 rows, 2 cols (0 and 1)
    // Row 0: [1.0, None]
    // Row 1: [3.0, 4.0]
    let tensor =
        CausalTensor::new(vec![Some(1.0), None, Some(3.0), Some(4.0)], vec![2, 2]).unwrap();
    CDL {
        state: WithCleanedData {
            tensor,
            records_count: 2,
        },
        config: CdlConfig::default(),
    }
}

#[test]
fn test_feature_select_success() {
    let cdl = create_cdl_with_cleaned_data();

    // Select column 0
    let res = cdl.feature_select(|_t| Ok(MrmrResult::new(vec![(0, 0.5)])));

    assert!(res.inner.is_ok());
    let with_feats = res.inner.unwrap();
    // Shape should be [2, 1]
    assert_eq!(with_feats.state.tensor.shape(), &[2, 1]);
    let data = with_feats.state.tensor.as_slice();
    // Row 0, Col 0 -> 1.0
    assert_eq!(data[0], Some(1.0));
    // Row 1, Col 0 -> 3.0
    assert_eq!(data[1], Some(3.0));
}

#[test]
fn test_feature_select_error() {
    let cdl = create_cdl_with_cleaned_data();
    let res = cdl.feature_select(|_t| Err(MrmrError::InvalidInput("Err".into())));
    assert!(res.inner.is_err());
}

#[test]
fn test_feature_select_invalid_indices_fallback() {
    let cdl = create_cdl_with_cleaned_data();
    // Tensor shape [2, 2]

    // Select index 99 (out of bounds)
    let res = cdl.feature_select(|_t| Ok(MrmrResult::new(vec![(99, 0.5)])));

    assert!(res.inner.is_ok());
    let with_feats = res.inner.unwrap();
    // Shape should be [2, 1]
    assert_eq!(with_feats.state.tensor.shape(), &[2, 1]);

    let data = with_feats.state.tensor.as_slice();
    // Should be filled with None
    assert_eq!(data[0], None);
    assert_eq!(data[1], None);
}
