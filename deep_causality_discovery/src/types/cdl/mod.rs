/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::brcd_input::BrcdInput;
use crate::types::cdl_discovery_outcome::CdlDiscoveryOutcome;
use crate::{BrcdLoaderConfig, ProcessAnalysis, SurdLoaderConfig};
use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

mod brcd_configured;
mod brcd_loaded;
mod brcd_results;
mod cdl_with_analysis;
mod surd_cleaned;
mod surd_configured;
mod surd_data;
mod surd_features;
mod surd_results;

// Typestate structs representing the pipeline's state.
//
// The pipeline has two compile-time-isolated sub-pipelines that share no state until
// the converged `WithAnalysis<T>`. Each sub-pipeline is seeded by a dedicated
// `CdlBuilder` entry that carries the run config built by `CdlConfigBuilder`:
//
//   SURD: SurdConfigured → SurdData → SurdCleaned → SurdFeatures → SurdResults → WithAnalysis
//   BRCD: BrcdConfigured → BrcdLoaded → BrcdResults → WithAnalysis
//
// Each sub-pipeline threads its own run config (`SurdLoaderConfig` / `BrcdLoaderConfig`)
// through its states; there is no separate master config object. Each algorithm's
// `*_discover` / `*_analyze` methods are implemented only on its own states, so
// crossing the sub-pipelines is a compile error.

// --- SURD sub-pipeline -----------------------------------------------------------

/// SURD entry state, carrying the run config from `CdlBuilder::build_surd`.
#[derive(Debug, Clone)]
pub struct SurdConfigured<T> {
    pub config: SurdLoaderConfig<T>,
}

/// SURD state after data has been loaded.
#[derive(Debug, Clone)]
pub struct SurdData<T> {
    pub tensor: CausalTensor<T>,
    pub records_count: usize,
    pub config: SurdLoaderConfig<T>,
}

/// SURD state after data has been cleaned (masked to `Option<T>`).
#[derive(Debug)]
pub struct SurdCleaned<T> {
    pub tensor: CausalTensor<Option<T>>,
    pub records_count: usize,
    pub config: SurdLoaderConfig<T>,
}

/// SURD state after feature selection has been applied.
#[derive(Debug, Clone)]
pub struct SurdFeatures<T> {
    pub tensor: CausalTensor<Option<T>>,
    pub selection_result: MrmrResult,
    pub records_count: usize,
    pub config: SurdLoaderConfig<T>,
}

/// SURD state after the SURD algorithm has run.
#[derive(Debug)]
pub struct SurdResults<T> {
    pub surd_result: SurdResult<T>,
    pub selection_result: MrmrResult,
    pub records_count: usize,
    pub config: SurdLoaderConfig<T>,
}

// --- BRCD sub-pipeline -----------------------------------------------------------

/// BRCD entry state, carrying the run config from `CdlBuilder::build_brcd`.
#[derive(Debug, Clone)]
pub struct BrcdConfigured<T> {
    pub config: BrcdLoaderConfig<T>,
}

/// BRCD state after the input bundle has been loaded.
#[derive(Debug)]
pub struct BrcdLoaded<T> {
    pub input: BrcdInput<T>,
    /// Human-readable source label for the report (the normal/anomalous paths).
    pub dataset_path: String,
}

/// BRCD state after the BRCD algorithm has run.
#[derive(Debug)]
pub struct BrcdResults<T> {
    pub brcd_result: deep_causality_algorithms::brcd::BrcdResult<T>,
    pub records_count: usize,
    pub dataset_path: String,
}

// --- Converged tail ---------------------------------------------------------

/// State after the raw discovery result has been analyzed. Both sub-pipelines
/// converge here, carrying the polymorphic [`CdlDiscoveryOutcome`].
#[derive(Debug)]
pub struct WithAnalysis<T> {
    pub analysis: ProcessAnalysis,
    pub outcome: CdlDiscoveryOutcome<T>,
    pub feature_selection: Option<MrmrResult>,
    pub records_count: usize,
    pub dataset_path: String,
}

/// The core builder for the Causal Discovery Language (CDL) pipeline.
///
/// `CDL` uses a typestate pattern to ensure that pipeline steps are called in a
/// valid order at compile time. Each method consumes the `CDL` instance and
/// returns a new one with an updated state. The run config travels inside the
/// state, so `CDL` carries no separate config object.
#[derive(Debug, Clone)]
pub struct CDL<State> {
    pub state: State,
}

impl<State> CDL<State> {
    pub fn state(&self) -> &State {
        &self.state
    }
}

// See the various per-sub-pipeline files for all the typestate implementations.
