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
use alloc::vec::Vec;
use deep_causality_algebra::{Real, RealField};
use deep_causality_num::{One, Zero};

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
    svd_impl(m, None)
}

/// One-sided Jacobi: rotate pairs of columns until they are orthogonal.
///
/// Chosen over power iteration because it converges for repeated and clustered singular values,
/// which the identity has in abundance and which the delegation baseline shows the existing
/// implementation handling only to about 1e-8.
fn svd_impl<M>(
    m: &M,
    keep: Option<&Truncation<M::Scalar>>,
) -> Result<SvdFactors<M::Scalar>, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: RealField,
{
    let (rows, cols) = (m.rows(), m.cols());
    if rows == 0 || cols == 0 {
        // The baseline decomposes the empty matrix rather than rejecting it.
        return Ok((
            DenseMatrix::from_vec(alloc::vec::Vec::new(), rows, rows).expect("empty"),
            DenseVector::from_vec(alloc::vec::Vec::new()),
            DenseMatrix::from_vec(alloc::vec::Vec::new(), cols, cols).expect("empty"),
        ));
    }

    // u holds the working columns, v the accumulated right rotations.
    let mut u = alloc::vec![M::Scalar::zero(); rows * cols];
    for i in 0..rows {
        for j in 0..cols {
            u[i * cols + j] = m.get(i, j)?;
        }
    }
    let mut v = alloc::vec![M::Scalar::zero(); cols * cols];
    for i in 0..cols {
        v[i * cols + i] = M::Scalar::one();
    }

    let two = M::Scalar::one() + M::Scalar::one();
    let eps = M::Scalar::epsilon();
    for _ in 0..60 {
        let mut off = M::Scalar::zero();
        for p in 0..cols {
            for q in (p + 1)..cols {
                let mut alpha = M::Scalar::zero();
                let mut beta = M::Scalar::zero();
                let mut gamma = M::Scalar::zero();
                for i in 0..rows {
                    let (a, b) = (u[i * cols + p], u[i * cols + q]);
                    alpha += a * a;
                    beta += b * b;
                    gamma += a * b;
                }
                if gamma.abs() <= eps * (alpha * beta).sqrt() {
                    continue;
                }
                off += gamma.abs();
                let zeta = (beta - alpha) / (two * gamma);
                let sign = if zeta >= M::Scalar::zero() {
                    M::Scalar::one()
                } else {
                    M::Scalar::zero() - M::Scalar::one()
                };
                let t = sign / (zeta.abs() + (M::Scalar::one() + zeta * zeta).sqrt());
                let c = M::Scalar::one() / (M::Scalar::one() + t * t).sqrt();
                let s = c * t;
                for i in 0..rows {
                    let (a, b) = (u[i * cols + p], u[i * cols + q]);
                    u[i * cols + p] = c * a - s * b;
                    u[i * cols + q] = s * a + c * b;
                }
                for i in 0..cols {
                    let (a, b) = (v[i * cols + p], v[i * cols + q]);
                    v[i * cols + p] = c * a - s * b;
                    v[i * cols + q] = s * a + c * b;
                }
            }
        }
        if off <= eps {
            break;
        }
    }

    // The singular values are the column norms; normalising the columns gives U.
    let mut order: Vec<(usize, M::Scalar)> = (0..cols)
        .map(|j| {
            let mut acc = M::Scalar::zero();
            for i in 0..rows {
                acc += u[i * cols + j] * u[i * cols + j];
            }
            (j, acc.sqrt())
        })
        .collect();
    order.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));

    let mut keep_count = order.len();
    if let Some(spec) = keep {
        keep_count = match spec {
            Truncation::Rank(k) => (*k).min(keep_count),
            Truncation::Tolerance(t) => order.iter().filter(|(_, s)| *s > *t).count(),
            Truncation::RankAndTolerance { rank, tolerance } => order
                .iter()
                .filter(|(_, s)| *s > *tolerance)
                .count()
                .min(*rank),
        };
    }

    let mut u_out = alloc::vec![M::Scalar::zero(); rows * keep_count];
    let mut s_out = alloc::vec::Vec::with_capacity(keep_count);
    let mut vt_out = alloc::vec![M::Scalar::zero(); keep_count * cols];
    for (k, &(j, sigma)) in order.iter().take(keep_count).enumerate() {
        s_out.push(sigma);
        for i in 0..rows {
            u_out[i * keep_count + k] = if sigma > eps {
                u[i * cols + j] / sigma
            } else {
                M::Scalar::zero()
            };
        }
        for i in 0..cols {
            vt_out[k * cols + i] = v[i * cols + j];
        }
    }

    Ok((
        DenseMatrix::from_vec(u_out, rows, keep_count).expect("built from the shape"),
        DenseVector::from_vec(s_out),
        DenseMatrix::from_vec(vt_out, keep_count, cols).expect("built from the shape"),
    ))
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
    Ok(svd(m)?.1)
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
    svd_impl(m, Some(spec))
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
    // Modified Gram-Schmidt: numerically better behaved than the classical form, and enough for
    // the sizes this workspace factorises.
    let (rows, cols) = (m.rows(), m.cols());
    let mut a = alloc::vec![M::Scalar::zero(); rows * cols];
    for i in 0..rows {
        for j in 0..cols {
            a[i * cols + j] = m.get(i, j)?;
        }
    }
    let mut q = alloc::vec![M::Scalar::zero(); rows * cols];
    let mut r = alloc::vec![M::Scalar::zero(); cols * cols];

    for k in 0..cols {
        let mut norm = M::Scalar::zero();
        for i in 0..rows {
            norm += a[i * cols + k] * a[i * cols + k];
        }
        let norm = norm.sqrt();
        r[k * cols + k] = norm;
        for i in 0..rows {
            q[i * cols + k] = if norm > M::Scalar::epsilon() {
                a[i * cols + k] / norm
            } else {
                M::Scalar::zero()
            };
        }
        for j in (k + 1)..cols {
            let mut dot = M::Scalar::zero();
            for i in 0..rows {
                dot += q[i * cols + k] * a[i * cols + j];
            }
            r[k * cols + j] = dot;
            for i in 0..rows {
                a[i * cols + j] -= dot * q[i * cols + k];
            }
        }
    }

    Ok((
        DenseMatrix::from_vec(q, rows, cols).expect("built from the shape"),
        DenseMatrix::from_vec(r, cols, cols).expect("built from the shape"),
    ))
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
    let (rows, cols) = (m.rows(), m.cols());
    if rows != cols {
        return Err(LinearError::NotSquare {
            shape: (rows, cols),
        });
    }
    let n = rows;
    let mut a = alloc::vec![M::Scalar::zero(); n * n];
    for i in 0..n {
        for j in 0..n {
            a[i * n + j] = m.get(i, j)?;
        }
    }
    let mut v = alloc::vec![M::Scalar::zero(); n * n];
    for i in 0..n {
        v[i * n + i] = M::Scalar::one();
    }

    // The cyclic Jacobi eigenvalue algorithm: zero the largest off-diagonal by rotation, repeat.
    // Symmetric by assumption, which is what `hermitian` in the name promises the caller has.
    let two = M::Scalar::one() + M::Scalar::one();
    let eps = M::Scalar::epsilon();
    for _ in 0..100 {
        let mut off = M::Scalar::zero();
        for p in 0..n {
            for q in (p + 1)..n {
                off += a[p * n + q] * a[p * n + q];
            }
        }
        if off.sqrt() <= eps {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                if a[p * n + q].abs() <= eps {
                    continue;
                }
                let theta = (a[q * n + q] - a[p * n + p]) / (two * a[p * n + q]);
                let sign = if theta >= M::Scalar::zero() {
                    M::Scalar::one()
                } else {
                    M::Scalar::zero() - M::Scalar::one()
                };
                let t = sign / (theta.abs() + (theta * theta + M::Scalar::one()).sqrt());
                let c = M::Scalar::one() / (t * t + M::Scalar::one()).sqrt();
                let s = t * c;
                for k in 0..n {
                    let (akp, akq) = (a[k * n + p], a[k * n + q]);
                    a[k * n + p] = c * akp - s * akq;
                    a[k * n + q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let (apk, aqk) = (a[p * n + k], a[q * n + k]);
                    a[p * n + k] = c * apk - s * aqk;
                    a[q * n + k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let (vkp, vkq) = (v[k * n + p], v[k * n + q]);
                    v[k * n + p] = c * vkp - s * vkq;
                    v[k * n + q] = s * vkp + c * vkq;
                }
            }
        }
    }

    let values: Vec<M::Scalar> = (0..n).map(|i| a[i * n + i]).collect();
    Ok((
        DenseVector::from_vec(values),
        DenseMatrix::from_vec(v, n, n).expect("built from the shape"),
    ))
}
