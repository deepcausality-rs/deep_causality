/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalTensor, CausalTensorError};

// Private helper methods
impl<T> CausalTensor<T>
where
    T: Copy + Default + PartialOrd,
{
    /// Calculates the flat index into the `data` vector from a multi-dimensional index.
    /// This is the core of the stride-based memory layout.
    pub(in crate::causal_tensor_type) fn get_flat_index(&self, index: &[usize]) -> Option<usize> {
        if index.len() != self.num_dim() {
            return None;
        }

        let mut flat_index = 0;
        for (i, &dim_index) in index.iter().enumerate() {
            if dim_index >= self.shape[i] {
                return None; // Index is out of bounds for this dimension
            }
            flat_index += dim_index * self.strides[i];
        }
        Some(flat_index)
    }

    /// Calculates the flat index for a broadcasted tensor. Assumes inputs are valid.
    fn get_flat_index_broadcasted(&self, index: &[usize]) -> usize {
        let mut flat_index = 0;
        let self_ndim = self.num_dim();
        let broadcast_ndim = index.len();
        let ndim_diff = broadcast_ndim - self_ndim;

        for i in 0..self_ndim {
            let broadcast_i = i + ndim_diff;
            let mut dim_index = index[broadcast_i];

            if self.shape[i] == 1 {
                dim_index = 0;
            }

            flat_index += dim_index * self.strides[i];
        }
        flat_index
    }

    /// Helper to create a new tensor with a given shape and fill it with a value.
    pub(in crate::causal_tensor_type) fn full(shape: &[usize], value: T) -> Self
    where
        T: Clone,
    {
        let len = shape.iter().product();
        let data = vec![value; len];
        Self::from_vec_and_shape_unchecked(data, shape)
    }

    /// Internal constructor that calculates strides.
    /// Call only when shape is known to be valid.
    pub(in crate::causal_tensor_type) fn from_vec_and_shape_unchecked(
        data: Vec<T>,
        shape: &[usize],
    ) -> Self {
        let mut strides = vec![0; shape.len()];
        if !shape.is_empty() {
            let mut current_stride = 1;
            for i in (0..shape.len()).rev() {
                strides[i] = current_stride;
                current_stride *= shape[i];
            }
        }
        Self {
            data,
            shape: shape.to_vec(),
            strides,
        }
    }

    /// An optimized helper for element-wise binary operations with broadcasting.
    pub(in crate::causal_tensor_type) fn binary_op<F, U>(
        &self,
        rhs: &CausalTensor<U>,
        op: F,
    ) -> Result<Self, CausalTensorError>
    where
        U: Copy + Default + PartialOrd,
        T: Copy + Default + PartialOrd,
        F: Fn(T, U) -> T,
    {
        // 1. Determine broadcast shape
        let self_ndim = self.num_dim();
        let rhs_ndim = rhs.num_dim();
        let res_ndim = self_ndim.max(rhs_ndim);
        let mut result_shape = vec![0; res_ndim];

        for i in 1..=res_ndim {
            let self_dim = self.shape.get(self_ndim.saturating_sub(i)).unwrap_or(&1);
            let rhs_dim = rhs.shape.get(rhs_ndim.saturating_sub(i)).unwrap_or(&1);
            let res_i = res_ndim - i;

            if *self_dim != *rhs_dim && *self_dim != 1 && *rhs_dim != 1 {
                return Err(CausalTensorError::ShapeMismatch);
            }
            result_shape[res_i] = *self_dim.max(rhs_dim);
        }

        // 2. Create result tensor and iterate to fill data
        let result_len = result_shape.iter().product();
        let mut result_data = Vec::with_capacity(result_len);
        let mut current_index = vec![0; res_ndim];

        for _ in 0..result_len {
            let self_flat_idx = self.get_flat_index_broadcasted(&current_index);
            let rhs_flat_idx = rhs.get_flat_index_broadcasted(&current_index);

            let self_val = self.data[self_flat_idx];
            let rhs_val = rhs.data[rhs_flat_idx];

            result_data.push(op(self_val, rhs_val));

            // Increment multi-dimensional index
            for j in (0..res_ndim).rev() {
                current_index[j] += 1;
                if current_index[j] < result_shape[j] {
                    break;
                }
                current_index[j] = 0;
            }
        }

        CausalTensor::new(result_data, result_shape)
    }
}
