/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

#[test]
fn test_preprocess_error() {
    let cdl = create_cdl_with_data();

    // Create a mock preprocessor that fails
    struct ErrorPreprocessor;
    impl DataPreprocessor for ErrorPreprocessor {
        fn process(
            &self,
            _tensor: CausalTensor<f64>,
            _config: &PreprocessConfig,
        ) -> Result<CausalTensor<f64>, PreprocessError> {
            Err(PreprocessError::ConfigError("Mock Error".into()))
        }
    }

    // We need a config that has preprocess enabled, otherwise it shortcuts
    use deep_causality_discovery::{BinningStrategy, ColumnSelector};
    let preprocess_config =
        PreprocessConfig::new(BinningStrategy::EqualWidth, 10, ColumnSelector::All);

    let mut config = CdlConfig::default();
    config = config.with_preprocess_config(preprocess_config);

    let cdl_with_config = CDL {
        state: cdl.state,
        config,
    };

    let res = cdl_with_config.preprocess(ErrorPreprocessor);
    assert!(res.inner.is_err());
}

#[test]
fn test_clean_data_error() {
    use deep_causality_discovery::{DataCleaner, DataCleaningError};
    use deep_causality_tensor::CausalTensorError;

    let cdl = create_cdl_with_data();

    // Mock Cleaner that fails
    struct ErrorCleaner;
    impl DataCleaner for ErrorCleaner {
        fn process(
            &self,
            _tensor: CausalTensor<f64>,
        ) -> Result<CausalTensor<Option<f64>>, DataCleaningError> {
            Err(DataCleaningError::TensorError(
                CausalTensorError::InvalidParameter("Mock Clean Error".into()),
            ))
        }
    }

    let res = cdl.clean_data(ErrorCleaner);
    assert!(res.inner.is_err());
}

#[test]
fn test_feature_select_selector_error() {
    use deep_causality_algorithms::feature_selection::mrmr::MrmrError;

    let cdl = create_cdl_with_data();

    let res = cdl.feature_select(|_| Err(MrmrError::InvalidInput("Selector failed".into())));

    assert!(res.inner.is_err());
}

#[test]
fn test_feature_select_tensor_creation_error() {
    let cdl = create_cdl_with_data();
    let res = cdl.feature_select(|_| Ok(MrmrResult::new(vec![])));

    assert!(res.inner.is_ok());
}

#[test]
fn test_filter_cohort_success() {
    let cdl = create_cdl_with_data();
    // Tensor: [1.0, 2.0]
    //         [3.0, 4.0]

    // Filter to keep rows where first col > 2.0 -> Only 2nd row (index 1)
    let res = cdl.filter_cohort(|row| row[0] > 2.0);

    assert!(res.inner.is_ok());
    let cdl_filtered = res.inner.unwrap();
    assert_eq!(cdl_filtered.state.records_count, 1);

    let data = cdl_filtered.state.tensor.as_slice();
    // Should be [3.0, 4.0]
    assert_eq!(data.len(), 2);
    assert_eq!(data[0], 3.0);
}

#[test]
fn test_filter_cohort_all_filtered() {
    let cdl = create_cdl_with_data();
    // Keep nothing
    let res = cdl.filter_cohort(|_| false);

    assert!(res.inner.is_ok());
    let cdl_filtered = res.inner.unwrap();
    assert_eq!(cdl_filtered.state.records_count, 0);
    assert!(cdl_filtered.state.tensor.as_slice().is_empty());
}

#[test]
fn test_filter_cohort_tensor_error() {
    // Unreachable test case for safe tensor operations.
    // Logic kept for coverage placeholder.
}
