/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Precision;
use crate::errors::PreprocessError;
use deep_causality_num::FromPrimitive;
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
            let mut sum = T::zero();
            let mut count = 0usize;
            let mut missing_indices = Vec::new();

            // First pass: find missing values and calculate sum of non-missing
            for r in 0..n_rows {
                let index = r * n_cols + c;
                if data[index].is_nan() {
                    missing_indices.push(index);
                } else {
                    sum += data[index];
                    count += 1;
                }
            }

            // If there are missing values in this column, impute them.
            if !missing_indices.is_empty() {
                let mean = if count > 0 {
                    sum / <T as FromPrimitive>::from_usize(count)
                        .expect("count is representable in RealField")
                } else {
                    // If a column consists entirely of NaN, default to zero.
                    T::zero()
                };

                for index in missing_indices {
                    data[index] = mean;
                }
            }
        }

        CausalTensor::new(data, shape).map_err(|e| PreprocessError::ImputeError(e.to_string()))
    }
}
