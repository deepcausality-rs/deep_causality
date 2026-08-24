/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Solving `Ax = b`, and the factorisation it is built from.

use crate::errors::linear_error::LinearError;
use crate::traits::row_ops::RowOps;
use crate::types::dense_vector::DenseVector;
use alloc::vec::Vec;
use deep_causality_algebra::NormedScalar;
use deep_causality_num::{One, Zero};

/// An LU factorisation with partial pivoting, kept so it can be applied more than once.
///
/// # Why this is a value rather than a step inside `solve`
///
/// Factorising costs `O(n³)` and each application costs `O(n²)`. A solve-only API makes a caller
/// with `k` right-hand sides pay the cubic cost `k` times. Both workloads in this workspace that
/// solve repeatedly — the Kalman filter in `deep_causality_physics` and the ridge fits in
/// `deep_causality_algorithms` — have the same matrix and many right-hand sides.
///
/// # It carries its permutation
///
/// Partial pivoting reorders rows. The permutation is part of the factorisation rather than
/// something the caller reconstructs, because applying `L` and `U` without it gives the solution to
/// a different system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lu<T> {
    factors: Vec<T>,
    order: usize,
    permutation: Vec<usize>,
    /// The sign the row swaps contribute to the determinant: `+1` for an even number, `-1` for odd.
    sign_is_negative: bool,
}

impl<T> Lu<T>
where
    T: NormedScalar,
{
    /// Factorises a square matrix.
    ///
    /// # Errors
    ///
    /// [`LinearError::NotSquare`] if the matrix is not square.
    ///
    /// [`LinearError::Singular`] if no pivot can be found in some column. The failure is reported
    /// here rather than at the first application, so that a caller who factorises once and applies
    /// many times learns about a singular matrix before the first solve rather than after it.
    pub fn factor<M>(m: &M) -> Result<Self, LinearError>
    where
        M: RowOps<Scalar = T> + Clone,
    {
        let (rows, cols) = (m.rows(), m.cols());
        if rows != cols {
            return Err(LinearError::NotSquare {
                shape: (rows, cols),
            });
        }
        let n = rows;
        let mut a = alloc::vec::Vec::with_capacity(n * n);
        for i in 0..n {
            for j in 0..n {
                a.push(m.get(i, j)?);
            }
        }
        let mut perm: alloc::vec::Vec<usize> = (0..n).collect();
        let mut negative = false;

        for col in 0..n {
            // Pivot by magnitude, searching at or below the current row. Never the diagonal on
            // faith: a Cayley-Menger matrix has a zero there and is not singular.
            let mut best = col;
            let mut best_mag = a[col * n + col].modulus_squared();
            for r in (col + 1)..n {
                let mag = a[r * n + col].modulus_squared();
                if mag > best_mag {
                    best = r;
                    best_mag = mag;
                }
            }
            if a[best * n + col].is_zero() {
                return Err(LinearError::Singular { at_column: col });
            }
            if best != col {
                for j in 0..n {
                    a.swap(col * n + j, best * n + j);
                }
                perm.swap(col, best);
                negative = !negative;
            }
            let head = a[col * n + col];
            for r in (col + 1)..n {
                let factor = a[r * n + col] / head;
                a[r * n + col] = factor;
                for j in (col + 1)..n {
                    let upd = a[r * n + j] - factor * a[col * n + j];
                    a[r * n + j] = upd;
                }
            }
        }

        Ok(Self {
            factors: a,
            order: n,
            permutation: perm,
            sign_is_negative: negative,
        })
    }

    /// Applies the factorisation to one right-hand side.
    ///
    /// # Errors
    ///
    /// [`LinearError::LengthMismatch`] if `b`'s length is not the matrix order.
    pub fn apply(&self, b: &DenseVector<T>) -> Result<DenseVector<T>, LinearError> {
        let n = self.order;
        if b.len() != n {
            return Err(LinearError::LengthMismatch {
                expected: n,
                found: b.len(),
            });
        }
        // Permute, then forward substitution through L (unit diagonal), then backward through U.
        let mut y = alloc::vec::Vec::with_capacity(n);
        for i in 0..n {
            let mut acc = b.get(self.permutation[i])?;
            for (j, yj) in y.iter().enumerate().take(i) {
                acc = acc - self.factors[i * n + j] * *yj;
            }
            y.push(acc);
        }
        let mut x = alloc::vec![T::zero(); n];
        for i in (0..n).rev() {
            let mut acc = y[i];
            for (j, xj) in x.iter().enumerate().skip(i + 1) {
                acc = acc - self.factors[i * n + j] * *xj;
            }
            x[i] = acc / self.factors[i * n + i];
        }
        Ok(DenseVector::from_vec(x))
    }

    /// The row permutation partial pivoting chose.
    pub fn permutation(&self) -> &[usize] {
        &self.permutation
    }

    /// The determinant, read off the factorisation.
    ///
    /// The product of `U`'s diagonal, negated when the row swaps were odd in number. Free once the
    /// factorisation exists, which is why a caller wanting both a solve and a determinant should
    /// factor once rather than call each.
    pub fn determinant(&self) -> T {
        let n = self.order;
        let mut d = T::one();
        for i in 0..n {
            d = d * self.factors[i * n + i];
        }
        if self.sign_is_negative {
            T::zero() - d
        } else {
            d
        }
    }
}

