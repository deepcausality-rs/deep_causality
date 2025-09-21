/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    CausalDiscovery, CdlConfig, CdlError, DataPreprocessor, FeatureSelector, ProcessAnalysis,
    ProcessDataLoader, ProcessFormattedResult, ProcessResultAnalyzer, ProcessResultFormatter,
};
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

// Typestate structs representing the pipeline's state.
/// Initial state of the CDL pipeline, with no data loaded.
pub struct NoData;
/// State after data has been successfully loaded.
pub struct WithData(CausalTensor<f64>);
/// State after feature selection has been applied.
pub struct WithFeatures(CausalTensor<f64>);
/// State after a causal discovery algorithm has been run.
pub struct WithCausalResults(SurdResult<f64>);
/// State after the raw causal results have been analyzed.
pub struct WithAnalysis(ProcessAnalysis);
/// Final state after the analysis has been formatted into a final result.
pub struct Finalized(ProcessFormattedResult);

/// The core builder for the Causal Discovery Language (CDL) pipeline.
///
/// `CDL` uses a typestate pattern to ensure that pipeline steps are called in a valid
/// order at compile time. Each method consumes the `CDL` instance and returns a new
/// one with an updated state.
pub struct CDL<State> {
    state: State,
    config: CdlConfig,
}

// Initial state
impl Default for CDL<NoData> {
    fn default() -> Self {
        Self::new()
    }
}

impl CDL<NoData> {
    /// Creates a new CDL pipeline builder in its initial state with a default configuration.
    pub fn new() -> Self {
        CDL {
            state: NoData,
            config: CdlConfig::new(),
        }
    }

    /// Creates a new CDL pipeline builder in its initial state with a specific configuration.
    pub fn with_config(config: CdlConfig) -> Self {
        CDL {
            state: NoData,
            config,
        }
    }

    /// Starts the pipeline by loading data from the given path.
    ///
    /// # Arguments
    /// * `loader` - An implementation of `ProcessDataLoader` (e.g., `CsvDataLoader`).
    /// * `path` - The path to the data source file.
    ///
    /// # Returns
    /// A `CDL` instance in the `WithData` state, or a `CdlError` if loading fails.
    pub fn start<L>(self, loader: L, path: &str) -> Result<CDL<WithData>, CdlError>
    where
        L: ProcessDataLoader,
    {
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
    /// An optional step to preprocess the loaded data.
    ///
    /// This method is a self-transition, returning the `CDL` in the same `WithData`
    /// state, allowing it to be chained or skipped.
    ///
    /// # Arguments
    /// * `preprocessor` - An implementation of `DataPreprocessor` (e.g., `DataDiscretizer`).
    ///
    /// # Returns
    /// A `CDL` instance in the `WithData` state, or a `CdlError` if preprocessing fails.
    pub fn preprocess<P>(self, preprocessor: P) -> Result<CDL<WithData>, CdlError>
    where
        P: DataPreprocessor,
    {
        if let Some(config) = self.config.preprocess_config() {
            let processed_tensor = preprocessor.process(self.state.0, config)?;
            Ok(CDL {
                state: WithData(processed_tensor),
                config: self.config,
            })
        } else {
            Ok(self) // If no config is present, pass through without changes.
        }
    }

    /// An optional step to select a subset of features from the data.
    ///
    /// # Arguments
    /// * `selector` - An implementation of `FeatureSelector` (e.g., `MrmrFeatureSelector`).
    ///
    /// # Returns
    /// A `CDL` instance in the `WithFeatures` state, or a `CdlError` if selection fails.
    pub fn feat_select<S>(self, selector: S) -> Result<CDL<WithFeatures>, CdlError>
    where
        S: FeatureSelector,
    {
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
    /// Runs a causal discovery algorithm on the feature-selected data.
    ///
    /// # Arguments
    /// * `discovery` - An implementation of `CausalDiscovery` (e.g., `SurdCausalDiscovery`).
    ///
    /// # Returns
    /// A `CDL` instance in the `WithCausalResults` state, or a `CdlError` if discovery fails.
    pub fn causal_discovery<D>(self, discovery: D) -> Result<CDL<WithCausalResults>, CdlError>
    where
        D: CausalDiscovery,
    {
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
    /// Analyzes the raw results from the discovery algorithm.
    ///
    /// # Arguments
    /// * `analyzer` - An implementation of `ProcessResultAnalyzer`.
    ///
    /// # Returns
    /// A `CDL` instance in the `WithAnalysis` state, or a `CdlError` if analysis fails.
    pub fn analyze<A>(self, analyzer: A) -> Result<CDL<WithAnalysis>, CdlError>
    where
        A: ProcessResultAnalyzer,
    {
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
    /// Formats the analysis into a final, presentable result.
    ///
    /// # Arguments
    /// * `formatter` - An implementation of `ProcessResultFormatter`.
    ///
    /// # Returns
    /// A `CDL` instance in the `Finalized` state, or a `CdlError` if formatting fails.
    pub fn finalize<F>(self, formatter: F) -> Result<CDL<Finalized>, CdlError>
    where
        F: ProcessResultFormatter,
    {
        let formatted_result = formatter.format(&self.state.0)?;
        Ok(CDL {
            state: Finalized(formatted_result),
            config: self.config,
        })
    }
}

// After process is finalized
impl CDL<Finalized> {
    /// Builds the final, executable runner for the pipeline.
    pub fn build(self) -> Result<CQDRunner, CdlError> {
        let CDL { state, config } = self; // Destructure self
        Ok(CQDRunner {
            result: state.0,
            config, // Use the destructured config
        })
    }
}

/// The final, executable runner for a configured CDL pipeline.
pub struct CQDRunner {
    result: ProcessFormattedResult,
    config: CdlConfig,
}

impl CQDRunner {
    /// Runs the pipeline and returns the final formatted result.
    ///
    /// In a more complex application, this could trigger logging, saving to a database, etc.
    pub fn run(self) -> Result<ProcessFormattedResult, CdlError> {
        // Use the config for logging/debugging
        println!("Running CDL pipeline with config: {}", self.config);
        // Return the formatted result
        Ok(self.result)
    }
}
