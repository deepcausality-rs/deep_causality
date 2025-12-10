/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{WithCleanedData, WithFeatures};
use crate::{CDL, CdlBuilder, CdlEffect, CdlError, FeatureSelectError};
use deep_causality_algorithms::feature_selection::mrmr::{MrmrError, MrmrResult};
use deep_causality_tensor::CausalTensor;

impl CDL<WithCleanedData> {
    /// Selects features using a provided closure from cleaned data.
    pub fn feature_select<F>(self, selector_fn: F) -> CdlEffect<CDL<WithFeatures>>
    where
        F: FnOnce(&CausalTensor<Option<f64>>) -> Result<MrmrResult, MrmrError>,
    {
        // Data is already cleaned (Option<f64>)
        let tensor = &self.state.tensor;

        // 1. Invoke the selection closure
        let selection_res = selector_fn(tensor);

        match selection_res {
            Ok(mrmr_result) => {
                // 2. Filter the tensor columns based on selection
                let selected_indices: Vec<usize> =
                    mrmr_result.iter().map(|(idx, _)| *idx).collect();

                // Helper to filter columns:
                let rows = tensor.shape()[0];
                let cols = selected_indices.len();
                let mut data: Vec<Option<f64>> = Vec::with_capacity(rows * cols);

                for r in 0..rows {
                    for &c_idx in &selected_indices {
                        if let Some(val) = tensor.get(&[r, c_idx]) {
                            data.push(*val);
                        } else {
                            data.push(None);
                        }
                    }
                }

                // Construct new filtered tensor
                let filtered_tensor_res = CausalTensor::new(data, vec![rows, cols]);

                match filtered_tensor_res {
                    Ok(filtered_tensor) => {
                        let with_features_state = WithFeatures {
                            tensor: filtered_tensor,
                            selection_result: mrmr_result,
                            records_count: self.state.records_count,
                        };

                        CdlBuilder::pure(CDL {
                            state: with_features_state,
                            config: self.config,
                        })
                    }
                    Err(e) => CdlEffect {
                        inner: Err(CdlError::FeatSelectError(FeatureSelectError::TensorError(
                            e,
                        ))),
                        warnings: Default::default(),
                    },
                }
            }
            Err(e) => CdlEffect {
                inner: Err(CdlError::FeatSelectError(FeatureSelectError::MrmrError(e))),
                warnings: Default::default(),
            },
        }
    }
}
