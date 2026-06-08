/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::SurdAnalyzeConfig;

#[test]
fn test_new_and_getters() {
    let c = SurdAnalyzeConfig::new(0.1, 0.2, 0.3);
    assert_eq!(c.synergy_threshold(), 0.1);
    assert_eq!(c.unique_threshold(), 0.2);
    assert_eq!(c.redundancy_threshold(), 0.3);
}

#[test]
fn test_default() {
    let c = SurdAnalyzeConfig::default();
    assert_eq!(c.synergy_threshold(), 0.01);
    assert_eq!(c.unique_threshold(), 0.01);
    assert_eq!(c.redundancy_threshold(), 0.01);
}

#[test]
fn test_display() {
    let c = SurdAnalyzeConfig::new(0.1, 0.2, 0.3);
    let s = format!("{}", c);
    assert!(s.contains("SurdAnalyzeConfig"));
    assert!(s.contains("synergy"));
}
