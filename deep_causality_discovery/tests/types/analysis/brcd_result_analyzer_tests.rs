/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    BrcdAnalyzeConfig, BrcdResult, BrcdResultAnalyzer, ProcessResultAnalyzer,
};

#[test]
fn test_brcd_analyze_renders_top_k() {
    let result: BrcdResult<f64> =
        BrcdResult::new(vec![vec![1], vec![0], vec![2]], vec![0.8, 0.15, 0.05]);
    let analyzer = BrcdResultAnalyzer;
    let cfg = BrcdAnalyzeConfig::new(2);

    let analysis = analyzer.analyze(&result, &cfg).unwrap();
    let output = analysis.0.join("\n");

    assert!(output.contains("BRCD Root-Cause Analysis"));
    assert!(output.contains("Top 2"));
    assert!(output.contains("{V1}"));
    assert!(output.contains("{V0}"));
    // Only top 2 rendered, not the third.
    assert!(!output.contains("{V2}"));
}

#[test]
fn test_brcd_analyze_empty_result() {
    let result: BrcdResult<f64> = BrcdResult::new(vec![], vec![]);
    let analyzer = BrcdResultAnalyzer;
    let analysis = analyzer
        .analyze(&result, &BrcdAnalyzeConfig::default())
        .unwrap();
    assert!(analysis.0.join("\n").contains("No candidate root causes"));
}
