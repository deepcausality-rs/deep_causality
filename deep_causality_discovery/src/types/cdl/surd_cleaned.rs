/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{SurdCleaned, SurdFeatures};
use crate::{CDL, CdlBuilder, CdlEffect, CdlError, FeatureSelectError, Precision};
use deep_causality_algorithms::feature_selection::mrmr::mrmr_features_selector;
use deep_causality_num::Float;
use deep_causality_tensor::CausalTensor;
use std::fmt::Debug;

// `feature_select` calls MRMR, whose `F: Float` bound `RealField`/`Precision` does
// not provide (the blanket impl runs `Float ⇒ RealField`, not the reverse). The
// `Float + Debug + 'static` bound is added here only, so it never leaks into the
// rest of the pipeline (where it would collide with `Real::nan`/`is_nan`).
impl<T: Precision + Float + Debug + 'static> CDL<SurdCleaned<T>> {
    /// Selects features with MRMR, using the feature count and target index from
    /// the run config (no inline parameters).
    pub fn feature_select(self) -> CdlEffect<CDL<SurdFeatures<T>>> {
        let config = self.state.config;
        let tensor = self.state.tensor;

        let selection_res = mrmr_features_selector::<Option<T>, T>(
            &tensor,
            config.num_features(),
            config.target_index(),
        );

        let mrmr_result = match selection_res {
            Ok(r) => r,
            Err(e) => {
                return CdlEffect {
                    inner: Err(CdlError::FeatSelectError(FeatureSelectError::MrmrError(e))),
                    warnings: Default::default(),
                };
            }
        };

        // Filter the tensor columns to the selected indices.
        let selected_indices: Vec<usize> = mrmr_result.iter().map(|(idx, _)| *idx).collect();
        let rows = tensor.shape()[0];
        let cols = selected_indices.len();
        let mut data: Vec<Option<T>> = Vec::with_capacity(rows * cols);
        for r in 0..rows {
            for &c_idx in &selected_indices {
                if let Some(val) = tensor.get(&[r, c_idx]) {
                    data.push(*val);
                } else {
                    data.push(None);
                }
            }
        }

        match CausalTensor::new(data, vec![rows, cols]) {
            Ok(filtered_tensor) => CdlBuilder::pure(CDL {
                state: SurdFeatures {
                    tensor: filtered_tensor,
                    selection_result: mrmr_result,
                    records_count: self.state.records_count,
                    config,
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::FeatSelectError(FeatureSelectError::TensorError(
                    e,
                ))),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage method on the effect.
impl<T: Precision + Float + Debug + 'static> CdlEffect<CDL<SurdCleaned<T>>> {
    /// See [`CDL::<SurdCleaned<T>>::feature_select`].
    pub fn feature_select(self) -> CdlEffect<CDL<SurdFeatures<T>>> {
        self.and_then(|cdl| cdl.feature_select())
    }
}
