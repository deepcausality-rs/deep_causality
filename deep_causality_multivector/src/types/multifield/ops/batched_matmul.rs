/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Ring;
#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
use deep_causality_tensor::MlxBackend;
use deep_causality_tensor::{CpuBackend, LinearAlgebraBackend, TensorBackend, TensorData};

/// Trait to support batched matrix multiplication across backends.
///
/// `CpuBackend` does not support broadcasting in `matmul`, requiring an explicit loop.
/// `MlxBackend` supports broadcasting natively.
pub trait BatchedMatMul<T>: LinearAlgebraBackend
where
    T: TensorData + Ring + Default + PartialOrd,
{
    fn batched_matmul(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T>;
}

impl<T> BatchedMatMul<T> for CpuBackend
where
    T: TensorData + Ring + Default + PartialOrd,
{
    fn batched_matmul(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        // CPU Loop Implementation
        let shape = Self::shape(a);
        // Expect Rank >= 3: [Batch..., D, D]
        let rank = shape.len();
        if rank < 3 {
            // Fallback for purely 2D (though CausalMultiField implies Rank 5)
            return Self::matmul(a, b);
        }

        let d_rows = shape[rank - 2];
        let d_cols = shape[rank - 1];
        let batch_dims = &shape[0..rank - 2];
        let batch_size: usize = batch_dims.iter().product();

        // 2. Reshape to [Batch, D, D]
        // This flattens all batch dimensions into one
        let a_flat = Self::reshape(a, &[batch_size, d_rows, d_cols]);
        let b_flat = Self::reshape(b, &[batch_size, d_rows, d_cols]);

        // 3. Loop
        let mut results = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            // Slice [i..i+1, 0..D, 0..D]
            let range = [i..i + 1, 0..d_rows, 0..d_cols];
            let a_slice = Self::slice(&a_flat, &range);
            let b_slice = Self::slice(&b_flat, &range);

            // Reshape output of slice to [D, D]
            let a_mat = Self::reshape(&a_slice, &[d_rows, d_cols]);
            let b_mat = Self::reshape(&b_slice, &[d_rows, d_cols]);

            let c_mat = Self::matmul(&a_mat, &b_mat);
            results.push(c_mat);
        }

        // 4. Stack results back into a tensor
        let stacked = Self::stack(&results, 0).expect("Stack failed during batched matmul");

        // 5. Reshape back to original shape [Batch..., D, D]
        Self::reshape(&stacked, &shape)
    }
}

#[cfg(all(feature = "mlx", target_os = "macos", target_arch = "aarch64"))]
impl<T> BatchedMatMul<T> for MlxBackend
where
    T: TensorData + Ring + Default + PartialOrd,
{
    fn batched_matmul(a: &Self::Tensor<T>, b: &Self::Tensor<T>) -> Self::Tensor<T> {
        Self::matmul(a, b)
    }
}
