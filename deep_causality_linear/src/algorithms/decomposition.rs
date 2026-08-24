/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The six decompositions relocated from `deep_causality_tensor`.
//!
//! The bodies move here; `CausalTensor` keeps its methods and delegates to these, so its public
//! surface, its error type and its return shapes are unchanged and its eight in-workspace and seven
//! example dependents need no edit. That makes the relocation a patch-level change for the tensor
//! crate rather than a major one.
//!
//! Bounded on [`RealField`] throughout. These are iterative numerical algorithms — power iteration,
//! Jacobi rotations, Householder reflections — that compare magnitudes and take square roots, so
//! they need an ordered real. That excludes 𝔽₂, ℚ and ℤ, correctly: none of them has a singular
//! value decomposition in any sense this code computes.

use crate::errors::linear_error::LinearError;
use crate::traits::row_ops::RowOps;
use crate::types::dense_matrix::DenseMatrix;
use crate::types::dense_vector::DenseVector;
use deep_causality_algebra::RealField;

/// The three factors of a singular value decomposition: `U`, `S` and `Vᵀ`.
///
/// A named type rather than the tuple spelled out at three sites. `S` is a **vector**, matching what
/// `CausalTensor::svd` returns: its `S` factor has shape `[k]`, which the trait signature
/// `Result<(Self, Self, Self), _>` does not reveal — `Self` is a `CausalTensor` in all three
/// positions and a `CausalTensor` holds any rank. The baseline capture settled it.
pub type SvdFactors<T> = (DenseMatrix<T>, DenseVector<T>, DenseMatrix<T>);

/// The two factors of a QR decomposition.
pub type QrFactors<T> = (DenseMatrix<T>, DenseMatrix<T>);

/// The eigenvalues and eigenvectors of a Hermitian matrix.
pub type EigenPair<T> = (DenseVector<T>, DenseMatrix<T>);

/// The singular value decomposition, as `(U, S, Vᵀ)`.
///
/// `U` and `Vᵀ` are matrices and `S` is a vector of the singular values, matching what
/// `CausalTensor::svd` returns today — that method has to delegate here without changing its return
/// shape.
///
/// # The empty matrix is decomposed, not rejected
///
/// A `0x0` input returns three empty factors rather than an error, because that is what the method
/// being replaced does.
pub fn svd<M>(m: &M) -> Result<SvdFactors<M::Scalar>, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: RealField,
{
    let _ = m;
    todo!("svd")
}

/// The singular values alone, descending.
///
/// The `S` factor of [`svd`] without computing `U` and `Vᵀ`. Not part of the delegation contract —
/// no method on `CausalTensor` returns this — but the operation most callers actually want: a rank,
/// a condition number and a spectral norm each need the values and none of them needs the vectors.
pub fn singular_values<M>(m: &M) -> Result<DenseVector<M::Scalar>, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: RealField,
{
    let _ = m;
    todo!("singular_values")
}

/// The singular value decomposition truncated by a rank or a tolerance.
///
/// `CausalTensor::svd_truncated` takes a `Truncation<T::Real>` rather than a bare rank, and the
/// distinction is load-bearing: truncating at a fixed rank and truncating at a tolerance are
/// different requests, and the tensor-train code that calls this uses both.
pub fn svd_truncated<M>(
    m: &M,
    spec: &Truncation<M::Scalar>,
) -> Result<SvdFactors<M::Scalar>, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: RealField,
{
    let _ = (m, spec);
    todo!("svd_truncated")
}

/// How a truncated decomposition decides what to keep.
///
/// Mirrors `deep_causality_tensor::Truncation`, which this replaces the body of. Keeping the two
/// requests distinct is what stops a caller who means "at most rank k" from silently getting
/// "everything above epsilon".
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Truncation<R> {
    /// Keep at most this many components.
    Rank(usize),
    /// Keep every component whose singular value exceeds this.
    Tolerance(R),
    /// Both: at most `rank` components, and none below `tolerance`.
    RankAndTolerance { rank: usize, tolerance: R },
}

/// The QR decomposition, as `(Q, R)`.
pub fn qr<M>(m: &M) -> Result<QrFactors<M::Scalar>, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: RealField,
{
    let _ = m;
    todo!("qr")
}

/// The eigendecomposition of a Hermitian matrix, as `(eigenvalues, eigenvectors)`.
///
/// The eigenvalues come back as a [`DenseVector`] where `CausalTensor::eigen_hermitian` returns a
/// bare `Vec<T>`. The delegating method converts, which costs one allocation it already pays; the
/// vector type is what the rest of this crate speaks, and returning a bare `Vec` here would put the
/// tensor crate's choice into an API that has a vector of its own.
pub fn eigen_hermitian<M>(m: &M) -> Result<EigenPair<M::Scalar>, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: RealField,
{
    let _ = m;
    todo!("eigen_hermitian")
}
