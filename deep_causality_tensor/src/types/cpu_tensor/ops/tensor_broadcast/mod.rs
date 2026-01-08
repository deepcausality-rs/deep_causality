/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalTensorError;
use crate::types::cpu_tensor::InternalCpuTensor;

// Private helper methods
impl<T> InternalCpuTensor<T>
where
    T: Clone,
{
    /// Helper for element-wise binary operations with broadcasting.
    /// This function determines the broadcasted shape, iterates through the result
    /// coordinates, and applies the given operation `op`.
    pub(crate) fn broadcast_op<F, U>(
        &self,
        rhs: &InternalCpuTensor<U>,
        op: F,
    ) -> Result<Self, CausalTensorError>
    where
        U: Clone,
        T: Clone,
        F: Fn(T, U) -> Result<T, CausalTensorError>, // op now returns Result
    {
        // 1. Handle empty tensors
        if self.is_empty() && rhs.is_empty() {
            // If both are empty, the result is an empty scalar tensor
            return InternalCpuTensor::new(vec![], vec![0]);
        }
        if self.is_empty() || rhs.is_empty() {
            // If only one is empty, it's a shape mismatch for binary ops
            return Err(CausalTensorError::ShapeMismatch);
        }

        // 2. Determine broadcast shape
        let self_ndim = self.ndim();
        let rhs_ndim = rhs.ndim();
        let res_ndim = self_ndim.max(rhs_ndim);
        let mut result_shape = vec![0; res_ndim];

        for i in 0..res_ndim {
            // Calculate indices from the right, safely
            let self_current_dim_from_right = i; // 0 for rightmost, 1 for second rightmost, etc.
            let rhs_current_dim_from_right = i;

            let self_dim_val = if self_current_dim_from_right < self_ndim {
                self.shape[self_ndim - 1 - self_current_dim_from_right]
            } else {
                1 // Dimension does not exist, treat as size 1 for broadcasting
            };

            let rhs_dim_val = if rhs_current_dim_from_right < rhs_ndim {
                rhs.shape[rhs_ndim - 1 - rhs_current_dim_from_right]
            } else {
                1 // Dimension does not exist, treat as size 1 for broadcasting
            };

            let res_i = res_ndim - 1 - i; // Index into result_shape from the right

            if self_dim_val != rhs_dim_val && self_dim_val != 1 && rhs_dim_val != 1 {
                return Err(CausalTensorError::ShapeMismatch);
            }
            result_shape[res_i] = self_dim_val.max(rhs_dim_val);
        }

        // 3. Create result tensor and iterate to fill data
        let result_len = result_shape.iter().product();
        let mut result_data = Vec::with_capacity(result_len);
        let mut current_index = vec![0; res_ndim];

        for _ in 0..result_len {
            let self_flat_idx = self.get_flat_index_broadcasted(&current_index);
            let rhs_flat_idx = rhs.get_flat_index_broadcasted(&current_index);

            let self_val = self.data[self_flat_idx].clone();
            let rhs_val = rhs.data[rhs_flat_idx].clone();

            result_data.push(op(self_val, rhs_val)?); // Handle Result from op

            // Increment multi-dimensional index
            for j in (0..res_ndim).rev() {
                current_index[j] += 1;
                if current_index[j] < result_shape[j] {
                    break;
                }
                current_index[j] = 0;
            }
        }

        InternalCpuTensor::new(result_data, result_shape)
    }

    /// Calculates the flat index for a broadcasted tensor. Assumes inputs are valid.
    pub(crate) fn get_flat_index_broadcasted(&self, result_index: &[usize]) -> usize {
        let mut flat_index = 0;
        let self_ndim = self.ndim();
        let result_ndim = result_index.len();

        // Iterate through the dimensions of the result_index from right to left
        for i in 1..=result_ndim {
            let result_dim_val = result_index[result_ndim - i]; // Value in the result_index for this dimension

            // Check if 'self' has a corresponding dimension at this position from the right
            if i <= self_ndim {
                let self_dim_idx = self_ndim - i; // Actual index into self.shape/strides

                // If self's dimension is 1, it broadcasts, so its index contribution is 0
                if self.shape[self_dim_idx] == 1 {
                    // No change to flat_index
                } else {
                    flat_index += result_dim_val * self.strides[self_dim_idx];
                }
            }
            // If i > self_ndim, it means this dimension was padded in 'self',
            // so its contribution to flat_index is 0.
        }
        flat_index
    }

    /// Helper to create a new tensor with a given shape and fill it with a value.
    pub(crate) fn full(shape: &[usize], value: T) -> Self
    where
        T: Clone,
    {
        let len = shape.iter().product();
        let data = vec![value; len];
        Self::from_vec_and_shape_unchecked(data, shape)
    }

    /// Helper for element-wise unary operations.
    /// Applies the given operation `op` to each element.
    pub(crate) fn unary_op<F>(&self, op: F) -> Result<Self, CausalTensorError>
    where
        T: Clone,
        F: Fn(T) -> Result<T, CausalTensorError>,
    {
        let mut result_data = Vec::with_capacity(self.data.len());
        for value in self.data.iter().cloned() {
            result_data.push(op(value)?);
        }
        Ok(Self::from_vec_and_shape_unchecked(
            result_data,
            self.shape(),
        ))
    }
}
