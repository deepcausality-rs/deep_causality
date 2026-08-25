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
//! These are iterative numerical algorithms — Jacobi rotations, Householder reflections, one-sided
//! Jacobi — that compare magnitudes and take square roots. That excludes 𝔽₂, ℚ and ℤ, correctly:
//! none of them has a singular value decomposition in any sense this code computes.
//!
//! # Bounded on `ConjugateScalar`, because complex is not ordered
//!
//! Every entry point here is bounded on [`ConjugateScalar`], which spans real fields, dual numbers
//! and complex. Magnitudes and thresholds live in `T::Real` and only the rotations are injected
//! back into `T`, so a Hermitian complex matrix decomposes as readily as a real symmetric one — the
//! case a density matrix needs. `RealField` could never cover it: `Complex` is unordered.
//!
//! [`svd`], [`svd_sorted`], [`svd_truncated`] and [`singular_values`] are on the same bound: the
//! one-sided Jacobi splits each complex 2x2 sub-problem into a real angle and a phase, so it
//! orthogonalises columns under the Hermitian inner product without ever ordering a complex value.
//!
//! # Read-only, so a view is enough
//!
//! [`qr`] and [`eigen_hermitian`] take [`MatrixView`] rather than [`RowOps`]. They never mutate a
//! row — they copy the entries out and work on a flat buffer — so requiring the mutating trait
//! excluded every read-only representation for nothing. `CausalTensor` is the one that matters:
//! in-place row mutation has no meaning on a tensor of rank three, and it does not need one.

use crate::algorithms::kernels;
use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use crate::types::dense_matrix::DenseMatrix;
use crate::types::dense_vector::DenseVector;
use alloc::vec::Vec;
use deep_causality_algebra::ConjugateScalar;
use deep_causality_num::Zero;

/// Copies a view's entries into a flat row-major buffer.
///
/// The kernels work on a slice, so every entry is read once here rather than through the trait in
/// an inner loop. A representation holding its entries contiguously overrides
/// [`MatrixView::to_row_major`] and this becomes the copy.
pub(crate) fn flatten<M: MatrixView>(m: &M) -> Result<Vec<M::Scalar>, LinearError> {
    m.to_row_major()
}

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

/// The full singular value decomposition with **real** singular values, sorted descending.
///
/// `(U, sigma, Vᴴ)` with `U` of `m × k`, `sigma` of length `k` and `Vᴴ` of `k × n`, where
/// `k = min(m, n)`.
///
/// # Why the singular values come back real
///
/// They are real, for a complex matrix as much as a real one — `A = U Σ Vᴴ` with `Σ` real
/// non-negative diagonal is what the decomposition *is*. [`svd`] injects them back into the scalar
/// type to keep the shape `CausalTensor::svd` returns; this is the entry point for a caller that
/// wants the spectrum itself, and for one applying its own truncation policy to it.
pub type SvdReal<T> = (
    DenseMatrix<T>,
    Vec<<T as ConjugateScalar>::Real>,
    DenseMatrix<T>,
);

/// The singular value decomposition, as `(U, S, Vᵀ)`.
///
/// `U` and `Vᵀ` are matrices and `S` is a vector of the singular values, matching what
/// `CausalTensor::svd` returns — that method delegates here and must not change its return shape.
///
/// **Thin:** `U` is `m × k`, `S` has length `k` and `Vᵀ` is `k × n` with `k = min(m, n)`. A matrix
/// has `min(m, n)` singular values and no more; returning `n` of them for a wide matrix means
/// returning zeros dressed as a spectrum.
///
/// The singular values are real and are injected back into the scalar type here. A caller that
/// wants them as reals — to apply a tolerance, or to read a condition number — should use
/// [`svd_sorted`], which does not do the round trip.
///
/// # The empty matrix is decomposed, not rejected
///
/// A `0x0` input returns three empty factors rather than an error, because that is what the method
/// being replaced does.
pub fn svd<M>(m: &M) -> Result<SvdFactors<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    let (u, sigma, vt) = svd_sorted(m)?;
    let s = sigma
        .into_iter()
        .map(<M::Scalar as ConjugateScalar>::from_real)
        .collect();
    Ok((u, DenseVector::from_vec(s), vt))
}

