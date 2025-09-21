/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::FeatureSelectError;
use crate::traits::feature_selector::FeatureSelector;
use crate::types::config::FeatureSelectorConfig;
use deep_causality_algorithms::mrmr::select_features;
use deep_causality_tensor::CausalTensor;

/// A concrete implementation of the `FeatureSelector` trait that uses the MRMR algorithm.
pub struct MrmrFeatureSelector;

impl FeatureSelector for MrmrFeatureSelector {
    fn select(
        &self,
        mut tensor: CausalTensor<f64>,
        config: &FeatureSelectorConfig,
    ) -> Result<CausalTensor<f64>, FeatureSelectError> {
        let FeatureSelectorConfig::Mrmr(mrmr_config) = config;

        let selected_indices = select_features(
            &mut tensor,
            mrmr_config.num_features(),
            mrmr_config.target_col(),
        )?;

        // This part is tricky. We need to create a new tensor with only the selected columns.
        // CausalTensor does not have a `select_columns` method, so we have to do it manually.
        let shape = tensor.shape();
        let n_rows = shape[0];
        let mut new_data = Vec::with_capacity(n_rows * selected_indices.len());
        let new_shape = vec![n_rows, selected_indices.len()];

        for i in 0..n_rows {
            for &col_idx in &selected_indices {
                new_data.push(*tensor.get(&[i, col_idx]).unwrap());
            }
        }

        Ok(CausalTensor::new(new_data, new_shape)?)
    }
}
