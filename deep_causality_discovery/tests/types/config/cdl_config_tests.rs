/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::surd::MaxOrder;
use deep_causality_discovery::{
    AnalyzeConfig, BinningStrategy, CausalDiscoveryConfig, CdlConfig, ColumnSelector, CsvConfig,
    DataLoaderConfig, FeatureSelectorConfig, MrmrConfig, ParquetConfig, PreprocessConfig,
    SurdConfig,
};

#[test]
fn test_new_and_default() {
    let config_new = CdlConfig::new();
    assert!(config_new.data_loader_config().is_none());
    assert!(config_new.preprocess_config().is_none());
    assert!(config_new.feature_selector_config().is_none());
    assert!(config_new.causal_discovery_config().is_none());
    assert!(config_new.analyze_config().is_none());

    let config_default: CdlConfig = Default::default();
    assert!(config_default.data_loader_config().is_none());
    assert!(config_default.preprocess_config().is_none());
    assert!(config_default.feature_selector_config().is_none());
    assert!(config_default.causal_discovery_config().is_none());
    assert!(config_default.analyze_config().is_none());
}

#[test]
fn test_builders_and_getters() {
    let data_loader_config = DataLoaderConfig::Csv(CsvConfig::new(true, b',', 0, None));
    let preprocess_config =
        PreprocessConfig::new(BinningStrategy::EqualWidth, 10, ColumnSelector::All);
    let feature_selector_config = FeatureSelectorConfig::Mrmr(MrmrConfig::new(5, 0));
    let causal_discovery_config =
        CausalDiscoveryConfig::Surd(SurdConfig::new(MaxOrder::Some(2), 0));
    let analyze_config = AnalyzeConfig::new(0.1, 0.2, 0.3);

    let config = CdlConfig::new()
        .with_data_loader(data_loader_config.clone())
        .with_preprocess_config(preprocess_config.clone())
        .with_feature_selector(feature_selector_config.clone())
        .with_causal_discovery(causal_discovery_config.clone())
        .with_analysis(analyze_config.clone());

    assert!(config.data_loader_config().is_some());
    assert!(config.preprocess_config().is_some());
    assert!(config.feature_selector_config().is_some());
    assert!(config.causal_discovery_config().is_some());
    assert!(config.analyze_config().is_some());

    // Test getters
    if let Some(DataLoaderConfig::Csv(c)) = config.data_loader_config() {
        assert!(c.has_headers());
    } else {
        panic!("Wrong data loader config");
    }

    if let Some(p_config) = config.preprocess_config() {
        assert_eq!(p_config.num_bins(), 10);
    } else {
        panic!("Wrong preprocess config");
    }

    if let Some(FeatureSelectorConfig::Mrmr(m_config)) = config.feature_selector_config() {
        assert_eq!(m_config.num_features(), 5);
    } else {
        panic!("Wrong feature selector config");
    }

    if let Some(CausalDiscoveryConfig::Surd(s_config)) = config.causal_discovery_config() {
        assert_eq!(s_config.max_order(), MaxOrder::Some(2));
    } else {
        panic!("Wrong causal discovery config");
    }

    if let Some(a_config) = config.analyze_config() {
        assert_eq!(a_config.synergy_threshold(), 0.1);
    } else {
        panic!("Wrong analyze config");
    }
}

#[test]
fn test_display_empty() {
    let config = CdlConfig::new();
    let display = format!("{}", config);
    assert!(display.contains("data_loader_config: None"));
    assert!(display.contains("preprocess_config: None"));
    assert!(display.contains("feature_selector_config: None"));
    assert!(display.contains("causal_discovery_config: None"));
    assert!(display.contains("analyze_config: None"));
}

#[test]
fn test_display_full() {
    let data_loader_config = DataLoaderConfig::Parquet(ParquetConfig::default());
    let preprocess_config = PreprocessConfig::new(
        BinningStrategy::EqualFrequency,
        5,
        ColumnSelector::ByIndex(vec![1, 2]),
    );
    let feature_selector_config = FeatureSelectorConfig::Mrmr(MrmrConfig::new(3, 1));
    let causal_discovery_config =
        CausalDiscoveryConfig::Surd(SurdConfig::new(MaxOrder::Some(3), 1));
    let analyze_config = AnalyzeConfig::new(0.4, 0.5, 0.6);

    let config = CdlConfig::new()
        .with_data_loader(data_loader_config)
        .with_preprocess_config(preprocess_config)
        .with_feature_selector(feature_selector_config)
        .with_causal_discovery(causal_discovery_config)
        .with_analysis(analyze_config);

    let display = format!("{}", config);
    assert!(display.contains("DataLoaderConfig::Parquet"));
    assert!(display.contains("PreprocessConfig"));
    assert!(display.contains("FeatureSelectorConfig::Mrmr"));
    assert!(display.contains("CausalDiscoveryConfig::Surd"));
    assert!(display.contains("AnalyzeConfig"));
}

