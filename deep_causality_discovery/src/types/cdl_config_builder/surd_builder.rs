/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Staged builder for [`SurdLoaderConfig`]. The five required fields are
//! sequential stages; the optional `exclude_indices` / `csv` and `build()` live on
//! the final ready stage. The pipeline precision `T` is pinned at
//! `build_surd_config::<T>()` and threaded through the stages.

use crate::types::cdl_config_builder::check_file_exists;
use crate::{CdlError, CsvConfig, SurdAnalyzeConfig, SurdLoaderConfig};
use deep_causality_algorithms::surd::MaxOrder;
use std::marker::PhantomData;

/// Stage 0: needs the dataset path.
pub struct SurdConfigNeedsPath<T> {
    _precision: PhantomData<T>,
}

impl<T> Default for SurdConfigNeedsPath<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SurdConfigNeedsPath<T> {
    pub(crate) fn new() -> Self {
        Self {
            _precision: PhantomData,
        }
    }

    /// Sets the dataset file path (required).
    pub fn with_path(self, path: impl Into<String>) -> SurdConfigNeedsTarget<T> {
        SurdConfigNeedsTarget {
            path: path.into(),
            _precision: PhantomData,
        }
    }
}

/// Stage 1: needs the target column index.
pub struct SurdConfigNeedsTarget<T> {
    path: String,
    _precision: PhantomData<T>,
}

impl<T> SurdConfigNeedsTarget<T> {
    /// Sets the target column index (required).
    pub fn with_target_index(self, target_index: usize) -> SurdConfigNeedsNumFeatures<T> {
        SurdConfigNeedsNumFeatures {
            path: self.path,
            target_index,
            _precision: PhantomData,
        }
    }
}

/// Stage 2: needs the MRMR feature count.
pub struct SurdConfigNeedsNumFeatures<T> {
    path: String,
    target_index: usize,
    _precision: PhantomData<T>,
}

impl<T> SurdConfigNeedsNumFeatures<T> {
    /// Sets the number of features MRMR selects (required).
    pub fn with_num_features(self, num_features: usize) -> SurdConfigNeedsMaxOrder<T> {
        SurdConfigNeedsMaxOrder {
            path: self.path,
            target_index: self.target_index,
            num_features,
            _precision: PhantomData,
        }
    }
}

/// Stage 3: needs the SURD max interaction order.
pub struct SurdConfigNeedsMaxOrder<T> {
    path: String,
    target_index: usize,
    num_features: usize,
    _precision: PhantomData<T>,
}

impl<T> SurdConfigNeedsMaxOrder<T> {
    /// Sets the maximum SURD interaction order (required).
    pub fn with_max_order(self, max_order: MaxOrder) -> SurdConfigNeedsAnalyze<T> {
        SurdConfigNeedsAnalyze {
            path: self.path,
            target_index: self.target_index,
            num_features: self.num_features,
            max_order,
            _precision: PhantomData,
        }
    }
}

/// Stage 4: needs the SURD analysis thresholds.
pub struct SurdConfigNeedsAnalyze<T> {
    path: String,
    target_index: usize,
    num_features: usize,
    max_order: MaxOrder,
    _precision: PhantomData<T>,
}

impl<T> SurdConfigNeedsAnalyze<T> {
    /// Sets the SURD analysis thresholds (required; explicit, no hidden default).
    pub fn with_analyze(self, analyze: SurdAnalyzeConfig) -> SurdConfigReady<T> {
        SurdConfigReady {
            path: self.path,
            target_index: self.target_index,
            num_features: self.num_features,
            max_order: self.max_order,
            analyze,
            exclude_indices: Vec::new(),
            csv: CsvConfig::default(),
            _precision: PhantomData,
        }
    }
}

/// Final stage: all required fields set; optional setters and `build()` available.
pub struct SurdConfigReady<T> {
    path: String,
    target_index: usize,
    num_features: usize,
    max_order: MaxOrder,
    analyze: SurdAnalyzeConfig,
    exclude_indices: Vec<usize>,
    csv: CsvConfig,
    _precision: PhantomData<T>,
}

impl<T> SurdConfigReady<T> {
    /// Sets the columns to exclude during load (defaults to none).
    pub fn with_exclude_indices(mut self, exclude_indices: Vec<usize>) -> Self {
        self.exclude_indices = exclude_indices;
        self
    }

    /// Overrides the CSV parse options (defaults to `CsvConfig::default`).
    pub fn with_csv(mut self, csv: CsvConfig) -> Self {
        self.csv = csv;
        self
    }

    /// Builds the [`SurdLoaderConfig`], verifying the dataset file exists.
    ///
    /// # Errors
    /// [`CdlError::ReadDataError`] if the dataset file is missing.
    pub fn build(self) -> Result<SurdLoaderConfig<T>, CdlError> {
        check_file_exists(&self.path)?;
        Ok(SurdLoaderConfig::new(
            self.path,
            self.target_index,
            self.exclude_indices,
            self.num_features,
            self.max_order,
            self.analyze,
            self.csv,
        ))
    }
}
