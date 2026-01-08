/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{FeatureSelectorConfig, MrmrConfig};

#[test]
fn test_display_mrmr() {
    let mrmr_config = MrmrConfig::new(5, 0);
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);
    let display = format!("{}", config);
    assert_eq!(
        display,
        "FeatureSelectorConfig::Mrmr(MrmrConfig(num_features: 5, target_col: 0))"
    );
}

#[test]
fn test_clone() {
    let mrmr_config = MrmrConfig::new(5, 0);
    let config1 = FeatureSelectorConfig::Mrmr(mrmr_config);
    let config2 = config1.clone();
    match (config1, config2) {
        (FeatureSelectorConfig::Mrmr(c1), FeatureSelectorConfig::Mrmr(c2)) => {
            assert_eq!(c1.num_features(), c2.num_features());
            assert_eq!(c1.target_col(), c2.target_col());
        }
    }
}

#[test]
fn test_debug() {
    let mrmr_config = MrmrConfig::new(5, 0);
    let config = FeatureSelectorConfig::Mrmr(mrmr_config);
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("Mrmr(MrmrConfig"));
    assert!(debug.contains("num_features: 5"));
    assert!(debug.contains("target_col: 0"));
}
