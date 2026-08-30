/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, CausalTensorError};
use alloc::vec;
use core::iter::Sum;
use core::ops::Neg;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<T> CausalTensor<T>
where
    T: RealField + Sum + Neg<Output = T> + FromPrimitive,
{
    /// The singular value decomposition, delegating the numerics to `deep_causality_linear`.
    ///
    /// Returns `(U, S, Vᵀ)` with `U` of `m × k`, `S` of shape `[k]` and `Vᵀ` of `k × n`, where
    /// `k = min(m, n)`.
    ///
    /// # What changed when this became a delegation
    ///
    /// The body here was power iteration with deflation. It is now the one-sided Jacobi in
    /// `deep_causality_linear`, which converges for the repeated and clustered singular values that
    /// power iteration handles only to about `1e-8` — the identity has them in abundance. The
    /// factors satisfy the same contract and reconstruct the same input; individual entries differ
    /// where the old kernel had not converged.
    pub(crate) fn svd_impl(&self) -> Result<(Self, Self, Self), CausalTensorError> {
        if self.shape.len() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let m = self.shape()[0];
        let n = self.shape()[1];
        let k = m.min(n);

        // The rank guard stays here: `deep_causality_linear` has no notion of tensor rank, so
        // `DimensionMismatch` has no source there.
        let (u, s, vt) =
            deep_causality_linear::svd(self).map_err(|_| CausalTensorError::ShapeMismatch)?;

        Ok((
            CausalTensor::new(u.as_slice().to_vec(), vec![m, k])?,
            CausalTensor::new(s.as_slice().to_vec(), vec![k])?,
            CausalTensor::new(vt.as_slice().to_vec(), vec![k, n])?,
        ))
    }
}
