/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::MrmrConfig;

#[test]
fn test_new_and_getters() {
    let config = MrmrConfig::new(10, 1);
    assert_eq!(config.num_features(), 10);
    assert_eq!(config.target_col(), 1);
}

#[test]
fn test_display() {
    let config = MrmrConfig::new(10, 1);
    let display = format!("{}", config);
    assert_eq!(display, "MrmrConfig(num_features: 10, target_col: 1)");
}

#[test]
fn test_clone() {
    let config1 = MrmrConfig::new(10, 1);
    let config2 = config1.clone();
    assert_eq!(config1.num_features(), config2.num_features());
    assert_eq!(config1.target_col(), config2.target_col());
}

#[test]
fn test_debug() {
    let config = MrmrConfig::new(10, 1);
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("MrmrConfig"));
    assert!(debug.contains("num_features: 10"));
    assert!(debug.contains("target_col: 1"));
}