/// The full decomposition, singular values real and sorted descending.
///
/// The shared body of [`svd`] and [`svd_truncated`], and the entry point a caller with its own
/// truncation policy applies that policy to: the factors come back at full rank `k = min(m, n)`
/// with the values sorted, so keeping the leading `k' ≤ k` of them is a slice.
pub fn svd_sorted<M>(m: &M) -> Result<SvdReal<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    let (rows, cols) = (m.rows(), m.cols());
    if rows == 0 || cols == 0 {
        // The baseline decomposes the empty matrix rather than rejecting it.
        return Ok((
            DenseMatrix::from_vec(Vec::new(), rows, 0).expect("empty"),
            Vec::new(),
            DenseMatrix::from_vec(Vec::new(), 0, cols).expect("empty"),
        ));
    }
    let flat = flatten(m)?;

    // One-sided Jacobi needs `rows >= cols`. A wide matrix is decomposed through its conjugate
    // transpose, and the roles of U and V swap on the way out: from `Aᴴ = U' Σ V'ᴴ` it follows that
    // `A = V' Σ U'ᴴ`, so `U = V'` and `Vᴴ = U'ᴴ`.
    let transposed = rows < cols;
    let (wr, wc, work) = if transposed {
        (cols, rows, kernels::conj_transpose(&flat, rows, cols))
    } else {
        (rows, cols, flat)
    };

    let (u_full, sigma, v_full) = kernels::jacobi_svd::<M::Scalar>(work, wr, wc);

    let mut order: Vec<usize> = (0..wc).collect();
    order.sort_by(|&a, &b| {
        sigma[b]
            .partial_cmp(&sigma[a])
            .unwrap_or(core::cmp::Ordering::Equal)
    });

    let k = wc; // = min(rows, cols) either way round
    let (left, right, left_rows, right_rows) = if transposed {
        (&v_full, &u_full, wc, wr)
    } else {
        (&u_full, &v_full, wr, wc)
    };

    let mut u_out = alloc::vec![M::Scalar::zero(); left_rows * k];
    for (col, &j) in order.iter().take(k).enumerate() {
        for row in 0..left_rows {
            u_out[row * k + col] = left[row * wc + j];
        }
    }
    // Row `r` of Vᴴ is the conjugate of the r-th right singular vector.
    let mut vt_out = alloc::vec![M::Scalar::zero(); k * right_rows];
    for (r, &j) in order.iter().take(k).enumerate() {
        for c in 0..right_rows {
            vt_out[r * right_rows + c] = right[c * wc + j].conjugate();
        }
    }
    let s_out: Vec<<M::Scalar as ConjugateScalar>::Real> =
        order.iter().take(k).map(|&j| sigma[j]).collect();

    Ok((
        DenseMatrix::from_vec(u_out, left_rows, k).expect("built from the shape"),
        s_out,
        DenseMatrix::from_vec(vt_out, k, right_rows).expect("built from the shape"),
    ))
}

/// The singular values alone, descending.
///
/// The `S` factor of [`svd`] without computing `U` and `Vᵀ`. Not part of the delegation contract —
/// no method on `CausalTensor` returns this — but the operation most callers actually want: a rank,
/// a condition number and a spectral norm each need the values and none of them needs the vectors.
pub fn singular_values<M>(m: &M) -> Result<DenseVector<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    Ok(svd(m)?.1)
}

