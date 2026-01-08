/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DataLoaderConfig, DataLoadingError};
use deep_causality_tensor::CausalTensor;

/// Defines the contract for loading data from a source into a `CausalTensor`.
///
/// Implementors of this trait handle the specifics of reading different file formats
/// (e.g., CSV, Parquet) and converting the tabular data into the tensor representation
/// required by the CDL pipeline.
pub trait DataLoader {
    /// Loads data from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A string slice representing the path to the data file.
    /// * `config` - A `DataLoaderConfig` enum containing format-specific settings,
    ///   such as whether a CSV has headers or the batch size for a Parquet file.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `CausalTensor<f64>` with the loaded data on success.
    /// The tensor is expected to be 2-dimensional (rows, columns).
    ///
    /// # Errors
    ///
    /// Returns a `DataError` if loading fails, which can be due to the file not
    /// being found, permission issues, or parsing errors.
    fn load(
        &self,
        path: &str,
        config: &DataLoaderConfig,
    ) -> Result<CausalTensor<f64>, DataLoadingError>;
}
