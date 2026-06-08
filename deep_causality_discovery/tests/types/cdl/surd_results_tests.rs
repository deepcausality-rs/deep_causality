/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    CdlBuilder, CdlConfigBuilder, CdlDiscoveryOutcome, MaxOrder, OptionNoneDataCleaner,
    SurdAnalyzeConfig,
};
use std::io::Write;
use tempfile::NamedTempFile;

fn write_csv(content: &str) -> NamedTempFile {
    let mut f = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

#[test]
fn test_surd_analyze_converges_with_path_and_outcome() {
    let file = write_csv(
        "s1,s2,s3,target\n1.0,2.0,3.0,1.5\n2.0,4.1,6.0,3.6\n3.0,6.2,9.0,5.4\n4.0,8.1,12.0,7.6",
    );
    let path = file.path().to_str().unwrap().to_string();
    let cfg = CdlConfigBuilder::build_surd_config::<f64>()
        .with_path(&path)
        .with_target_index(3)
        .with_num_features(3)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
        .build()
        .expect("file exists");

    let effect = CdlBuilder::build_surd(&cfg)
        .surd_load_input()
        .clean_data(OptionNoneDataCleaner)
        .feature_select()
        .surd_discover()
        .surd_analyze();

    assert!(effect.inner.is_ok());
    let cdl = effect.inner.unwrap();
    // dataset_path is carried from the run config (not a generic placeholder).
    assert_eq!(cdl.state.dataset_path, path);
    assert!(cdl.state.feature_selection.is_some());
    assert!(matches!(cdl.state.outcome, CdlDiscoveryOutcome::Surd(_)));
    assert!(!cdl.state.analysis.0.is_empty());
}
