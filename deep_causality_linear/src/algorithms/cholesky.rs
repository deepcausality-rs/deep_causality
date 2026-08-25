/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Cholesky factorisation, and the least-squares solve built on it.
//!
//! # Why this is not in `solve`
//!
//! [`solve`](crate::algorithms::solve) factors a general square matrix by LU with partial pivoting.
//! Cholesky asks more of its input — Hermitian and positive definite — and gives more back: half the
//! arithmetic, no pivoting, and a factor whose diagonal being real and positive is itself the test
//! that the input qualified. Conflating them would mean either checking a property the general path
//! does not need or skipping a check this one depends on.
//!
//! # Positive-definiteness is discovered, not asserted
//!
//! There is no cheap test for positive-definiteness separate from the factorisation. The
//! factorisation *is* the test: the first diagonal entry whose radicand is non-positive is the
//! point at which the input is shown not to qualify, and [`LinearError::NotPositiveDefinite`]
//! carries that index.
//!
//! A matrix can be non-singular and still fail here — `diag(1, -1)` is invertible and indefinite —
//! so this is a distinct failure from [`LinearError::Singular`] and gets a distinct variant.
//!
//! # Hermitian, not merely symmetric
//!
//! Bounded on [`ConjugateScalar`], so a Hermitian complex matrix factors as `A = L Lᴴ`. Only the
//! lower triangle and the real part of the diagonal are read: a Hermitian matrix has a real
//! diagonal, and reading `A[i][i]` as complex would carry an imaginary part that the mathematics
//! says is zero into a square root that has no meaning for it.

use crate::algorithms::decomposition::flatten;
use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use crate::types::dense_matrix::DenseMatrix;
use crate::types::dense_vector::DenseVector;
use alloc::vec::Vec;
use deep_causality_algebra::{ConjugateScalar, Real};
use deep_causality_num::Zero;

/// The real type carrying magnitudes for a scalar.
type Re<T> = <T as ConjugateScalar>::Real;

/// The Cholesky factor `L` of a Hermitian positive-definite matrix, so that `A = L Lᴴ`.
///
/// Lower-triangular with a real positive diagonal. The strict upper triangle is exact zero.
///
/// # Errors
///
/// [`LinearError::NotSquare`] if the matrix is not square.
///
/// [`LinearError::NotPositiveDefinite`] at the first diagonal position whose radicand is
/// non-positive — the point at which the input is shown not to qualify.
pub fn cholesky<M>(m: &M) -> Result<DenseMatrix<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    let (rows, cols) = (m.rows(), m.cols());
    if rows != cols {
        return Err(LinearError::NotSquare((rows, cols)));
    }
    let n = rows;
    let a = flatten(m)?;
    let mut l = alloc::vec![M::Scalar::zero(); n * n];

    // Cholesky–Banachiewicz: row by row, each entry from the ones already written to its left.
    for i in 0..n {
        for j in 0..=i {
            // Σ_{k<j} L[i][k] · conj(L[j][k]).
            let mut sum = M::Scalar::zero();
            for k in 0..j {
                sum += l[i * n + k] * l[j * n + k].conjugate();
            }
            if i == j {
                // The diagonal of a Hermitian matrix is real, and so is the accumulated sum at this
                // position — taking the real part is reading what is there, not discarding.
                let radicand = a[i * n + i].real_part() - sum.real_part();
                if radicand <= Re::<M::Scalar>::zero() {
                    return Err(LinearError::NotPositiveDefinite(i));
                }
                l[i * n + j] = <M::Scalar as ConjugateScalar>::from_real(radicand.sqrt());
            } else {
                l[i * n + j] = (a[i * n + j] - sum) / l[j * n + j];
            }
        }
    }

    Ok(DenseMatrix::from_vec(l, n, n).expect("built from the shape"))
}

/// The least-squares solution of `A x ≈ b` by the normal equations, factored with Cholesky.
///
/// Returns the `x` minimising `‖A x − b‖₂` for an overdetermined `A` of `m × n` with `m ≥ n` and
/// full column rank.
///
/// # The normal equations square the condition number
///
/// `AᴴA` has the condition number of `A` squared, so this loses roughly half the available digits
/// against a QR-based least squares. It is the method the tensor crate's
/// `solve_least_squares_cholsky` has always used — the name says so — and this is where that body
/// now lives. A caller wanting the better-conditioned route should factor with [`qr`](crate::qr)
/// and back-substitute.
///
/// # Errors
///
/// [`LinearError::LengthMismatch`] if `b`'s length is not `A`'s row count.
///
/// [`LinearError::NotPositiveDefinite`] if `AᴴA` is not positive definite, which for a real `A`
/// means the columns are linearly dependent — a rank-deficient least-squares problem has no unique
/// solution, and this is where that shows up.
pub fn solve_least_squares<M>(
    a: &M,
    b: &DenseVector<M::Scalar>,
) -> Result<DenseVector<M::Scalar>, LinearError>
where
    M: MatrixView,
    M::Scalar: ConjugateScalar,
{
    let (rows, cols) = (a.rows(), a.cols());
    if b.len() != rows {
        return Err(LinearError::LengthMismatch(rows, b.len()));
    }
    let n = cols;
    let flat = flatten(a)?;
    let rhs = b.as_slice();

    // AᴴA, an n × n Hermitian matrix. Formed directly rather than by transposing and multiplying,
    // which would allocate the transpose for nothing.
    let mut ata = alloc::vec![M::Scalar::zero(); n * n];
    for i in 0..n {
        for j in 0..n {
            let mut acc = M::Scalar::zero();
            for k in 0..rows {
                acc += flat[k * n + i].conjugate() * flat[k * n + j];
            }
            ata[i * n + j] = acc;
        }
    }
    // Aᴴb.
    let mut atb: Vec<M::Scalar> = alloc::vec![M::Scalar::zero(); n];
    for (i, slot) in atb.iter_mut().enumerate() {
        let mut acc = M::Scalar::zero();
        for k in 0..rows {
            acc += flat[k * n + i].conjugate() * rhs[k];
        }
        *slot = acc;
    }

    let l = cholesky(&DenseMatrix::from_vec(ata, n, n).expect("built from the shape"))?;
    let lf = l.as_slice();

    // L z = Aᴴb by forward substitution.
    let mut z = alloc::vec![M::Scalar::zero(); n];
    for i in 0..n {
        let mut sum = M::Scalar::zero();
        for j in 0..i {
            sum += lf[i * n + j] * z[j];
        }
        z[i] = (atb[i] - sum) / lf[i * n + i];
    }
    // Lᴴ x = z by backward substitution. `Lᴴ[i][j]` is `conj(L[j][i])`, read from the factor rather
    // than from a transpose built to hold it.
    let mut x = alloc::vec![M::Scalar::zero(); n];
    for i in (0..n).rev() {
        let mut sum = M::Scalar::zero();
        for j in (i + 1)..n {
            sum += lf[j * n + i].conjugate() * x[j];
        }
        x[i] = (z[i] - sum) / lf[i * n + i].conjugate();
    }

    Ok(DenseVector::from_vec(x))
}
