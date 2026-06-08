/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    BinningStrategy, CdlBuilder, CdlConfigBuilder, CdlError, ColumnSelector, DataDiscretizer,
    MaxOrder, OptionNoneDataCleaner, PreprocessConfig, SurdAnalyzeConfig,
};
use std::io::Write;
use tempfile::NamedTempFile;

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