#[test]
fn test_clone() {
    let config1 = CdlConfig::new().with_analysis(AnalyzeConfig::new(0.1, 0.2, 0.3));
    let config2 = config1.clone();
    assert!(config2.analyze_config().is_some());
    assert_eq!(
        config1
            .analyze_config()
            .as_ref()
            .unwrap()
            .unique_threshold(),
        config2
            .analyze_config()
            .as_ref()
            .unwrap()
            .unique_threshold()
    );
}

#[test]
fn test_debug() {
    let config = CdlConfig::new();
    let debug = format!("{:?}", config);
    assert!(debug.starts_with("CdlConfig"));
    assert!(debug.contains("data_loader_config: None"));
}

#[test]
fn test_display_only_data_loader_config() {
    let data_loader_config = DataLoaderConfig::Csv(CsvConfig::new(true, b',', 0, None));
    let config = CdlConfig::new().with_data_loader(data_loader_config);
    let display = format!("{}", config);
    assert!(display.contains("data_loader_config: DataLoaderConfig::Csv(CsvConfig(headers: true, delimiter: ,, skip: 0, columns: None))"));
    assert!(display.contains("preprocess_config: None"));
    assert!(display.contains("feature_selector_config: None"));
    assert!(display.contains("causal_discovery_config: None"));
    assert!(display.contains("analyze_config: None"));
}

#[test]
fn test_display_only_preprocess_config() {
    let preprocess_config =
        PreprocessConfig::new(BinningStrategy::EqualWidth, 10, ColumnSelector::All);
    let config = CdlConfig::new().with_preprocess_config(preprocess_config);
    let display = format!("{}", config);
    assert!(display.contains("data_loader_config: None"));
    assert!(display.contains(
        "preprocess_config: PreprocessConfig(strategy: EqualWidth, num_bins: 10, columns: All)"
    ));
    assert!(display.contains("feature_selector_config: None"));
    assert!(display.contains("causal_discovery_config: None"));
    assert!(display.contains("analyze_config: None"));
}

#[test]
fn test_display_only_feature_selector_config() {
    let feature_selector_config = FeatureSelectorConfig::Mrmr(MrmrConfig::new(5, 0));
    let config = CdlConfig::new().with_feature_selector(feature_selector_config);
    let display = format!("{}", config);
    assert!(display.contains("data_loader_config: None"));
    assert!(display.contains("preprocess_config: None"));
    assert!(display.contains("feature_selector_config: FeatureSelectorConfig::Mrmr(MrmrConfig(num_features: 5, target_col: 0))"));
    assert!(display.contains("causal_discovery_config: None"));
    assert!(display.contains("analyze_config: None"));
}

#[test]
fn test_display_only_causal_discovery_config() {
    let causal_discovery_config =
        CausalDiscoveryConfig::Surd(SurdConfig::new(MaxOrder::Some(2), 0));
    let config = CdlConfig::new().with_causal_discovery(causal_discovery_config);
    let display = format!("{}", config);
    assert!(display.contains("data_loader_config: None"));
    assert!(display.contains("preprocess_config: None"));
    assert!(display.contains("feature_selector_config: None"));
    assert!(display.contains("causal_discovery_config: CausalDiscoveryConfig::Surd(SurdConfig(max_order: Some(2), target_col: 0))"));
    assert!(display.contains("analyze_config: None"));
}

#[test]
fn test_display_only_analyze_config() {
    let analyze_config = AnalyzeConfig::new(0.1, 0.2, 0.3);
    let config = CdlConfig::new().with_analysis(analyze_config);
    let display = format!("{}", config);
    assert!(display.contains("data_loader_config: None"));
    assert!(display.contains("preprocess_config: None"));
    assert!(display.contains("feature_selector_config: None"));
    assert!(display.contains("causal_discovery_config: None"));
    assert!(
        display
            .contains("analyze_config: AnalyzeConfig(synergy: 0.1, unique: 0.2, redundancy: 0.3)")
    );
}
