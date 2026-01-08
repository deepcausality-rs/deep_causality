/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::MaxOrder;
use deep_causality_discovery::{CausalDiscoveryConfig, SurdConfig};

#[test]
fn test_display_surd() {
    let surd_config = SurdConfig::new(MaxOrder::Some(2), 0);
    let config = CausalDiscoveryConfig::Surd(surd_config);
    let display = format!("{}", config);
    assert_eq!(
        display,
        "CausalDiscoveryConfig::Surd(SurdConfig(max_order: Some(2), target_col: 0))"
    );
}

#[test]
fn test_clone() {
    let surd_config = SurdConfig::new(MaxOrder::Some(2), 0);
    let config1 = CausalDiscoveryConfig::Surd(surd_config);
    let config2 = config1.clone();
    match (config1, config2) {
        (CausalDiscoveryConfig::Surd(c1), CausalDiscoveryConfig::Surd(c2)) => {
            assert_eq!(c1.max_order(), MaxOrder::Some(2));
            assert_eq!(c1.target_col(), c2.target_col());
        }
    }
}

#[test]
fn test_debug() {
    let surd_config = SurdConfig::new(MaxOrder::Some(2), 0);
    let config = CausalDiscoveryConfig::Surd(surd_config);
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("Surd(SurdConfig"));
    assert!(debug.contains("max_order: Some(2)"));
    assert!(debug.contains("target_col: 0"));
}
