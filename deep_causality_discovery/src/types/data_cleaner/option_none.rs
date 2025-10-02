/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DataCleaner, DataCleaningError};
use deep_causality_tensor::CausalTensor;

pub struct OptionNoneDataCleaner;

impl DataCleaner for OptionNoneDataCleaner {
    /// Processes a `CausalTensor<f64>` by replacing `NaN` values with `None` and
    /// wrapping valid `f64` values in `Some`.
    ///
    /// # Arguments
    /// * `tensor` - The input `CausalTensor<f64>` to be cleaned.
    ///
    /// # Returns
    /// A `Result` containing a `CausalTensor<Option<f64>>` on success, or a `DataCleaningError` on failure.
    fn process(
        &self,
        tensor: CausalTensor<f64>,
    ) -> Result<CausalTensor<Option<f64>>, DataCleaningError> {
        let shape = tensor.shape().to_vec();
        let data: Vec<Option<f64>> = tensor
            .as_slice()
            .iter()
            .map(|&val| if val.is_nan() { None } else { Some(val) })
            .collect();

        CausalTensor::new(data, shape).map_err(DataCleaningError::TensorError)
    }
}
