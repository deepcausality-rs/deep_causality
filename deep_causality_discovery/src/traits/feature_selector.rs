/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{FeatureSelectError, FeatureSelectorConfig};
use deep_causality_tensor::CausalTensor;

/// Defines the contract for feature selection algorithms.
///
/// This trait abstracts the process of selecting a subset of relevant features (columns)
/// from a larger dataset, which is a common step in preparing data for causal discovery.
pub trait FeatureSelector {
    /// Selects a subset of features from the input tensor.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The input `CausalTensor<f64>` containing the full set of features.
    /// * `config` - A `FeatureSelectorConfig` enum containing the specific settings
    ///   for the selection algorithm (e.g., `MrmrConfig`).
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `CausalTensor<f64>` with only the selected
    /// feature columns on success.
    ///
    /// # Errors
    ///
    /// Returns a `FeatureSelectError` if the selection process fails, for example if
    /// the number of requested features is invalid.
    fn select(
        &self,
        tensor: CausalTensor<f64>,
        config: &FeatureSelectorConfig,
    ) -> Result<CausalTensor<f64>, FeatureSelectError>;
}
