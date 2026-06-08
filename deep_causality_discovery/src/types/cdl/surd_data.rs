/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::cdl::{SurdCleaned, SurdData};
use crate::{
    CDL, CausalDiscoveryError, CdlBuilder, CdlEffect, CdlError, DataCleaner, DataPreprocessor,
    Precision, PreprocessConfig,
};
use deep_causality_tensor::CausalTensor;

// After data is loaded (SURD lineage)
impl<T: Precision> CDL<SurdData<T>> {
    /// Optional preprocessing step. The binning/imputation config is passed
    /// explicitly because it is off the canonical chain.
    pub fn preprocess<P>(
        self,
        preprocessor: P,
        config: &PreprocessConfig,
    ) -> CdlEffect<CDL<SurdData<T>>>
    where
        P: DataPreprocessor<T>,
    {
        match preprocessor.process(self.state.tensor, config) {
            Ok(tensor) => CdlBuilder::pure(CDL {
                state: SurdData {
                    tensor,
                    records_count: self.state.records_count,
                    config: self.state.config,
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(e.into()),
                warnings: Default::default(),
            },
        }
    }

    /// Cleans the data using a provided DataCleaner.
    pub fn clean_data<C>(self, cleaner: C) -> CdlEffect<CDL<SurdCleaned<T>>>
    where
        C: DataCleaner<T>,
    {
        match cleaner.process(self.state.tensor) {
            Ok(t) => CdlBuilder::pure(CDL {
                state: SurdCleaned {
                    tensor: t,
                    records_count: self.state.records_count,
                    config: self.state.config,
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(e.into()),
                warnings: Default::default(),
            },
        }
    }

    /// Filters the dataset rows based on a provided predicate.
    ///
    /// `predicate` takes a row slice `&[T]` and returns `true` to keep the row.
    pub fn filter_cohort<P>(self, predicate: P) -> CdlEffect<CDL<SurdData<T>>>
    where
        P: Fn(&[T]) -> bool,
    {
        let tensor = &self.state.tensor;
        let rows = tensor.shape()[0];
        let cols = tensor.shape()[1];
        let data_slice = tensor.as_slice();

        let mut filtered_data = Vec::with_capacity(data_slice.len() / 2);
        let mut new_row_count = 0;

        for r in 0..rows {
            let start_idx = r * cols;
            let end_idx = start_idx + cols;

            if end_idx > data_slice.len() {
                return CdlEffect {
                    inner: Err(CdlError::CausalDiscoveryError(
                        CausalDiscoveryError::TensorError(
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

        match CausalTensor::new(filtered_data, vec![new_row_count, cols]) {
            Ok(new_tensor) => CdlBuilder::pure(CDL {
                state: SurdData {
                    tensor: new_tensor,
                    records_count: new_row_count,
                    config: self.state.config,
                },
            }),
            Err(e) => CdlEffect {
                inner: Err(CdlError::CausalDiscoveryError(
                    CausalDiscoveryError::TensorError(e),
                )),
                warnings: Default::default(),
            },
        }
    }
}

// Fluent stage methods on the effect.
impl<T: Precision> CdlEffect<CDL<SurdData<T>>> {
    /// See [`CDL::<SurdData<T>>::preprocess`].
    pub fn preprocess<P>(
        self,
        preprocessor: P,
        config: &PreprocessConfig,
    ) -> CdlEffect<CDL<SurdData<T>>>
    where
        P: DataPreprocessor<T>,
    {
        // The config borrow must outlive the closure; clone it in.
        let config = config.clone();
        self.and_then(move |cdl| cdl.preprocess(preprocessor, &config))
    }

    /// See [`CDL::<SurdData<T>>::clean_data`].
    pub fn clean_data<C>(self, cleaner: C) -> CdlEffect<CDL<SurdCleaned<T>>>
    where
        C: DataCleaner<T>,
    {
        self.and_then(move |cdl| cdl.clean_data(cleaner))
    }

    /// See [`CDL::<SurdData<T>>::filter_cohort`].
    pub fn filter_cohort<P>(self, predicate: P) -> CdlEffect<CDL<SurdData<T>>>
    where
        P: Fn(&[T]) -> bool,
    {
        self.and_then(move |cdl| cdl.filter_cohort(predicate))
    }
}
