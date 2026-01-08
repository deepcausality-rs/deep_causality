/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::MaxOrder;
use deep_causality_discovery::SurdConfig;

#[test]
fn test_new_and_getters() {
    let config = SurdConfig::new(MaxOrder::Some(3), 2);
    assert_eq!(config.max_order(), MaxOrder::Some(3));
    assert_eq!(config.target_col(), 2);
}

#[test]
fn test_display() {
    let config = SurdConfig::new(MaxOrder::Some(3), 2);
    let display = format!("{}", config);
    assert_eq!(display, "SurdConfig(max_order: Some(3), target_col: 2)");
}

#[test]
fn test_clone() {
    let config1 = SurdConfig::new(MaxOrder::Some(3), 2);
    let config2 = config1.clone();
    assert_eq!(config1.max_order(), config2.max_order());
    assert_eq!(config1.target_col(), config2.target_col());
}

#[test]
fn test_debug() {
    let config = SurdConfig::new(MaxOrder::Some(3), 2);
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("SurdConfig"));
    assert!(debug.contains("max_order: Some(3)"));
    assert!(debug.contains("target_col: 2"));
}
