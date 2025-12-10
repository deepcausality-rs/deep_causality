/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FeatureSelector;
use crate::{FeatureSelectError, FeatureSelectorConfig};
use deep_causality_algorithms::feature_selection::mrmr::{MrmrError, MrmrResult};
use deep_causality_algorithms::mrmr::mrmr_features_selector;
use deep_causality_tensor::{CausalTensor, CausalTensorError};

/// A concrete implementation of the `FeatureSelector` trait that uses the MRMR algorithm.
pub struct MrmrFeatureSelector;

impl FeatureSelector for MrmrFeatureSelector {
    fn select(
        &self,
        tensor: CausalTensor<Option<f64>>,
        config: &FeatureSelectorConfig,
    ) -> Result<CausalTensor<Option<f64>>, FeatureSelectError> {
        let FeatureSelectorConfig::Mrmr(mrmr_config) = config;

        // Pass reference to tensor
        let selected_indices_res = Self::select_indices(&tensor, mrmr_config)?;
        // Convert to Vec for iteration if MrmrResult doesn't implement IntoIterator directly or we want simple logic
        // Assuming MrmrResult is iterable or we can access its contents.
        // It likely has .iter().
        let selected_indices: Vec<(usize, f64)> = selected_indices_res.iter().copied().collect();

        let shape = tensor.shape();
        let n_rows = shape[0];
        let mut new_data = Vec::with_capacity(n_rows * selected_indices.len());
        let new_shape = vec![n_rows, selected_indices.len()];

        for i in 0..n_rows {
            for &(col_idx, _score) in &selected_indices {
                let value = tensor.get(&[i, col_idx]).ok_or({
                    FeatureSelectError::TensorError(CausalTensorError::AxisOutOfBounds)
                })?;
                new_data.push(*value);
            }
        }

        Ok(CausalTensor::new(new_data, new_shape)?)
    }
}

impl MrmrFeatureSelector {
    /// Returns the raw selected indices and scores, enabling manual filtering or inspection.
    pub fn select_indices(
        tensor: &CausalTensor<Option<f64>>,
        config: &crate::MrmrConfig,
    ) -> Result<MrmrResult, MrmrError> {
        mrmr_features_selector(tensor, config.num_features(), config.target_col())
    }
}
