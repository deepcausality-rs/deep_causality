/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CsvConfig, SurdAnalyzeConfig};
use deep_causality_algorithms::surd::MaxOrder;
use std::fmt;
use std::marker::PhantomData;

/// The fully-specified configuration for a SURD CDL run.
///
/// Constructed only through [`crate::CdlConfigBuilder::build_surd_config`] (the
/// single source of truth), so every required parameter is present and explicit:
/// there are no hidden algorithm defaults. The pipeline precision `T` is carried
/// in the type so the config-driven `build_surd` pipeline needs no turbofish.
#[derive(Debug, Clone)]
pub struct SurdLoaderConfig<T> {
    path: String,
    target_index: usize,
    exclude_indices: Vec<usize>,
    num_features: usize,
    max_order: MaxOrder,
    analyze: SurdAnalyzeConfig,
    csv: CsvConfig,
    _precision: PhantomData<T>,
}

impl<T> SurdLoaderConfig<T> {
    /// Builder-only constructor.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        path: String,
        target_index: usize,
        exclude_indices: Vec<usize>,
        num_features: usize,
        max_order: MaxOrder,
        analyze: SurdAnalyzeConfig,
        csv: CsvConfig,
    ) -> Self {
        Self {
            path,
            target_index,
            exclude_indices,
            num_features,
            max_order,
            analyze,
            csv,
            _precision: PhantomData,
        }
    }
}

// Getters
impl<T> SurdLoaderConfig<T> {
    /// The dataset file path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The target column index.
    pub fn target_index(&self) -> usize {
        self.target_index
    }

    /// Columns excluded during load.
    pub fn exclude_indices(&self) -> &[usize] {
        &self.exclude_indices
    }

    /// Number of features MRMR selects.
    pub fn num_features(&self) -> usize {
        self.num_features
    }

    /// The maximum SURD interaction order.
    pub fn max_order(&self) -> MaxOrder {
        self.max_order
    }

    /// The SURD analysis thresholds.
    pub fn analyze(&self) -> &SurdAnalyzeConfig {
        &self.analyze
    }

    /// The CSV parse options.
    pub fn csv(&self) -> &CsvConfig {
        &self.csv
    }
}

impl<T> fmt::Display for SurdLoaderConfig<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SurdLoaderConfig(path: {}, target: {}, num_features: {}, max_order: {})",
            self.path, self.target_index, self.num_features, self.max_order
        )
    }
}
