/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Batched matrix multiplication over a rank-3 tensor.
//!
//! # Decision: this stays on the tensor surface
//!
//! `add-linear-algebra-crate` marked three duplicated operations in this crate for replacement by
//! `deep_causality_linear`. Two — the vector 2-norm and the squared magnitude — were routed there.
//! This one was not, and the reason is the split the linear crate is defined by: **arity, not
//! density**. A two-index object is a matrix and belongs in `deep_causality_linear`; an N-index
//! object stays in `deep_causality_tensor`.
//!
//! What this does is reshape to `[batch, d, d]`, `slice` along axis 0, `stack` the results and
//! reshape back. Reshape, slice and stack are all N-index operations with no matrix meaning, and
//! they are the bulk of the body. The only two-index step is the inner `matmul`, which is already
//! `CausalTensor`'s own einsum path — `linear-crate-identity` keeps einsum in the tensor crate for
//! the same reason it keeps `broadcast` and `kronecker` there.
//!
//! Routing the inner call through `deep_causality_linear` would mean converting each rank-2 slice
//! to a `DenseMatrix` and back, once per batch element, to reach an operation the tensor crate
//! already has. That is a copy per batch element in exchange for no consolidation: the batching,
//! which is the whole content of this file, would still be here.
//!
//! The comparison worth stating is with the norms. `MultiVectorL2Norm::norm_l2` was a genuine
//! second definition of the Euclidean norm — the same expression, written again. This is not a
//! second definition of anything: `CausalTensor::matmul` is called, not reimplemented.

use alloc::vec::Vec;

use deep_causality_algebra::{Field, Ring};
use deep_causality_tensor::{CausalTensor, Tensor};

/// Trait to support batched matrix multiplication.
pub trait BatchedMatMul<T>
where
    T: Field + Ring + Copy + Default + PartialOrd,
{
    fn batched_matmul(&self, other: &Self) -> Self;
}

impl<T> BatchedMatMul<T> for CausalTensor<T>
where
    T: Field + Ring + Copy + Default + PartialOrd,
{
    fn batched_matmul(&self, other: &Self) -> Self {
        let shape = self.shape().to_vec();
        let rank = shape.len();

        if rank < 3 {
            // Fallback for purely 2D - use matmul method
            return self.matmul(other).expect("matmul failed in batched_matmul");
        }

        let d_rows = shape[rank - 2];
        let d_cols = shape[rank - 1];
        let batch_dims = &shape[0..rank - 2];
        let batch_size: usize = batch_dims.iter().product();

        // Reshape to [Batch, D, D]
        let a_flat = self
            .reshape(&[batch_size, d_rows, d_cols])
            .expect("reshape failed");
        let b_flat = other
            .reshape(&[batch_size, d_rows, d_cols])
            .expect("reshape failed");

        // Loop over batches
        let mut results = Vec::with_capacity(batch_size);
        for i in 0..batch_size {
            // Use slice(axis, index) method
            let a_slice = a_flat.slice(0, i).expect("slice failed");
            let b_slice = b_flat.slice(0, i).expect("slice failed");

            let c_mat = a_slice.matmul(&b_slice).expect("matmul failed");
            results.push(c_mat);
        }

        // Stack results back - stack takes &[Self]
        let stacked = CausalTensor::stack(&results, 0).expect("Stack failed during batched matmul");

        // Reshape back to original shape
        stacked.reshape(&shape).expect("reshape failed")
    }
}
