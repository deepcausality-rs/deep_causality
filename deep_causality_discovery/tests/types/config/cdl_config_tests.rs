/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_discovery::{
    AnalyzeConfig, CausalDiscoveryConfig, CdlConfig, CsvConfig, DataLoaderConfig,
    FeatureSelectorConfig, MrmrConfig, PreprocessConfig, SurdConfig,
};

#[test]
fn test_cdl_config_new_default() {
    let config = CdlConfig::new();
    assert!(config.data_loader_config().is_none());
    assert!(config.preprocess_config().is_none());
    assert!(config.feature_selector_config().is_none());
    assert!(config.causal_discovery_config().is_none());
    assert!(config.analyze_config().is_none());
}

#[test]
fn test_cdl_config_builder() {
    let config = CdlConfig::new()
        .with_data_loader(DataLoaderConfig::Csv(CsvConfig::default()))
        .with_preprocess_config(PreprocessConfig::new(
            deep_causality_discovery::BinningStrategy::EqualWidth,
            10,
            deep_causality_discovery::ColumnSelector::All,
        ))
        .with_feature_selector(FeatureSelectorConfig::Mrmr(MrmrConfig::new(1, 0)))
        .with_causal_discovery(CausalDiscoveryConfig::Surd(SurdConfig::new(
            deep_causality_algorithms::surd::MaxOrder::Max,
            0,
        )))
        .with_analysis(AnalyzeConfig::new(0.1, 0.1, 0.1));

    assert!(config.data_loader_config().is_some());
    assert!(config.preprocess_config().is_some());
    assert!(config.feature_selector_config().is_some());
    assert!(config.causal_discovery_config().is_some());
    assert!(config.analyze_config().is_some());
}

#[test]
fn test_cdl_config_display() {
    let config = CdlConfig::new().with_data_loader(DataLoaderConfig::Csv(CsvConfig::default()));
    let s = format!("{}", config);
    assert!(s.contains("CdlConfig"));
    assert!(s.contains("data_loader_config"));
}
