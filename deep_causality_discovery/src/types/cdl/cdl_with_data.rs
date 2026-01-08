/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{WithData, WithFeatures};
use crate::{
    CDL, CdlBuilder, CdlEffect, CdlError, DataCleaner, DataPreprocessor, FeatureSelectError,
    OptionNoneDataCleaner,
};
use deep_causality_algorithms::feature_selection::mrmr::{MrmrError, MrmrResult};
use deep_causality_tensor::CausalTensor;

// After data is loaded
impl CDL<WithData> {
    pub fn preprocess<P>(self, preprocessor: P) -> CdlEffect<CDL<WithData>>
    where
        P: DataPreprocessor,
    {
        let config_opt = self.config.preprocess_config();

        let result = if let Some(config) = config_opt {
            preprocessor
                .process(self.state.tensor, config)
                .map(|t| CDL {
                    state: WithData {
                        tensor: t,
                        records_count: self.state.records_count,
                    },
                    config: self.config.clone(),
                })
        } else {
            Ok(self) // Pass through
        };

        match result {
            Ok(cdl) => CdlBuilder::pure(cdl),
            Err(e) => CdlEffect {
                inner: Err(e.into()),
                warnings: Default::default(),
            },
        }
    }

    /// Cleans the data using a provided DataCleaner.
    pub fn clean_data<C>(self, cleaner: C) -> CdlEffect<CDL<crate::types::cdl::WithCleanedData>>
    where
        C: DataCleaner,
    {
        match cleaner.process(self.state.tensor) {
            Ok(t) => CdlBuilder::pure(CDL {
                state: crate::types::cdl::WithCleanedData {
                    tensor: t,
                    records_count: self.state.records_count,
                },
                config: self.config,
            }),
            Err(e) => CdlEffect {
                inner: Err(e.into()),
                warnings: Default::default(),
            },
        }
    }

    /// Selects features using a provided closure.
    pub fn feature_select<F>(self, selector_fn: F) -> CdlEffect<CDL<WithFeatures>>
    where
        F: FnOnce(&CausalTensor<Option<f64>>) -> Result<MrmrResult, MrmrError>,
    {
        // 1. Clean data (f64 -> Option<f64>)
        let cleaner = OptionNoneDataCleaner;
        let cleaned_tensor_res = cleaner.process(self.state.tensor);

        let cleaned_tensor = match cleaned_tensor_res {
            Ok(t) => t,
            Err(e) => {
                // Map DataCleaningError to CdlError::CleanDataError
                // Explicit conversion or use Into/From if available. CdlError has From<DataCleaningError>.
                return CdlEffect {
                    inner: Err(e.into()),
                    warnings: Default::default(),
                };
            }
        };

        // 2. Invoke the selection closure
        let selection_res = selector_fn(&cleaned_tensor);

        match selection_res {
            Ok(mrmr_result) => {
                // MrmrResult is already constructed

                // 3. Filter the tensor columns based on selection
                let selected_indices: Vec<usize> =
                    mrmr_result.iter().map(|(idx, _)| *idx).collect();

                // Helper to filter columns:
                let rows = cleaned_tensor.shape()[0];
                let cols = selected_indices.len();
                let mut data: Vec<Option<f64>> = Vec::with_capacity(rows * cols);

                for r in 0..rows {
                    for &c_idx in &selected_indices {
                        // Safely get element. Assuming cleaned_tensor shape is correct.
                        // Standard layout is [row, col]
                        // We use internal get logic if possible, but get() returns Option<&T>.
                        if let Some(val) = cleaned_tensor.get(&[r, c_idx]) {
                            data.push(*val);
                        } else {
                            // Should not happen if indices are valid, but safe fallback
                            data.push(None);
                        }
                    }
                }

                // Construct new filtered tensor
                // Note: CausalTensor::new validates data len vs shape.
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
                // MrmrError -> FeatureSelectError -> CdlError
                inner: Err(CdlError::FeatSelectError(FeatureSelectError::MrmrError(e))),
                warnings: Default::default(),
            },
        }
    }

    /// Filters the dataset rows based on a provided predicate.
    ///
    /// # Arguments
    ///
    /// * `predicate` - A closure that takes a row slice `&[f64]` and returns `bool`.
    ///
    ///  Returns `true` to keep the row, `false` to discard it.
    pub fn filter_cohort<P>(self, predicate: P) -> CdlEffect<CDL<WithData>>
    where
        P: Fn(&[f64]) -> bool,
    {
        let tensor = &self.state.tensor;
        let rows = tensor.shape()[0];
        let cols = tensor.shape()[1];

        // Access data via public interface
        let data_slice = tensor.as_slice();

        // Estimate capacity (assume 50% retention to avoid immediate reallocation if filtering is aggressive)
        let mut filtered_data = Vec::with_capacity(data_slice.len() / 2);
        let mut new_row_count = 0;

        for r in 0..rows {
            let start_idx = r * cols;
            let end_idx = start_idx + cols;

            if end_idx > data_slice.len() {
                return CdlEffect {
                    inner: Err(CdlError::CausalDiscoveryError(
                        crate::CausalDiscoveryError::TensorError(
                            deep_causality_tensor::CausalTensorError::IndexOutOfBounds,
                        ),
                    )),
                    warnings: Default::default(),
                };
            }

            let row_slice = &data_slice[start_idx..end_idx];

            if predicate(row_slice) {
                filtered_data.extend_from_slice(row_slice);
                new_row_count += 1;
            }
        }

        // Create new tensor from filtered data
        match CausalTensor::new(filtered_data, vec![new_row_count, cols]) {
            Ok(new_tensor) => CdlBuilder::pure(CDL {
                state: WithData {
                    tensor: new_tensor,
                    records_count: new_row_count,
                },
                config: self.config,
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::CausalDiscoveryError(
                    crate::CausalDiscoveryError::TensorError(e),
                )),
                warnings: Default::default(),
            },
        }
    }
}
