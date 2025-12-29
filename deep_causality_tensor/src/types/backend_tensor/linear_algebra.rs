/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Linear algebra operations for BackendTensor.

use super::BackendTensor;
use crate::traits::{LinearAlgebraBackend, TensorBackend, TensorData};
use core::iter::Sum;
use deep_causality_num::{RealField, Ring};

impl<T, B> BackendTensor<T, B>
where
    T: TensorData,
    B: TensorBackend + LinearAlgebraBackend,
{
    /// Matrix multiplication.
    ///
    /// For 2D tensors A (m×k) and B (k×n), computes C = A @ B resulting in shape (m×n).
    pub fn matmul(&self, rhs: &Self) -> Self
    where
        T: Ring + Default + PartialOrd,
    {
        Self::from_inner(B::matmul(&self.inner, &rhs.inner))
    }

    /// QR decomposition.
    ///
    /// Returns (Q, R) where Q is orthogonal and R is upper triangular.
    pub fn qr(&self) -> (Self, Self)
    where
        T: RealField + Sum + PartialEq,
    {
        let (q, r) = B::qr(&self.inner);
        (Self::from_inner(q), Self::from_inner(r))
    }

    /// Singular Value Decomposition.
    ///
    /// Returns (U, S, Vt) where A = U @ diag(S) @ Vt.
    pub fn svd(&self) -> (Self, Self, Self)
    where
        T: RealField + Sum + PartialEq,
    {
        let (u, s, vt) = B::svd(&self.inner);
        (
            Self::from_inner(u),
            Self::from_inner(s),
            Self::from_inner(vt),
        )
    }

    /// Matrix inverse.
    ///
    /// Returns A⁻¹ such that A @ A⁻¹ = I.
    pub fn inverse(&self) -> Self
    where
        T: RealField + Sum + PartialEq,
    {
        Self::from_inner(B::inverse(&self.inner))
    }
}
