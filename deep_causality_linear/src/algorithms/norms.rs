/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Matrix norms.
//!
//! The vector norms live on [`DenseVector`](crate::DenseVector), because they are properties of a
//! vector. These are properties of a matrix and are generic over any
//! [`MatrixView`](crate::MatrixView), so they apply to the sparse and the packed representations
//! too.
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
use deep_causality_algebra::{Normed, NormedScalar};
use deep_causality_num::Zero;

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
