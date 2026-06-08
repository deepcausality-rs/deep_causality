/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! End-to-end SURD regression: the re-designed pipeline must still run the full
//! SURD chain and produce a report with the feature-selection and SURD sections.

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
fn test_surd_full_pipeline_produces_report() {
    let file = write_csv(
        "s1,s2,s3,target\n1.0,2.0,3.0,1.5\n2.0,4.1,6.0,3.6\n3.0,6.2,9.0,5.4\n4.0,8.1,12.0,7.6",
    );
    let config = CdlConfigBuilder::build_surd_config::<f64>()
        .with_path(file.path().to_str().unwrap())
        .with_target_index(3)
        .with_num_features(3)
        .with_max_order(MaxOrder::Max)
        .with_analyze(SurdAnalyzeConfig::new(0.01, 0.01, 0.01))
        .build()
        .expect("file exists");

    let report_effect = CdlBuilder::build_surd(&config)
        .surd_load_input()
        .clean_data(OptionNoneDataCleaner)
        .feature_select()
        .surd_discover()
        .surd_analyze()
        .finalize();

    assert!(report_effect.inner.is_ok());
    let report = report_effect.inner.unwrap();
    assert_eq!(report.records_processed, 4);
    // SURD lineage carries the MRMR feature-selection result.
    assert!(report.feature_selection.is_some());
    // The discovery outcome is the SURD variant.
    assert!(matches!(
        report.causal_analysis,
        CdlDiscoveryOutcome::Surd(_)
    ));

    let rendered = format!("{}", report);
    assert!(rendered.contains("FEATURE SELECTION (MRMR)"));
    assert!(rendered.contains("CAUSAL DISCOVERY (SURD)"));
}
