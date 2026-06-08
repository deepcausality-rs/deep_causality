/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    CdlBuilder, CdlConfigBuilder, MaxOrder, OptionNoneDataCleaner, SurdAnalyzeConfig,
};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_csv(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn test_feature_select_filters_to_num_features() {
    let file = write_csv(
        "s1,s2,s3,target\n1.0,2.0,3.0,1.5\n2.0,4.1,6.0,3.6\n3.0,6.2,9.0,5.4\n4.0,8.1,12.0,7.6",
    );
    let cfg = CdlConfigBuilder::build_surd_config::<f64>()
        .with_path(file.path().to_str().unwrap())
        .with_target_index(3)
        .with_num_features(3)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
        .build()
        .expect("file exists");

    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .clean_data(OptionNoneDataCleaner)
        .feature_select();

    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    // MRMR keeps `num_features` columns and records the selection.
    assert_eq!(cdl.state.tensor.shape()[1], 3);
    assert_eq!(cdl.state.selection_result.len(), 3);
    assert_eq!(cdl.state.records_count, 4);
}
