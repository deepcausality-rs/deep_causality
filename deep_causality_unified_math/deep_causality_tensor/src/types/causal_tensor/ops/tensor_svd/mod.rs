/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cholesky, and the least-squares solve built on it.
//!
//! Both bodies live in `deep_causality_linear` now. What stays here is the rank guard — that crate
//! has no notion of tensor rank, so `DimensionMismatch` has no source there — and the conversion
//! between a `CausalTensor` and the vector type the solver speaks.

use alloc::vec;

use crate::{CausalTensor, CausalTensorError};
use deep_causality_algebra::RealField;
use deep_causality_linear::DenseVector;
use deep_causality_num::{FromPrimitive, One, Zero};

impl<T: Default> CausalTensor<T> {
    /// The least-squares solution of `A x ≈ b` by the normal equations, factored with Cholesky.
    ///
    /// `a` is the `m × n` design matrix and `b` the `m × 1` observation column; the result is
    /// `n × 1`.
    ///
    /// # Errors
    ///
    /// [`CausalTensorError::DimensionMismatch`] if either input is not 2-dimensional,
    /// [`CausalTensorError::ShapeMismatch`] if `b` is not an `m × 1` column, and
    /// [`CausalTensorError::SingularMatrix`] if `AᴴA` is not positive definite — which for a real
    /// design matrix means its columns are linearly dependent and the problem has no unique answer.
    pub(in crate::types::causal_tensor) fn solve_least_squares_cholsky_impl(
        a: &Self,
        b: &Self,
    ) -> Result<Self, CausalTensorError>
    where
        T: Default + Clone + RealField + Zero + One + Copy + PartialEq + FromPrimitive,
    {
        if a.num_dim() != 2 || b.num_dim() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let m = a.shape()[0];
        let n = a.shape()[1];
        if b.shape()[0] != m || b.shape()[1] != 1 {
            return Err(CausalTensorError::ShapeMismatch);
        }

        let rhs = DenseVector::from_vec(b.as_slice().to_vec());
        let x = deep_causality_linear::solve_least_squares(a, &rhs)
            .map_err(|_| CausalTensorError::SingularMatrix)?;

        CausalTensor::new(x.as_slice().to_vec(), vec![n, 1])
    }

    /// The Cholesky factor `L` of a Hermitian positive-definite matrix, so that `A = L Lᴴ`.
    ///
    /// # Errors
    ///
    /// [`CausalTensorError::DimensionMismatch`] if the tensor is not 2-dimensional,
    /// [`CausalTensorError::ShapeMismatch`] if it is not square, and
    /// [`CausalTensorError::SingularMatrix`] if it is not positive definite.
    pub(in crate::types::causal_tensor) fn cholesky_decomposition_impl(
        &self,
    ) -> Result<Self, CausalTensorError>
    where
        T: Default + Clone + RealField + Zero + One + PartialEq + FromPrimitive,
    {
        if self.num_dim() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let n = self.shape()[0];
        if n != self.shape()[1] {
            return Err(CausalTensorError::ShapeMismatch);
        }

        let l =
            deep_causality_linear::cholesky(self).map_err(|_| CausalTensorError::SingularMatrix)?;
        CausalTensor::new(l.as_slice().to_vec(), vec![n, n])
    }
}
