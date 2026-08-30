/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The matrix inverse, delegating the numerics to `deep_causality_linear`.

use crate::{CausalTensor, CausalTensorError};
use alloc::vec;
use deep_causality_algebra::RealField;
use deep_causality_linear::DenseMatrix;
use deep_causality_num::{FromPrimitive, One, Zero};

impl<T> CausalTensor<T> {
    /// The inverse of a square matrix.
    ///
    /// # What changed when this became a delegation
    ///
    /// The body here was Gauss-Jordan on an augmented `n × 2n` matrix. It is now LU with partial
    /// pivoting in `deep_causality_linear`, solved against each basis column. Both are exact for a
    /// well-conditioned input; the LU route allocates `n × n` where the augmented one allocated
    /// `n × 2n`, and it shares its factorisation with the crate's `solve`.
    ///
    /// A caller computing `A⁻¹ b` should use `solve` instead — forming the inverse to multiply by
    /// it is both slower and less accurate than solving directly.
    ///
    /// # Errors
    ///
    /// [`CausalTensorError::DimensionMismatch`] if the tensor is not 2-dimensional,
    /// [`CausalTensorError::ShapeMismatch`] if it is not square, and
    /// [`CausalTensorError::SingularMatrix`] if it has no inverse.
    pub(in crate::types::causal_tensor) fn inverse_impl(&self) -> Result<Self, CausalTensorError>
    where
        T: Clone + RealField + Zero + One + Copy + PartialEq + FromPrimitive,
    {
        // The rank guard stays here: `deep_causality_linear` has no notion of tensor rank.
        if self.num_dim() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let n = self.shape()[0];
        if n != self.shape()[1] {
            return Err(CausalTensorError::ShapeMismatch);
        }
        if n == 0 {
            return Self::new(vec![], vec![0, 0]);
        }

        // `inverse` returns the representation it was given and builds it through `MatrixBuild`,
        // which a tensor does not implement — in-place construction by position has no meaning at
        // rank three. The dense matrix is that representation for the length of the call.
        let a = DenseMatrix::from_vec(self.as_slice().to_vec(), n, n)
            .map_err(|_| CausalTensorError::ShapeMismatch)?;
        let inv =
            deep_causality_linear::inverse(&a).map_err(|_| CausalTensorError::SingularMatrix)?;

        Self::new(inv.as_slice().to_vec(), vec![n, n])
    }
}
