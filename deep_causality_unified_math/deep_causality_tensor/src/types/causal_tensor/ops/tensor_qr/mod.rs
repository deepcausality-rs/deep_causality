/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;

use crate::{CausalTensor, CausalTensorError};
use deep_causality_algebra::ConjugateScalar;

impl<T> CausalTensor<T>
where
    T: ConjugateScalar,
{
    /// Thin Householder QR: `A = Q · R` with `Q` (`m × k`) orthonormal columns and `R` (`k × n`)
    /// upper-triangular, where `k = min(m, n)`.
    ///
    /// This is the canonicalization primitive for tensor trains (Stage 0): QR — not SVD — is the
    /// standard, cheaper gauge sweep. The reflectors are applied in place to a working copy of `A`
    /// to form `R`, and accumulated into `Q`. For a complex scalar the reflectors are the genuine
    /// Householder reflectors `H = I − β v vᴴ` (conjugated inner products, unitary `Q`); for a real
    /// scalar the conjugation is the identity and it reduces to the ordinary real Householder QR.
    ///
    /// # Reference
    /// G. H. Golub and C. F. Van Loan, *Matrix Computations*, 4th ed. (Johns Hopkins Univ. Press,
    /// 2013), §5.2 (Householder QR factorization).
    ///
    /// # Errors
    /// Returns [`CausalTensorError::DimensionMismatch`] if `self` is not 2-dimensional, or
    /// [`CausalTensorError::EmptyTensor`] if either dimension is zero.
    pub fn qr(&self) -> Result<(Self, Self), CausalTensorError> {
        if self.shape().len() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let m = self.shape()[0];
        let n = self.shape()[1];
        if m == 0 || n == 0 {
            return Err(CausalTensorError::EmptyTensor);
        }
        let k = m.min(n);

        // The guards above stay here: `deep_causality_linear` has no notion of tensor rank, so
        // neither `DimensionMismatch` nor `EmptyTensor` has a source there. The reflectors do not
        // stay -- they live in `deep_causality_linear::algorithms::kernels` now, and this crate
        // reads them through the same read-trait every other representation uses.
        let (q_lin, r_lin) =
            deep_causality_linear::qr(self).map_err(|_| CausalTensorError::ShapeMismatch)?;
        let q_thin = q_lin.as_slice().to_vec();
        let r_thin = r_lin.as_slice().to_vec();

        let q_tensor = CausalTensor::new(q_thin, vec![m, k])?;
        let r_tensor = CausalTensor::new(r_thin, vec![k, n])?;
        Ok((q_tensor, r_tensor))
    }
}
