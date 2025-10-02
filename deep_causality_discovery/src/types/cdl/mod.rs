/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CdlConfig, CdlError, ProcessAnalysis, ProcessFormattedResult};
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

mod cdl_with_analysis;
mod cdl_with_causal_results;
mod cdl_with_data;
mod cdl_with_features;
mod cdl_with_no_data;

// Typestate structs representing the pipeline's state.
/// Initial state of the CDL pipeline, with no data loaded.
#[derive(Debug)]
pub struct NoData;
/// State after data has been successfully loaded.
#[derive(Debug)]
pub struct WithData(CausalTensor<f64>);
/// State after feature selection has been applied.
#[derive(Debug)]
pub struct WithFeatures(CausalTensor<Option<f64>>);
/// State after a causal discovery algorithm has been run.
#[derive(Debug)]
pub struct WithCausalResults(SurdResult<f64>);
/// State after the raw causal results have been analyzed.
#[derive(Debug)]
pub struct WithAnalysis(ProcessAnalysis);
/// Final state after the analysis has been formatted into a final result.
#[derive(Debug)]
pub struct Finalized(ProcessFormattedResult);
/// The final, executable runner for a configured CDL pipeline.
#[derive(Debug)]
pub struct CDLRunner {
    result: ProcessFormattedResult,
    config: CdlConfig,
}

/// The core builder for the Causal Discovery Language (CDL) pipeline.
///
/// `CDL` uses a typestate pattern to ensure that pipeline steps are called in a valid
/// order at compile time. Each method consumes the `CDL` instance and returns a new
/// one with an updated state.
#[derive(Debug)]
pub struct CDL<State> {
    state: State,
    config: CdlConfig,
}

impl<State> CDL<State> {
    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn config(&self) -> &CdlConfig {
        &self.config
    }
}

// See the various cdl_with files for all the type state implementations

// After process is finalized
impl CDL<Finalized> {
    /// Builds the final, executable runner for the pipeline.
    pub fn build(self) -> Result<CDLRunner, CdlError> {
        let CDL { state, config } = self; // Destructure self
        Ok(CDLRunner {
            result: state.0,
            config, // Use the destructured config
        })
    }
}

impl CDLRunner {
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
