/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{PreprocessConfig, PreprocessError};
use deep_causality_tensor::CausalTensor;

/// Defines the contract for data pre-processing steps.
///
/// Implementors of this trait provide specific data transformation logic, such as
/// discretization or missing value imputation, which can be applied as part of a
/// CDL pipeline.
pub trait DataPreprocessor<T> {
    /// Processes the input tensor according to the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The input `CausalTensor<T>` to be processed.
    /// * `config` - A `PreprocessConfig` containing the settings for this
    ///   processing step.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new, processed `CausalTensor<T>` on success.
    ///
    /// # Errors
    ///
    /// Returns a `PreprocessError` if the processing fails, for example due to an
    /// invalid configuration or an issue with the data itself.
    fn process(
        &self,
        tensor: CausalTensor<T>,
        config: &PreprocessConfig,
    ) -> Result<CausalTensor<T>, PreprocessError>;
}
