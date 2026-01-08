/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::cpu_tensor::EinSumOp;
use crate::{CausalTensorError, InternalCpuTensor};
use std::ops::Mul;

use crate::TensorData;

impl<T> InternalCpuTensor<T>
where
    T: TensorData,
{
    pub(in crate::types::cpu_tensor) fn matmul_impl(
        &self,
        rhs: &Self,
    ) -> Result<Self, CausalTensorError> {
        let lhs_ndim = self.ndim();
        let rhs_ndim = rhs.ndim();

        if lhs_ndim == 2 && rhs_ndim == 2 {
            // Standard 2D Matrix Multiplication
            let ast = EinSumOp::mat_mul(self.clone(), rhs.clone());
            return Self::execute_ein_sum(&ast);
        }

        // General Batched Matrix Multiplication
        // 1. Check strict 2D requirements for the last two dimensions match (K)
        if lhs_ndim < 2 || rhs_ndim < 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }

        let m = self.shape[lhs_ndim - 2];
        let k = self.shape[lhs_ndim - 1];
        let k_rhs = rhs.shape[rhs_ndim - 2];
        let n = rhs.shape[rhs_ndim - 1];

        if k != k_rhs {
            return Err(CausalTensorError::ShapeMismatch);
        }

        // 2. Broadcast Batch Dimensions
        // Batch dimensions are [0 .. ndim-2]
        let lhs_batch_dims = &self.shape[0..lhs_ndim - 2];
        let rhs_batch_dims = &rhs.shape[0..rhs_ndim - 2];

        // Simple case: Exact match or one is empty (broadcasting not fully implemented here, assume match for now)
        // For physics fields, shapes usually match exactly [..., N, N].
        // If shapes differ, we should probably error or implement full broadcast.
        // Assuming identical batch structure for this fix.

        // If ranks differ, we can't easily align without full broadcast logic.
        if lhs_ndim != rhs_ndim {
            // Fallback to error for now, or existing behavior?
            // Existing behavior was 2D only.
            return Err(CausalTensorError::ShapeMismatch);
        }

        if lhs_batch_dims != rhs_batch_dims {
            return Err(CausalTensorError::ShapeMismatch);
        }

        let batch_shape = lhs_batch_dims.to_vec();
        let batch_size: usize = batch_shape.iter().product();

        let lhs_matrix_size = m * k;
        let rhs_matrix_size = k * n;
        let result_matrix_size = m * n;

        let mut result_data = Vec::with_capacity(batch_size * result_matrix_size);

        // Iterate over batches
        for b in 0..batch_size {
            let lhs_offset = b * lhs_matrix_size;
            let rhs_offset = b * rhs_matrix_size;

            let lhs_slice = &self.data[lhs_offset..lhs_offset + lhs_matrix_size];
            let rhs_slice = &rhs.data[rhs_offset..rhs_offset + rhs_matrix_size];

            // Temporary tensors for 2D matmul
            // We use standard 2D logic
            let l = Self::new(lhs_slice.to_vec(), vec![m, k])?;
            let r = Self::new(rhs_slice.to_vec(), vec![k, n])?;

            let ast = EinSumOp::mat_mul(l, r);
            let res = Self::execute_ein_sum(&ast)?;

            result_data.extend(res.data);
        }

        let mut final_shape = batch_shape;
        final_shape.push(m);
        final_shape.push(n);

        Self::new(result_data, final_shape)
    }
}

impl<T> InternalCpuTensor<T>
where
    T: Clone + Mul<Output = T>,
{
    pub(in crate::types::cpu_tensor) fn tensor_product_impl(
        &self,
        rhs: &InternalCpuTensor<T>,
    ) -> Result<InternalCpuTensor<T>, CausalTensorError> {
        if self.is_empty() || rhs.is_empty() {
            return Err(CausalTensorError::EmptyTensor);
        }

        // Calculate the new shape by concatenating the shapes of self and rhs.
        let mut result_shape = Vec::with_capacity(self.ndim() + rhs.ndim());
        result_shape.extend_from_slice(self.shape());
        result_shape.extend_from_slice(rhs.shape());

        let result_len = self.len() * rhs.len();
        let mut result_data = Vec::with_capacity(result_len);

        // Iterate through all elements of self and rhs, performing the multiplication.
        for self_val in self.data.iter() {
            for rhs_val in rhs.data.iter() {
                result_data.push(self_val.clone() * rhs_val.clone());
            }
        }

        InternalCpuTensor::new(result_data, result_shape)
    }
}