/// Solves `Ax = b` for a dense square system, by LU with partial pivoting.
///
/// # Prefer this to inverting
///
/// `deep_causality_physics` computes the Kalman gain as `K = P Hᵀ S⁻¹` by explicitly inverting `S`
/// and multiplying (`kernels/dynamics/estimation.rs:158-164`). Explicit inversion is both less
/// accurate and more work than a solve, and it is written that way because no solve existed to call.
///
/// # Errors
///
/// [`LinearError::NotSquare`], [`LinearError::LengthMismatch`], or [`LinearError::Singular`].
pub fn solve<M>(a: &M, b: &DenseVector<M::Scalar>) -> Result<DenseVector<M::Scalar>, LinearError>
where
    M: RowOps + Clone,
    M::Scalar: NormedScalar,
{
    if b.len() != a.rows() {
        return Err(LinearError::LengthMismatch {
            expected: a.rows(),
            found: b.len(),
        });
    }
    Lu::factor(a)?.apply(b)
}

/// Solves a lower-triangular system by forward substitution.
///
/// No factorisation: a triangular system is already factorised, and running an LU over it would be
/// quadratically wasteful and would lose the structure. This is also the operation the LU
/// applications are built from, and Cholesky and QR both produce triangular factors.
///
/// # Errors
///
/// [`LinearError::ZeroDiagonal`] if a diagonal entry is zero.
///
/// [`LinearError::WrongTriangle`] if a non-zero entry sits above the diagonal, naming the first one
/// found rather than ignoring it.
pub fn solve_lower<M>(
    a: &M,
    b: &DenseVector<M::Scalar>,
) -> Result<DenseVector<M::Scalar>, LinearError>
where
    M: RowOps,
    M::Scalar: NormedScalar,
{
    let n = a.rows();
    if n != a.cols() {
        return Err(LinearError::NotSquare {
            shape: (n, a.cols()),
        });
    }
    if b.len() != n {
        return Err(LinearError::LengthMismatch {
            expected: n,
            found: b.len(),
        });
    }
    // The wrong triangle is rejected rather than ignored: a non-zero where the caller promised a
    // zero means the caller has the wrong matrix, and silently dropping it answers a question that
    // was not asked.
    for i in 0..n {
        for j in 0..n {
            if j > i && !a.get(i, j)?.is_zero() {
                return Err(LinearError::WrongTriangle { at: (i, j) });
            }
        }
    }
    let mut x = alloc::vec![M::Scalar::zero(); n];
    for i in 0..n {
        let d = a.get(i, i)?;
        if d.is_zero() {
            return Err(LinearError::ZeroDiagonal { at_index: i });
        }
        let mut acc = b.get(i)?;
        for (j, xj) in x.iter().enumerate().take(i) {
            acc = acc - a.get(i, j)? * *xj;
        }
        x[i] = acc / d;
    }
    Ok(DenseVector::from_vec(x))
}

/// Solves an upper-triangular system by backward substitution.
///
/// # Errors
///
/// As [`solve_lower`], with the triangles exchanged.
pub fn solve_upper<M>(
    a: &M,
    b: &DenseVector<M::Scalar>,
) -> Result<DenseVector<M::Scalar>, LinearError>
where
    M: RowOps,
    M::Scalar: NormedScalar,
{
    let n = a.rows();
    if n != a.cols() {
        return Err(LinearError::NotSquare {
            shape: (n, a.cols()),
        });
    }
    if b.len() != n {
        return Err(LinearError::LengthMismatch {
            expected: n,
            found: b.len(),
        });
    }
    // The wrong triangle is rejected rather than ignored: a non-zero where the caller promised a
    // zero means the caller has the wrong matrix, and silently dropping it answers a question that
    // was not asked.
    for i in 0..n {
        for j in 0..n {
            if j < i && !a.get(i, j)?.is_zero() {
                return Err(LinearError::WrongTriangle { at: (i, j) });
            }
        }
    }
    let mut x = alloc::vec![M::Scalar::zero(); n];
    for i in (0..n).rev() {
        let d = a.get(i, i)?;
        if d.is_zero() {
            return Err(LinearError::ZeroDiagonal { at_index: i });
        }
        let mut acc = b.get(i)?;
        for (j, xj) in x.iter().enumerate().skip(i + 1) {
            acc = acc - a.get(i, j)? * *xj;
        }
        x[i] = acc / d;
    }
    Ok(DenseVector::from_vec(x))
}

/// The inverse.
///
/// # Use [`solve`] instead when the inverse is only wanted to multiply by
///
/// `A⁻¹b` computed as `solve(A, b)` is more accurate and cheaper than forming `A⁻¹` and multiplying.
/// The inverse is here for the cases that genuinely need the matrix — a covariance whose entries are
/// read individually, say — and this note is on it so that the next caller sees the alternative.
///
/// # Errors
///
/// [`LinearError::NotSquare`] or [`LinearError::Singular`].
pub fn inverse<M>(a: &M) -> Result<M, LinearError>
where
    M: RowOps + Clone + crate::traits::matrix_build::MatrixBuild,
    M::Scalar: NormedScalar,
{
    let n = a.rows();
    if n != a.cols() {
        return Err(LinearError::NotSquare {
            shape: (n, a.cols()),
        });
    }
    let lu = Lu::factor(a)?;
    let mut out = M::zeros(n, n);
    for j in 0..n {
        let mut e = alloc::vec![M::Scalar::zero(); n];
        e[j] = M::Scalar::one();
        let col = lu.apply(&DenseVector::from_vec(e))?;
        for i in 0..n {
            out.set(i, j, col.get(i)?)?;
        }
    }
    Ok(out)
}
