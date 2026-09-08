/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Precision;
use crate::errors::PreprocessError;
use deep_causality_stats::MeanAccumulator;
use deep_causality_tensor::CausalTensor;

/// A pre-processor for handling missing numerical data in a `CausalTensor`.
///
/// This struct provides static methods to perform imputation. It is intended to be used
/// within a CDL pipeline but can also be used as a standalone utility.
pub struct MissingValueImputer;

impl MissingValueImputer {
    /// Imputes missing values (NaN) in a 2D tensor with the mean of their respective columns.
    ///
    /// # Arguments
    ///
    /// * `tensor` - The input `CausalTensor<f64>` which may contain NaN values.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `CausalTensor<f64>` with missing values imputed, or a `PreprocessError`.
    ///
    pub fn impute_mean<T: Precision>(
        tensor: CausalTensor<T>,
    ) -> Result<CausalTensor<T>, PreprocessError> {
        let shape = tensor.shape().to_vec();
        if shape.len() != 2 {
            return Err(PreprocessError::ImputeError(
                "Tensor must be 2-dimensional for column-wise imputation".to_string(),
            ));
        }

        let n_rows = shape[0];
        let n_cols = shape[1];

        if n_rows == 0 || n_cols == 0 {
            return Ok(tensor); // Nothing to impute
        }

        let mut data = tensor.as_slice().to_vec();

        for c in 0..n_cols {
            // `MeanAccumulator` rather than `mean` over a slice: the present values are found in
            // the same pass that records the missing ones, and a column is not contiguous in a
            // row-major matrix, so a slice would have to be built to be thrown away.
            let mut present = MeanAccumulator::<T>::new();
            let mut missing_indices = Vec::new();

            // First pass: find missing values and accumulate the non-missing.
            for r in 0..n_rows {
                let index = r * n_cols + c;
                if data[index].is_nan() {
                    missing_indices.push(index);
                } else {
                    present.push(data[index]);
                }
            }

            // If there are missing values in this column, impute them.
            if !missing_indices.is_empty() {
                // A column that is entirely NaN has no mean to impute from; zero is this
                // preprocessor's answer, where the statistics crate refuses with `EmptyInput`.
                let mean = present.mean().unwrap_or_else(|_| T::zero());
                for index in missing_indices {
                    data[index] = mean;
                }
            }
        }

        CausalTensor::new(data, shape).map_err(|e| PreprocessError::ImputeError(e.to_string()))
    }
}
