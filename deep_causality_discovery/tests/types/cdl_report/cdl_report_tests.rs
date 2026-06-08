/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::BrcdResult;
use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_discovery::{CdlDiscoveryOutcome, CdlReport};
use std::collections::HashMap;

#[test]
fn test_surd_report_display() {
    let mrmr_res = MrmrResult::new(vec![(0, 0.9), (2, 0.8)]);
    let surd_res = SurdResult::new(
        HashMap::from([(vec![0], 0.5)]),
        HashMap::from([(vec![1], 0.3)]),
        HashMap::default(),
        0.1,
        HashMap::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
        Default::default(),
    );

    let report = CdlReport {
        dataset_path: "/tmp/test.csv".into(),
        records_processed: 100,
        feature_selection: Some(mrmr_res),
        causal_analysis: CdlDiscoveryOutcome::Surd(Box::new(surd_res)),
    };

    let s = format!("{}", report);
    assert!(s.contains("[1] DATASET SUMMARY"));
    assert!(s.contains("File: .............. /tmp/test.csv"));
    assert!(s.contains("Records: ........... 100"));
    assert!(s.contains("[2] FEATURE SELECTION (MRMR)"));
    assert!(s.contains("[3] CAUSAL DISCOVERY (SURD)"));
}

#[test]
fn test_brcd_report_display_has_no_feature_selection() {
    let brcd_res: BrcdResult<f64> = BrcdResult::new(vec![vec![1], vec![0]], vec![0.9, 0.1]);
    let report = CdlReport {
        dataset_path: "BRCD (normal + anomalous datasets)".into(),
        records_processed: 240,
        feature_selection: None,
        causal_analysis: CdlDiscoveryOutcome::Brcd(brcd_res),
    };

    let s = format!("{}", report);
    assert!(s.contains("Records: ........... 240"));
    assert!(s.contains("[3] ROOT-CAUSE DISCOVERY (BRCD)"));
    // BRCD performs no feature selection, so that section is absent.
    assert!(!s.contains("FEATURE SELECTION"));
}
