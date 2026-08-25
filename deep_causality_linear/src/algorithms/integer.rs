/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exact linear algebra over the integers, which never leaves ℤ.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use deep_causality_algebra::EuclideanDomain;
use deep_causality_num::{One, Zero};

/// The determinant of an integer matrix, by fraction-free (Bareiss) elimination.
///
/// # Why not Gaussian elimination
///
/// The determinant of an integer matrix is an integer, but Gaussian elimination's intermediates are
/// not: it divides by its pivot and leaves ℤ on the first step. Bareiss keeps every intermediate in
/// the ring and reaches the answer in cubic time, where Laplace expansion takes factorial time.
///
/// # What the bound supplies, and what makes it correct
///
/// [`EuclideanDomain`] supplies the operation: `div_euclid` performs the divisions. What makes those
/// divisions *exact* is one rung lower — `EuclideanDomain: IntegralDomain`, and an integral domain
/// has no zero divisors, which is what licenses cancellation. Bareiss rests on cancellation, not on
/// a Euclidean valuation; the valuation is merely how the division is spelled.
///
/// Before `IntegralDomain` existed in the tower, this bound carried only `CommutativeRing` and the
/// exactness claim rested on nothing the type system held.
///
/// # No float appears
///
/// Not at any point, including intermediates. This is the operation
/// `deep_causality_topology` needs and reaches today by densifying `CsrMatrix<i8>` to `f64` and
/// running an SVD.
///
/// # The bound is checked, not merely stated
///
/// Widening this from `EuclideanDomain` to `CommutativeRing` would compile — the body's divisions
/// would simply stop being exact — and no behavioural test over ℤ would notice, because ℤ satisfies
/// both. What notices is a scalar that satisfies the wider bound and not this one:
///
/// ```compile_fail,E0277
/// use deep_causality_linear::{DenseMatrix, determinant_exact};
///
/// // f64 is a CommutativeRing and is not a EuclideanDomain in this tower. The fraction-free path
/// // must refuse it; if the bound is widened, this starts compiling and the doctest fails.
/// let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 0.0, 1.0], 2, 2).unwrap();
/// let _ = determinant_exact(&m);
/// ```
///
/// # Errors
///
/// [`LinearError::NotSquare`] if the matrix is not square.
pub fn determinant_exact<M>(m: &M) -> Result<M::Scalar, LinearError>
where
    M: MatrixView,
    M::Scalar: EuclideanDomain,
{
    let (rows, cols) = (m.rows(), m.cols());
    if rows != cols {
        return Err(LinearError::NotSquare((rows, cols)));
    }
    let n = rows;
    if n == 0 {
        return Ok(M::Scalar::one());
    }
    let mut a = alloc::vec::Vec::with_capacity(n * n);
    for i in 0..n {
        for j in 0..n {
            a.push(m.get(i, j)?);
        }
    }

    // Bareiss. Every division below is exact, guaranteed by the integral-domain structure that
    // `EuclideanDomain` sits above: no zero divisors means cancellation holds.
    let mut prev = M::Scalar::one();
    let mut sign_negative = false;
    for k in 0..(n - 1) {
        if a[k * n + k].is_zero() {
            // Pivot by search, as everywhere else here.
            let Some(p) = ((k + 1)..n).find(|&r| !a[r * n + k].is_zero()) else {
                return Ok(M::Scalar::zero());
            };
            for j in 0..n {
                a.swap(k * n + j, p * n + j);
            }
            sign_negative = !sign_negative;
        }
        for i in (k + 1)..n {
            for j in (k + 1)..n {
                const OP: &str = "fraction-free determinant";
                let lhs = mul_or_overflow(&a[i * n + j], &a[k * n + k], OP)?;
                let rhs = mul_or_overflow(&a[i * n + k], &a[k * n + j], OP)?;
                let num = sub_or_overflow(&lhs, &rhs, OP)?;
                a[i * n + j] = num.div_euclid(&prev);
            }
        }
        prev = a[k * n + k].clone();
    }
    let d = a[(n - 1) * n + (n - 1)].clone();
    Ok(if sign_negative {
        M::Scalar::zero() - d
    } else {
        d
    })
}

