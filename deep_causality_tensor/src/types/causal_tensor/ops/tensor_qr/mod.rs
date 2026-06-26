/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalTensor, CausalTensorError};
use deep_causality_num::Scalar;

impl<T> CausalTensor<T>
where
    T: Scalar,
{
    /// Thin Householder QR: `A = Q · R` with `Q` (`m × k`) orthonormal columns and `R` (`k × n`)
    /// upper-triangular, where `k = min(m, n)`.
    ///
    /// This is the canonicalization primitive for tensor trains (Stage 0): QR — not SVD — is the
    /// standard, cheaper gauge sweep. The reflectors are applied in place to a working copy of `A`
    /// to form `R`, and accumulated into `Q`.
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
        let two = T::one() + T::one();

        // Working copy of A (becomes R after the reflectors); Q accumulates as m × m.
        let mut r = self.as_slice().to_vec(); // m × n, row-major
        let mut q = vec![T::zero(); m * m];
        for i in 0..m {
            q[i * m + i] = T::one();
        }

        for j in 0..k {
            // Norm of the sub-column r[j..m, j].
            let mut norm_sq = T::zero();
            for i in j..m {
                let x = r[i * n + j];
                norm_sq += x * x;
            }
            let norm = norm_sq.sqrt();
            if norm <= T::zero() {
                continue;
            }
            // Sign chosen to avoid cancellation: alpha = -sign(r[j,j]) · ‖x‖.
            let alpha = if r[j * n + j] > T::zero() {
                -norm
            } else {
                norm
            };

            // Householder vector v = x - alpha·e_j, supported on rows j..m.
            let mut v = vec![T::zero(); m];
            for i in j..m {
                v[i] = r[i * n + j];
            }
            v[j] -= alpha;

            let mut v_norm_sq = T::zero();
            for &vi in v.iter().skip(j) {
                v_norm_sq += vi * vi;
            }
            if v_norm_sq <= T::zero() {
                continue;
            }

            // Apply H = I - 2 vvᵀ / (vᵀv) to the trailing columns of R.
            for col in j..n {
                let mut dot = T::zero();
                for i in j..m {
                    dot += v[i] * r[i * n + col];
                }
                let factor = two * dot / v_norm_sq;
                for i in j..m {
                    r[i * n + col] -= factor * v[i];
                }
            }
            // Accumulate Q ← Q · H (apply on the right, so rows of Q are transformed).
            for row in 0..m {
                let mut dot = T::zero();
                for i in j..m {
                    dot += q[row * m + i] * v[i];
                }
                let factor = two * dot / v_norm_sq;
                for i in j..m {
                    q[row * m + i] -= factor * v[i];
                }
            }
        }

        // Thin Q: first k columns of the m × m accumulator.
        let mut q_thin = vec![T::zero(); m * k];
        for i in 0..m {
            for c in 0..k {
                q_thin[i * k + c] = q[i * m + c];
            }
        }
        // Thin R: first k rows, with the strictly-lower triangle cleaned to exact zero.
        let mut r_thin = vec![T::zero(); k * n];
        for i in 0..k {
            for col in 0..n {
                r_thin[i * n + col] = if col < i { T::zero() } else { r[i * n + col] };
            }
        }

        let q_tensor = CausalTensor::new(q_thin, vec![m, k])?;
        let r_tensor = CausalTensor::new(r_thin, vec![k, n])?;
        Ok((q_tensor, r_tensor))
    }
}
