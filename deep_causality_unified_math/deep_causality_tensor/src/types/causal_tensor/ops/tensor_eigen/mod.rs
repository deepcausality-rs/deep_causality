/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;

use crate::CausalTensorError;
use crate::types::causal_tensor::CausalTensor;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_linear::DenseMatrix;

/// Eigendecomposition of a **Hermitian** row-major `n×n` matrix by cyclic Jacobi rotations
/// (`Uᴴ A U`). Returns `(eigenvalues, V)` where the columns of the row-major `n×n` `V` are the
/// eigenvectors: `A = V diag(λ) Vᴴ`. Eigenvalues are real (returned as `T`) and unsorted. For a real
/// scalar the phase `ρ = ±1` and this reduces to the ordinary real-symmetric Jacobi.
///
/// # Reference
/// G. H. Golub and C. F. Van Loan, *Matrix Computations*, 4th ed. (Johns Hopkins Univ. Press,
/// 2013), §8.5 (Jacobi methods).
pub(crate) fn sym_eig<T: ConjugateScalar>(mat: &[T], n: usize) -> (Vec<T>, Vec<T>) {
    // One copy of the algorithm, and it is not this one. The DMRG local solver
    // (`causal_tensor_network::solve::local`) needs the kernel at slice level rather than through
    // `CausalTensor`, so this stays as the shape that call site wants — but the numerics come from
    // `deep_causality_linear`, which is where the rest of the workspace reads them from.
    let m = DenseMatrix::from_vec(mat.to_vec(), n, n)
        .expect("n * n entries in an n x n shape, by construction");
    let (vals, vecs) =
        deep_causality_linear::eigen_hermitian(&m).expect("square, so the only error cannot arise");
    (vals.as_slice().to_vec(), vecs.as_slice().to_vec())
}

impl<T> CausalTensor<T>
where
    T: ConjugateScalar,
{
    /// Dense eigendecomposition of a **Hermitian** (real: symmetric) `n×n` matrix by cyclic
    /// Jacobi rotations, for real, dual, and complex scalars alike.
    ///
    /// Returns `(eigenvalues, V)` where the eigenvalues are real (carried as `T`, unsorted) and
    /// the columns of the `n×n` tensor `V` are the corresponding orthonormal eigenvectors, so
    /// `A = V · diag(λ) · Vᴴ`.
    ///
    /// The input is **assumed** (numerically) Hermitian: only the strict upper triangle and the
    /// real part of the diagonal are read, so a non-Hermitian input yields an unspecified
    /// decomposition (it is not silently symmetrized). Callers that need the guarantee should
    /// validate `A == Aᴴ` first.
    ///
    /// # Reference
    /// G. H. Golub and C. F. Van Loan, *Matrix Computations*, 4th ed. (Johns Hopkins Univ.
    /// Press, 2013), §8.5 (Jacobi methods).
    ///
    /// # Errors
    /// Returns [`CausalTensorError::DimensionMismatch`] if `self` is not 2-dimensional,
    /// [`CausalTensorError::ShapeMismatch`] if it is not square, or
    /// [`CausalTensorError::EmptyTensor`] if either dimension is zero.
    pub fn eigen_hermitian(&self) -> Result<(Vec<T>, Self), CausalTensorError> {
        if self.shape().len() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        if self.shape()[0] == 0 || self.shape()[1] == 0 {
            return Err(CausalTensorError::EmptyTensor);
        }
        let n = self.shape()[0];
        if n != self.shape()[1] {
            return Err(CausalTensorError::ShapeMismatch);
        }
        // The guards above stay here: `deep_causality_linear` has no notion of tensor rank, so
        // `DimensionMismatch` and `EmptyTensor` have no source there. What it does have is the
        // kernel, which this crate no longer carries a second copy of.
        let (vals, vecs) = deep_causality_linear::eigen_hermitian(self)
            .map_err(|_| CausalTensorError::ShapeMismatch)?;
        let v = CausalTensor::new(vecs.as_slice().to_vec(), vec![n, n])?;
        Ok((vals.as_slice().to_vec(), v))
    }
}
