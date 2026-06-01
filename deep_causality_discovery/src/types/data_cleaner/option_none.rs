/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DataCleaner, DataCleaningError, Precision};
use deep_causality_tensor::CausalTensor;

pub struct OptionNoneDataCleaner;

impl<T: Precision> DataCleaner<T> for OptionNoneDataCleaner {
    /// Processes a `CausalTensor<T>` by replacing `NaN` values with `None` and
    /// wrapping valid values in `Some`.
    ///
    /// # Arguments
    /// * `tensor` - The input `CausalTensor<T>` to be cleaned.
    ///
    /// # Returns
    /// A `Result` containing a `CausalTensor<Option<T>>` on success, or a `DataCleaningError` on failure.
    fn process(
        &self,
        tensor: CausalTensor<T>,
    ) -> Result<CausalTensor<Option<T>>, DataCleaningError> {
        let shape = tensor.shape().to_vec();
        let data: Vec<Option<T>> = tensor
            .as_slice()
            .iter()
            .map(|&val| if val.is_nan() { None } else { Some(val) })
            .collect();

        CausalTensor::new(data, shape).map_err(DataCleaningError::TensorError)
    }
}
