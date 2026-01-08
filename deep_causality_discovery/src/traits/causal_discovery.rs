/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalDiscoveryConfig, CausalDiscoveryError};
use deep_causality_algorithms::surd::SurdResult;
use deep_causality_tensor::CausalTensor;

/// Defines the contract for causal discovery algorithms.
///
/// This trait abstracts the core logic of a causal discovery method, which takes a data
/// tensor and a configuration, and returns a structured result representing the
/// discovered causal relationships.
pub trait CausalDiscovery {
    /// Performs a causal discovery analysis on the provided data.
    ///
    /// # Arguments
    ///
    /// * `tensor` - A `CausalTensor<f64>` representing the dataset. It's expected that
    ///   the data is already pre-processed and in the correct format for the algorithm.
    /// * `config` - A `CausalDiscoveryConfig` enum containing the specific settings
    ///   for the discovery algorithm (e.g., `SurdConfig`).
    ///
    /// # Returns
    ///
    /// A `Result` containing a `SurdResult<f64>` on success, which encapsulates the
    /// synergistic, unique, and redundant causal influences.
    ///
    /// # Errors
    ///
    /// Returns a `CausalDiscoveryError` if the analysis fails, for instance due to
    /// incompatible tensor shapes or other algorithm-specific issues.
    fn discover(
        &self,
        tensor: CausalTensor<Option<f64>>,
        config: &CausalDiscoveryConfig,
    ) -> Result<SurdResult<f64>, CausalDiscoveryError>;
}
