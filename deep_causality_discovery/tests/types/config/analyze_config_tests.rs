/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::AnalyzeConfig;

#[test]
fn test_new() {
    let config = AnalyzeConfig::new(0.1, 0.2, 0.3);
    assert_eq!(config.synergy_threshold(), 0.1);
    assert_eq!(config.unique_threshold(), 0.2);
    assert_eq!(config.redundancy_threshold(), 0.3);
}

#[test]
fn test_getters() {
    let config = AnalyzeConfig::new(0.1, 0.2, 0.3);
    assert_eq!(config.synergy_threshold(), 0.1);
    assert_eq!(config.unique_threshold(), 0.2);
    assert_eq!(config.redundancy_threshold(), 0.3);
}

#[test]
fn test_display() {
    let config = AnalyzeConfig::new(0.1, 0.2, 0.3);
    let display = format!("{}", config);
    assert_eq!(
        display,
        "AnalyzeConfig(synergy: 0.1, unique: 0.2, redundancy: 0.3)"
    );
}

#[test]
fn test_clone() {
    let config1 = AnalyzeConfig::new(0.1, 0.2, 0.3);
    let config2 = config1.clone();
    assert_eq!(config1.synergy_threshold(), config2.synergy_threshold());
    assert_eq!(config1.unique_threshold(), config2.unique_threshold());
    assert_eq!(
        config1.redundancy_threshold(),
        config2.redundancy_threshold()
    );
}

#[test]
fn test_debug() {
    let config = AnalyzeConfig::new(0.1, 0.2, 0.3);
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("AnalyzeConfig"));
    assert!(debug.contains("synergy_threshold: 0.1"));
    assert!(debug.contains("unique_threshold: 0.2"));
    assert!(debug.contains("redundancy_threshold: 0.3"));
}
