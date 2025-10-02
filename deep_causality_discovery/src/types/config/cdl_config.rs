/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AnalyzeConfig, CausalDiscoveryConfig, DataLoaderConfig, FeatureSelectorConfig, PreprocessConfig,
};
use std::fmt;

/// The master configuration struct for a CDL pipeline.
///
/// It holds optional configuration structs for each stage of the pipeline.
/// The presence of a configuration determines whether a particular stage
/// (like preprocessing or feature selection) is executed.
#[derive(Debug, Clone, Default)]
pub struct CdlConfig {
    data_loader_config: Option<DataLoaderConfig>,
    preprocess_config: Option<PreprocessConfig>,
    feature_selector_config: Option<FeatureSelectorConfig>,
    causal_discovery_config: Option<CausalDiscoveryConfig>,
    analyze_config: Option<AnalyzeConfig>,
}

impl CdlConfig {
    pub fn new() -> Self {
        Self {
            data_loader_config: None,
            preprocess_config: None,
            feature_selector_config: None,
            causal_discovery_config: None,
            analyze_config: None,
        }
    }
}

// Builders
impl CdlConfig {
    pub fn with_data_loader(mut self, config: DataLoaderConfig) -> Self {
        self.data_loader_config = Some(config);
        self
    }

    pub fn with_preprocess_config(mut self, config: PreprocessConfig) -> Self {
        self.preprocess_config = Some(config);
        self
    }

    pub fn with_feature_selector(mut self, config: FeatureSelectorConfig) -> Self {
        self.feature_selector_config = Some(config);
        self
    }

    pub fn with_causal_discovery(mut self, config: CausalDiscoveryConfig) -> Self {
        self.causal_discovery_config = Some(config);
        self
    }

    pub fn with_analysis(mut self, config: AnalyzeConfig) -> Self {
        self.analyze_config = Some(config);
        self
    }
}

// Getters
impl CdlConfig {
    pub fn data_loader_config(&self) -> &Option<DataLoaderConfig> {
        &self.data_loader_config
    }

    pub fn preprocess_config(&self) -> &Option<PreprocessConfig> {
        &self.preprocess_config
    }

    pub fn feature_selector_config(&self) -> &Option<FeatureSelectorConfig> {
        &self.feature_selector_config
    }

    pub fn causal_discovery_config(&self) -> &Option<CausalDiscoveryConfig> {
        &self.causal_discovery_config
    }

    pub fn analyze_config(&self) -> &Option<AnalyzeConfig> {
        &self.analyze_config
    }
}

impl fmt::Display for CdlConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "CdlConfig {{")?;
        writeln!(
            f,
            "    data_loader_config: {}",
            self.data_loader_config
                .as_ref()
                .map_or("None".to_string(), |c| c.to_string())
        )?;
        writeln!(
            f,
            "    preprocess_config: {}",
            self.preprocess_config
                .as_ref()
                .map_or("None".to_string(), |c| c.to_string())
        )?;
        writeln!(
            f,
            "    feature_selector_config: {}",
            self.feature_selector_config
                .as_ref()
                .map_or("None".to_string(), |c| c.to_string())
        )?;
        writeln!(
            f,
            "    causal_discovery_config: {}",
            self.causal_discovery_config
                .as_ref()
                .map_or("None".to_string(), |c| c.to_string())
        )?;
        writeln!(
            f,
            "    analyze_config: {}",
            self.analyze_config
                .as_ref()
                .map_or("None".to_string(), |c| c.to_string())
        )?;
        write!(f, "}}")
    }
}
