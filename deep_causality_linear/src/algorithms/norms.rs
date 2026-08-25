/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Norms.
//!
//! # Two shapes, one body each
//!
//! The matrix norms are generic over any [`MatrixView`](crate::MatrixView), so they apply to the
//! sparse and the packed representations too. The vector norms are generic over a **slice**, and
//! [`DenseVector`](crate::DenseVector)'s methods of the same names delegate to them.
//!
//! The slice form exists because the callers being consolidated do not hold a `DenseVector`.
//! `deep_causality_multivector`'s coefficients are a `Vec<T>` and `CausalMultiField`'s are a
//! tensor's buffer; routing either through a `DenseVector` would allocate and copy the whole
//! coefficient vector on every norm. A slice is what both already have, and the method delegating
//! to the function is what keeps this one body rather than two.
//!
//! # Defined once
//!
//! The workspace answers the Euclidean-norm question four times today: `CausalTensor::norm_l2` and
//! `norm_sq`, `frobenius_norm` in `deep_causality_quantum`, `MultiVectorL2Norm::norm_l2` and
//! `CausalMultiField::squared_magnitude` in `deep_causality_multivector`. Each is correct where it
//! sits; together they are four places to fix a bug.
//!
//! Bounded on [`NormedScalar`] because `modulus_squared` lands in an ordered real, which is what
//! makes the complex case work without a second surface.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use deep_causality_algebra::{Normed, NormedScalar, Real};
use deep_causality_num::Zero;

/// The 1-norm of a vector, `Σ |aᵢ|`.
pub fn vector_norm_l1<T: NormedScalar>(v: &[T]) -> <T as Normed>::Real {
    let mut acc = <T as Normed>::Real::zero();
    for x in v {
        acc += x.modulus_squared().sqrt();
    }
    acc
}

/// The squared 2-norm of a vector, `Σ |aᵢ|²`.
///
/// Separate from [`vector_norm_l2`] because the square root is the expensive part and comparisons
/// rarely need it. `deep_causality_multivector`'s `CausalMultiField::squared_magnitude` is exactly
/// this quantity and now is exactly this call.
pub fn vector_norm_sq<T: NormedScalar>(v: &[T]) -> <T as Normed>::Real {
    let mut acc = <T as Normed>::Real::zero();
    for x in v {
        acc += x.modulus_squared();
    }
    acc
}

/// The 2-norm of a vector, `sqrt(Σ |aᵢ|²)`.
///
/// Uses `modulus_squared`, so the complex case is right without a separate surface. This is the
/// one definition of the Euclidean norm in the workspace.
pub fn vector_norm_l2<T: NormedScalar>(v: &[T]) -> <T as Normed>::Real {
    vector_norm_sq(v).sqrt()
}

/// The ∞-norm of a vector, `max |aᵢ|`.
///
/// Zero for the empty vector, which is the supremum over an empty set in the convention this crate
/// uses, and never `NaN`.
pub fn vector_norm_inf<T: NormedScalar>(v: &[T]) -> <T as Normed>::Real {
    let mut best = <T as Normed>::Real::zero();
    for x in v {
        let m = x.modulus_squared().sqrt();
        if m > best {
            best = m;
        }
    }
    best
}

/// The 1-norm: the largest column sum of moduli.
pub fn matrix_norm_l1<M>(m: &M) -> Result<<M::Scalar as Normed>::Real, LinearError>
where
    M: MatrixView,
    M::Scalar: NormedScalar,
{
    use deep_causality_algebra::Real;
    let (r, c) = (m.rows(), m.cols());
    let mut best = <M::Scalar as Normed>::Real::zero();
    for j in 0..c {
        let mut col = <M::Scalar as Normed>::Real::zero();
        for i in 0..r {
            col += m.get(i, j)?.modulus_squared().sqrt();
        }
        if col > best {
            best = col;
        }
    }
    Ok(best)
}

/// The ∞-norm: the largest row sum of moduli.
pub fn matrix_norm_inf<M>(m: &M) -> Result<<M::Scalar as Normed>::Real, LinearError>
where
    M: MatrixView,
    M::Scalar: NormedScalar,
{
    use deep_causality_algebra::Real;
    let (r, c) = (m.rows(), m.cols());
    let mut best = <M::Scalar as Normed>::Real::zero();
    for i in 0..r {
        let mut row = <M::Scalar as Normed>::Real::zero();
        for j in 0..c {
            row += m.get(i, j)?.modulus_squared().sqrt();
        }
        if row > best {
            best = row;
        }
    }
    Ok(best)
}

/// The Frobenius norm: `sqrt(Σ |aᵢⱼ|²)`.
///
/// Equal to the 2-norm of the entries read as one vector, which is a test rather than a remark.
pub fn matrix_norm_frobenius<M>(m: &M) -> Result<<M::Scalar as Normed>::Real, LinearError>
where
    M: MatrixView,
    M::Scalar: NormedScalar,
{
    use deep_causality_algebra::Real;
    let (r, c) = (m.rows(), m.cols());
    let mut acc = <M::Scalar as Normed>::Real::zero();
    for i in 0..r {
        for j in 0..c {
            acc += m.get(i, j)?.modulus_squared();
        }
    }
    Ok(acc.sqrt())
}