/// Multiplies, reporting an overflow rather than wrapping or panicking.
///
/// ℤ is unbounded and the scalar is not. `EuclideanDomain::checked_mul` is what makes this
/// detectable: checking the product afterwards is too late, because a fixed-width multiply panics
/// on overflow in debug builds and wraps in release, so an after-the-fact check runs after the
/// panic in one profile and against a wrapped value in the other.
fn mul_or_overflow<T: EuclideanDomain>(a: &T, b: &T, op: &'static str) -> Result<T, LinearError> {
    a.checked_mul(b).ok_or(LinearError::Overflow(op))
}

/// Subtracts, reporting an overflow rather than wrapping or panicking.
fn sub_or_overflow<T: EuclideanDomain>(a: &T, b: &T, op: &'static str) -> Result<T, LinearError> {
    a.checked_sub(b).ok_or(LinearError::Overflow(op))
}

/// The rank of an integer matrix, exactly.
///
/// # No tolerance, in the signature or the body
///
/// The rank of an integer matrix is an exact question with an exact answer. This function takes no
/// epsilon and applies none. `deep_causality_topology` currently obtains the same number by
/// densifying to `f64`, running an SVD, and counting singular values above `1e-5`, and every Betti
/// number it reports depends on that threshold.
///
/// # This is the characteristic-zero rank
///
/// Rank over ℤ equals rank over ℚ — rank is a fraction-field notion — so this computes the same
/// number that exact rational elimination would, without leaving ℤ. The integer path exists because
/// the rational route suffers coefficient growth that overflows a machine integer well before the
/// matrix gets large.
///
/// It is **not** the rank over 𝔽₂, which is a different number. See
/// [`rank_gf2`](crate::algorithms::gf2::rank_gf2).
///
/// ```compile_fail,E0277
/// use deep_causality_linear::{DenseMatrix, rank_exact};
///
/// // As for the determinant: f64 must be refused, or the "exact" in the name is false.
/// let m: DenseMatrix<f64> = DenseMatrix::from_vec(vec![1.0, 0.0, 0.0, 1.0], 2, 2).unwrap();
/// let _ = rank_exact(&m);
/// ```
pub fn rank_exact<M>(m: &M) -> Result<usize, LinearError>
where
    M: MatrixView,
    M::Scalar: EuclideanDomain + PartialEq,
{
    let (rows, cols) = (m.rows(), m.cols());
    if rows == 0 || cols == 0 {
        return Ok(0);
    }
    let mut a = alloc::vec::Vec::with_capacity(rows * cols);
    for i in 0..rows {
        for j in 0..cols {
            a.push(m.get(i, j)?);
        }
    }

    // Each row is reduced by its content — the gcd of its entries — before elimination starts.
    //
    // This is not an optimisation. Rank is scale-invariant, so dividing a row through by a common
    // factor cannot change the answer; but the fraction-free intermediates are products of entries,
    // and a matrix of large entries overflows on products whose *difference* is zero. Reducing
    // `[[i64::MAX, i64::MAX], [i64::MAX, i64::MAX]]` to `[[1, 1], [1, 1]]` first makes the
    // computation deserve the invariance the mathematics already has.
    for i in 0..rows {
        let mut content = M::Scalar::zero();
        for j in 0..cols {
            content = content.gcd(&a[i * cols + j]);
        }
        if !content.is_zero() && content != M::Scalar::one() {
            for j in 0..cols {
                a[i * cols + j] = a[i * cols + j].div_euclid(&content);
            }
        }
    }

    // Fraction-free forward elimination, counting pivots. No threshold: over ℤ an entry is zero or
    // it is not, and the answer is exact.
    let mut prev = M::Scalar::one();
    let mut row = 0usize;
    let mut rank = 0usize;
    for col in 0..cols {
        if row >= rows {
            break;
        }
        let Some(p) = (row..rows).find(|&r| !a[r * cols + col].is_zero()) else {
            continue;
        };
        if p != row {
            for j in 0..cols {
                a.swap(row * cols + j, p * cols + j);
            }
        }
        for i in (row + 1)..rows {
            for j in (col + 1)..cols {
                const OP: &str = "exact integer rank";
                let lhs = mul_or_overflow(&a[i * cols + j], &a[row * cols + col], OP)?;
                let rhs = mul_or_overflow(&a[i * cols + col], &a[row * cols + j], OP)?;
                let num = sub_or_overflow(&lhs, &rhs, OP)?;
                a[i * cols + j] = num.div_euclid(&prev);
            }
            a[i * cols + col] = M::Scalar::zero();
        }
        prev = a[row * cols + col].clone();
        row += 1;
        rank += 1;
    }
    Ok(rank)
}
