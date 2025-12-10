/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_discovery::{
    CDL, CdlConfig, DataPreprocessor, OptionNoneDataCleaner, PreprocessConfig, PreprocessError,
    WithData,
};
use deep_causality_tensor::CausalTensor;

// Mock Preprocessor
struct MockPreprocessor;
impl DataPreprocessor for MockPreprocessor {
    fn process(
        &self,
        tensor: CausalTensor<f64>,
        _config: &PreprocessConfig,
    ) -> Result<CausalTensor<f64>, PreprocessError> {
        // Just return tensor as is for mock
        Ok(tensor)
    }
}

// Helper to create a CDL<WithData> instance
fn create_cdl_with_data() -> CDL<WithData> {
    let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0, 4.0], vec![2, 2]).unwrap();
    CDL {
        state: WithData {
            tensor,
            records_count: 2,
        },
        config: CdlConfig::default(),
    }
}

#[test]
fn test_clean_data_success() {
    let cdl = create_cdl_with_data();
    let res = cdl.clean_data(OptionNoneDataCleaner);
    assert!(res.inner.is_ok());

    // Check type transition indirectly via map or extraction
    let cdl_cleaned = res.inner.unwrap();
    let data = cdl_cleaned.state.tensor.as_slice();
    assert_eq!(data[0], Some(1.0));
}

#[test]
fn test_preprocess_pass_through_no_config() {
    let cdl = create_cdl_with_data();
    // No preprocess config in default CDLConfig
    // Should return pure(self)
    let res = cdl.preprocess(MockPreprocessor);
    assert!(res.inner.is_ok());
}

#[test]
fn test_feature_select_success() {
    let cdl = create_cdl_with_data();

    let res = cdl.feature_select(|_t| Ok(MrmrResult::new(vec![(0, 1.0)])));

    assert!(res.inner.is_ok());
    let cdl_feats = res.inner.unwrap();
    // 2 rows, select 1 col -> 2 elements
    assert_eq!(cdl_feats.state.tensor.shape(), &[2, 1]);
}
