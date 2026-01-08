/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_discovery::CdlReport;
use std::collections::HashMap;

#[test]
fn test_cdl_report_display() {
    // Mock data
    let mrmr_res = MrmrResult::new(vec![(0, 0.9), (2, 0.8)]);

    // Construct fake SurdResult
    // SurdResult fields are usually private, check if I can construct it or mock it.
    // SurdResult { redundant: ..., unique: ..., synergistic: ..., mutual_info: ..., info_leak: ..., state_maps: ... }
    // Ideally SurdResult has a constructor or is public fields.
    // Assuming we can default construct or use a helper.
    // SurdResult might need to be sourced from deep_causality_algorithms if possible.
    // However, for this test, if fields are public I can fill them.
    // Let's assume standard formatting check is enough, or use Default if available.

    // Wait, testing Display output usually requires specific mock data.
    // If SurdResult is hard to construct manually due to complexity,
    // maybe we skip complex SurdResult validation or use a minimal valid one.

    // Let's try to construct a dummy SurdResult using default types if possible.
    // Use HashMaps for fields.
    // Construct SurdResult using new()
    let surd_res = SurdResult::new(
        HashMap::from([(vec![0], 0.5)]), // redundant (mocked)
        HashMap::from([(vec![1], 0.3)]), // unique (mocked)
        HashMap::default(),              // synergistic
        0.1,                             // mutual_info (f64)
        HashMap::default(),              // info_leak_or_state_maps (HashMap)
        Default::default(),              // state_maps
        Default::default(),              // state_maps
        Default::default(),
        Default::default(),
        Default::default(),
    );

    let report = CdlReport {
        dataset_path: "/tmp/test.csv".into(),
        records_processed: 100,
        feature_selection: mrmr_res,
        causal_analysis: surd_res,
    };

    let s = format!("{}", report);
    assert!(s.contains("[1] DATASET SUMMARY"));
    assert!(s.contains("File: .............. /tmp/test.csv"));
    assert!(s.contains("Records: ........... 100"));
    assert!(s.contains("[2] FEATURE SELECTION (MRMR)"));
    assert!(s.contains("0          | 0.9000"));
    assert!(s.contains("[3] CAUSAL DISCOVERY (SURD)"));
    // println!("{}", s); // Debug
}
