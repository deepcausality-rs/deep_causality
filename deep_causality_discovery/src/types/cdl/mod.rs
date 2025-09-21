/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::CdlError;
use crate::traits::causal_discovery::CausalDiscovery;
use crate::traits::feature_selector::FeatureSelector;
use crate::traits::process_data_loader::ProcessDataLoader;
use crate::traits::process_result::{
    ProcessAnalysis, ProcessFormattedResult, ProcessResultAnalyzer, ProcessResultFormatter,
};
use crate::types::config::CdlConfig;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

// Typestate structs
pub struct NoData;
pub struct WithData(CausalTensor<f64>);
pub struct WithFeatures(CausalTensor<f64>);
pub struct WithCausalResults(SurdResult<f64>);
pub struct WithAnalysis(ProcessAnalysis);
pub struct Finalized(ProcessFormattedResult);

pub struct CDL<State> {
    state: State,
    config: CdlConfig, // Added this field
}

// Initial state
impl Default for CDL<NoData> {
    fn default() -> Self {
        Self::new()
    }
}

impl CDL<NoData> {
    pub fn new() -> Self {
        CDL {
            state: NoData,
            config: CdlConfig::new(),
        }
    }

    pub fn with_config(config: CdlConfig) -> Self {
        CDL {
            state: NoData,
            config,
        }
    }

    pub fn start<L: ProcessDataLoader>(
        self,
        loader: L,
        path: &str,
    ) -> Result<CDL<WithData>, CdlError> {
        let loader_config = self
            .config
            .data_loader_config()
            .as_ref()
            .ok_or(CdlError::MissingDataLoaderConfig)?;

        let tensor = loader.load(path, loader_config)?;
        Ok(CDL {
            state: WithData(tensor),
            config: self.config,
        })
    }
}

// After data is loaded
impl CDL<WithData> {
    pub fn feat_select<S: FeatureSelector>(
        self,
        selector: S,
    ) -> Result<CDL<WithFeatures>, CdlError> {
        let feature_config = self
            .config
            .feature_selector_config()
            .as_ref()
            .ok_or(CdlError::MissingFeatureSelectorConfig)?;

        let selected_tensor = selector.select(self.state.0, feature_config)?;
        Ok(CDL {
            state: WithFeatures(selected_tensor),
            config: self.config,
        })
    }
}

// After features are selected
impl CDL<WithFeatures> {
    pub fn causal_discovery<D: CausalDiscovery>(
        self,
        discovery: D,
    ) -> Result<CDL<WithCausalResults>, CdlError> {
        let discovery_config = self
            .config
            .causal_discovery_config()
            .as_ref()
            .ok_or(CdlError::MissingCausalDiscoveryConfig)?;

        let results = discovery.discover(self.state.0, discovery_config)?;
        Ok(CDL {
            state: WithCausalResults(results),
            config: self.config,
        })
    }
}

// After causal discovery is performed
impl CDL<WithCausalResults> {
    pub fn analyze<A: ProcessResultAnalyzer>(
        self,
        analyzer: A,
    ) -> Result<CDL<WithAnalysis>, CdlError> {
        let analyze_config = self
            .config
            .analyze_config()
            .as_ref()
            .ok_or(CdlError::MissingAnalyzeConfig)?;

        let analysis = analyzer.analyze(&self.state.0, analyze_config)?;
        Ok(CDL {
            state: WithAnalysis(analysis),
            config: self.config,
        })
    }
}

// After results are analyzed
impl CDL<WithAnalysis> {
    pub fn finalize<F: ProcessResultFormatter>(
        self,
        formatter: F,
    ) -> Result<CDL<Finalized>, CdlError> {
        let formatted_result = formatter.format(&self.state.0)?;
        Ok(CDL {
            state: Finalized(formatted_result),
            config: self.config,
        })
    }
}

// After process is finalized
impl CDL<Finalized> {
    pub fn build(self) -> Result<CQDRunner, CdlError> {
        let CDL { state, config } = self; // Destructure self
        Ok(CQDRunner {
            result: state.0,
            config, // Use the destructured config
        })
    }
}

// Runner for the built pipeline
pub struct CQDRunner {
    result: ProcessFormattedResult,
    config: CdlConfig,
}

impl CQDRunner {
    pub fn run(self) -> Result<ProcessFormattedResult, CdlError> {
        // Use the config for logging/debugging
        println!("Running CDL pipeline with config: {}", self.config);
        // Return the formatted result
        Ok(self.result)
    }
}
