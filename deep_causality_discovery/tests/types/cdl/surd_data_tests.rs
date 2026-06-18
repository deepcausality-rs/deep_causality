/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    BinningStrategy, CDL, CdlBuilder, CdlConfigBuilder, CdlError, ColumnSelector, DataCleaner,
    DataCleaningError, DataDiscretizer, MaxOrder, OptionNoneDataCleaner, PreprocessConfig,
    SurdAnalyzeConfig, SurdData,
};
use deep_causality_tensor::{CausalTensor, CausalTensorError};
use std::io::Write;
use tempfile::NamedTempFile;

/// A `DataCleaner` that always fails, to drive the `clean_data` error branch.
struct FailingCleaner;
impl DataCleaner<f64> for FailingCleaner {
    fn process(
        &self,
        _tensor: CausalTensor<f64>,
    ) -> Result<CausalTensor<Option<f64>>, DataCleaningError> {
        Err(DataCleaningError::TensorError(
            CausalTensorError::DimensionMismatch,
        ))
    }
}

fn write_csv(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

fn config(path: &str) -> deep_causality_discovery::SurdLoaderConfig<f64> {
    CdlConfigBuilder::build_surd_config::<f64>()
        .with_path(path)
        .with_target_index(3)
        .with_num_features(3)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
        .build()
        .expect("file exists")
}

const DATA: &str =
    "s1,s2,s3,target\n1.0,2.0,3.0,1.5\n2.0,4.1,6.0,3.6\n3.0,6.2,9.0,5.4\n4.0,8.1,12.0,7.6";

#[test]
fn test_clean_data() {
    let file = write_csv(DATA);
    let cfg = config(file.path().to_str().unwrap());
    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .clean_data(OptionNoneDataCleaner);
    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    assert_eq!(cdl.state.records_count, 4);
    assert_eq!(cdl.state.tensor.shape(), &[4, 4]);
}

#[test]
fn test_filter_cohort_keeps_matching_rows() {
    let file = write_csv(DATA);
    let cfg = config(file.path().to_str().unwrap());
    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .filter_cohort(|row| row[0] >= 2.0); // keeps rows with s1 >= 2.0 (3 of 4)
    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    assert_eq!(cdl.state.records_count, 3);
}

#[test]
fn test_preprocess_discretizes() {
    let file = write_csv(DATA);
    let cfg = config(file.path().to_str().unwrap());
    let pp = PreprocessConfig::new(BinningStrategy::EqualWidth, 2, ColumnSelector::All);
    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .preprocess(DataDiscretizer, &pp);
    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    assert_eq!(cdl.state.tensor.shape(), &[4, 4]);
}

#[test]
fn test_preprocess_invalid_column_errors() {
    // A column index that does not exist surfaces as a CdlError::PreprocessError
    // (covers the preprocess error branch).
    let file = write_csv(DATA);
    let cfg = config(file.path().to_str().unwrap());
    let pp = PreprocessConfig::new(
        BinningStrategy::EqualWidth,
        2,
        ColumnSelector::ByIndex(vec![99]),
    );
    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .preprocess(DataDiscretizer, &pp);
    assert!(matches!(effect.inner, Err(CdlError::PreprocessError(_))));
}

#[test]
fn test_clean_data_surfaces_a_cleaner_error() {
    let file = write_csv(DATA);
    let cfg = config(file.path().to_str().unwrap());
    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .clean_data(FailingCleaner);
    assert!(
        effect.inner.is_err(),
        "a failing cleaner must surface its error"
    );
}

#[test]
fn test_filter_cohort_rejects_a_non_2d_tensor() {
    // `filter_cohort` is only defined for a 2-D matrix; a rank-1 tensor must error rather than
    // panic on the `shape[1]` index.
    let file = write_csv(DATA);
    let cfg = config(file.path().to_str().unwrap());
    let tensor = CausalTensor::new(vec![1.0_f64, 2.0, 3.0], vec![3]).unwrap(); // rank 1
    let cdl = CDL {
        state: SurdData {
            tensor,
            records_count: 3,
            config: cfg,
        },
    };
    let effect = cdl.filter_cohort(|_row| true);
    assert!(matches!(
        effect.inner,
        Err(CdlError::CausalDiscoveryError(_))
    ));
}

#[test]
fn test_filter_cohort_keeping_no_rows_yields_an_empty_dataset() {
    let file = write_csv(DATA);
    let cfg = config(file.path().to_str().unwrap());
    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .filter_cohort(|_row| false); // keep nothing
    assert!(effect.inner.is_ok());
    assert_eq!(effect.inner.unwrap().state.records_count, 0);
}