/// The singular value decomposition truncated by a rank or a tolerance.
///
/// `CausalTensor::svd_truncated` takes a `Truncation<T::Real>` rather than a bare rank, and the
/// distinction is load-bearing: truncating at a fixed rank and truncating at a tolerance are
/// different requests, and the tensor-train code that calls this uses both.
///
/// The tolerance is in `T::Real`, because that is what a singular value is. A complex tolerance
/// would have no ordering to compare against.
pub fn svd_truncated<M>(
    m: &M,
    spec: &Truncation<<M::Scalar as ConjugateScalar>::Real>,
) -> Result<SvdFactors<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    let (u, sigma, vt) = svd_sorted(m)?;
    // The values arrive sorted descending, so every gate is a prefix length and truncation is a
    // slice rather than a selection.
    let keep = match spec {
        Truncation::Rank(k) => (*k).min(sigma.len()),
        Truncation::Tolerance(t) => sigma.iter().filter(|s| *s > t).count(),
        Truncation::RankAndTolerance { rank, tolerance } => {
            sigma.iter().filter(|s| *s > tolerance).count().min(*rank)
        }
    };

    let (rows, k_full) = u.shape();
    let (_, cols) = vt.shape();
    let mut u_out = alloc::vec![M::Scalar::zero(); rows * keep];
    for i in 0..rows {
        for j in 0..keep {
            u_out[i * keep + j] = u.as_slice()[i * k_full + j];
        }
    }
    let vt_out = vt.as_slice()[..keep * cols].to_vec();
    let s_out = sigma
        .into_iter()
        .take(keep)
        .map(<M::Scalar as ConjugateScalar>::from_real)
        .collect();

    Ok((
        DenseMatrix::from_vec(u_out, rows, keep).expect("built from the shape"),
        DenseVector::from_vec(s_out),
        DenseMatrix::from_vec(vt_out, keep, cols).expect("built from the shape"),
    ))
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
///
/// Thin: `Q` is `m×k` and `R` is `k×n` with `k = min(m, n)`. For a wide matrix that is narrower than
/// the input, because a `Q` with more columns than rows cannot have orthonormal columns.
///
/// Bounded on `ConjugateScalar` rather than `RealField`, so it admits complex and dual scalars as
/// well as real ones. See [`kernels`](crate::algorithms::kernels) for why that is the right bound
/// and what reduces to what for a real scalar.
///
/// # Errors
///
/// [`LinearError::IndexOutOfBounds`] if the view refuses a position inside its own shape.
pub fn qr<M>(m: &M) -> Result<QrFactors<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    let (rows, cols) = (m.rows(), m.cols());
    let a = flatten(m)?;
    let (q, r, k) = kernels::householder_qr(&a, rows, cols);
    Ok((
        DenseMatrix::from_vec(q, rows, k).expect("built from the shape"),
        DenseMatrix::from_vec(r, k, cols).expect("built from the shape"),
    ))
}

/// The eigendecomposition of a Hermitian matrix, as `(eigenvalues, eigenvectors)`.
///
/// The columns of the returned matrix are the eigenvectors, so `A = V diag(λ) Vᴴ`. The eigenvalues
/// are unsorted.
///
/// The eigenvalues come back as a [`DenseVector`] where `CausalTensor::eigen_hermitian` returns a
/// bare `Vec<T>`. The delegating method converts, which costs one allocation it already pays; the
/// vector type is what the rest of this crate speaks, and returning a bare `Vec` here would put the
/// tensor crate's choice into an API that has a vector of its own.
///
/// Bounded on `ConjugateScalar`, so a Hermitian complex matrix decomposes here as well as a real
/// symmetric one — which is the case `deep_causality_quantum` needs for a density matrix.
///
/// # Errors
///
/// [`LinearError::NotSquare`] if the matrix is not square.
pub fn eigen_hermitian<M>(m: &M) -> Result<EigenPair<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    let (rows, cols) = (m.rows(), m.cols());
    if rows != cols {
        return Err(LinearError::NotSquare((rows, cols)));
    }
    let a = flatten(m)?;
    let (values, v) = kernels::sym_eig(&a, rows);
    Ok((
        DenseVector::from_vec(values),
        DenseMatrix::from_vec(v, rows, rows).expect("built from the shape"),
    ))
}
