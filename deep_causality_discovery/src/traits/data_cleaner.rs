/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::DataCleaningError;
use deep_causality_tensor::CausalTensor;

pub trait DataCleaner<T> {
    /// Cleans the input tensor according to the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The input `CausalTensor<T>` to be processed.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new, processed `CausalTensor<Option<T>>` on success.
    ///
    /// # Errors
    ///
    /// Returns a `PreprocessError` if the processing fails, for example due to an
    /// invalid configuration or an issue with the data itself.
    fn process(
        &self,
        tensor: CausalTensor<T>,
    ) -> Result<CausalTensor<Option<T>>, DataCleaningError>;
}
