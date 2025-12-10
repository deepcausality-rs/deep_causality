/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CdlConfig, ProcessAnalysis};
use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

mod cdl_with_analysis;
mod cdl_with_causal_results;
mod cdl_with_cleaned_data;
mod cdl_with_data;
mod cdl_with_features;
mod cdl_with_no_data;

// Typestate structs representing the pipeline's state.
/// Initial state of the CDL pipeline, with no data loaded.
#[derive(Debug)]
pub struct NoData;
/// State after data has been successfully loaded.
#[derive(Debug)]
pub struct WithData {
    pub tensor: CausalTensor<f64>,
    pub records_count: usize,
}
/// State after data has been cleaned.
#[derive(Debug)]
pub struct WithCleanedData {
    pub tensor: CausalTensor<Option<f64>>,
    pub records_count: usize,
}
/// State after feature selection has been applied.
#[derive(Debug)]
pub struct WithFeatures {
    pub tensor: CausalTensor<Option<f64>>,
    pub selection_result: MrmrResult,
    pub records_count: usize,
}
/// State after a causal discovery algorithm has been run.
#[derive(Debug)]
pub struct WithCausalResults {
    pub surd_result: SurdResult<f64>,
    pub selection_result: MrmrResult,
    pub records_count: usize,
}
/// State after the raw causal results have been analyzed.
#[derive(Debug)]
pub struct WithAnalysis {
    pub analysis: ProcessAnalysis,
    pub surd_result: SurdResult<f64>,
    pub selection_result: MrmrResult,
    pub records_count: usize,
}
/// Final state (marker).
#[derive(Debug)]
pub struct Finalized; // Finalize now returns CdlReport, so this might be unused or just a marker.

/// The core builder for the Causal Discovery Language (CDL) pipeline.
///
/// `CDL` uses a typestate pattern to ensure that pipeline steps are called in a valid
/// order at compile time. Each method consumes the `CDL` instance and returns a new
/// one with an updated state.
#[derive(Debug)]
pub struct CDL<State> {
    pub state: State,
    pub config: CdlConfig,
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
