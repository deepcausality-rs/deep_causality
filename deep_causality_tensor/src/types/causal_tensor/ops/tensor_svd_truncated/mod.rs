/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::causal_tensor_network::truncation::Truncation;
use crate::{CausalTensor, CausalTensorError};
use deep_causality_num::Scalar;

impl<T> CausalTensor<T>
where
    T: Scalar,
{
    /// Robust truncated thin-SVD: `A ≈ U · diag(S) · Vt`, retaining only the rank selected by
    /// `trunc`.
    ///
    /// This is the tensor-network numerical foundation (Stage 0). It is an **addition** to the
    /// existing power-iteration `svd` and does not alter it. The decomposition is computed by
    /// **one-sided Jacobi rotations**, chosen over implicit-shift Golub–Kahan because it delivers
    /// orthonormal factors to high relative accuracy with a simple, branch-stable kernel — the
    /// property TT-SVD and rounding compound over many sweeps.
    ///
    /// For an `m × n` input it returns `(U, S, Vt)` where `U` is `m × k`, `S` is the length-`k`
    /// vector of singular values in non-increasing order, and `Vt` is `k × n`, with `k` the
    /// retained rank under `trunc`. `U` and `Vt` have orthonormal columns / rows to working
    /// precision.
    ///
    /// Magnitude comparisons run on the real scalar directly (real field); divisions are guarded by
    /// checked-nonzero pivots so the kernel stays valid when the bound later widens to `Normed`
    /// for complex scalars.
    ///
    /// # Errors
    /// Returns [`CausalTensorError::DimensionMismatch`] if `self` is not 2-dimensional, or
    /// [`CausalTensorError::EmptyTensor`] if either dimension is zero.
    pub fn svd_truncated(
        &self,
        trunc: &Truncation<T>,
    ) -> Result<(Self, Self, Self), CausalTensorError> {
        if self.shape().len() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let m = self.shape()[0];
        let n = self.shape()[1];
        if m == 0 || n == 0 {
            return Err(CausalTensorError::EmptyTensor);
        }

        // One-sided Jacobi requires rows ≥ cols. When the input is wide, decompose its transpose
        // and swap the roles of U and V on the way out.
        let transposed = m < n;
        let (rows, cols, work) = if transposed {
            (n, m, transpose(self.as_slice(), m, n))
        } else {
            (m, n, self.as_slice().to_vec())
        };

        // jacobi returns: left factor (rows × cols, columns are U·σ normalized to U), the singular
        // values (length cols, unsorted), and the right factor V (cols × cols).
        let (u_full, sigma, v_full) = jacobi_svd::<T>(work, rows, cols);

        // Sort singular triplets by magnitude, descending.
        let mut order: Vec<usize> = (0..cols).collect();
        order.sort_by(|&a, &b| {
            sigma[b]
                .partial_cmp(&sigma[a])
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        let sorted: Vec<T> = order.iter().map(|&j| sigma[j]).collect();

        let k = trunc.retained_rank(&sorted);

        // Assemble the retained factors in the *original* orientation.
        // Non-transposed: U = u_full (rows×cols → m×k), V = v_full (cols×cols → n×k), Vt = Vᵀ.
        // Transposed:     the computed U,V swap roles for the original A.
        let (left, right, left_rows, right_rows) = if transposed {
            // For Aᵀ = U' S V'ᵀ we have A = V' S U'ᵀ ⇒ U = V', Vt = U'ᵀ.
            (&v_full, &u_full, cols, rows)
        } else {
            (&u_full, &v_full, rows, cols)
        };

        // U is left_rows × k, taking the selected columns.
        let mut u_data = vec![T::zero(); left_rows * k];
        for (col, &j) in order.iter().take(k).enumerate() {
            for row in 0..left_rows {
                u_data[row * k + col] = left[row * cols + j];
            }
        }
        // Vt is k × right_rows, each row r is the right singular vector for the r-th triplet.
        let mut vt_data = vec![T::zero(); k * right_rows];
        for (r, &j) in order.iter().take(k).enumerate() {
            for c in 0..right_rows {
                vt_data[r * right_rows + c] = right[c * cols + j];
            }
        }
        let s_data: Vec<T> = sorted.into_iter().take(k).collect();

        let u = CausalTensor::new(u_data, vec![left_rows, k])?;
        let s = CausalTensor::new(s_data, vec![k])?;
        let vt = CausalTensor::new(vt_data, vec![k, right_rows])?;
        Ok((u, s, vt))
    }
}

/// Transposes a row-major `rows × cols` buffer into `cols × rows`.
fn transpose<T: Scalar>(data: &[T], rows: usize, cols: usize) -> Vec<T> {
    let mut out = vec![T::zero(); rows * cols];
    for i in 0..rows {
        for j in 0..cols {
            out[j * rows + i] = data[i * cols + j];
        }
    }
    out
}

/// One-sided Jacobi SVD of a tall-or-square matrix (`rows ≥ cols`).
///
/// Returns `(u, sigma, v)` where `u` is `rows × cols` with orthonormal columns (for nonzero
/// singular values), `sigma` is the length-`cols` vector of singular values (unsorted), and `v` is
/// the `cols × cols` orthogonal matrix of right singular vectors. Columns are orthogonalized by
/// repeated Jacobi rotations; the accumulated rotations form `v`.
fn jacobi_svd<T>(mut u: Vec<T>, rows: usize, cols: usize) -> (Vec<T>, Vec<T>, Vec<T>)
where
    T: Scalar,
{
    let two = T::one() + T::one();
    // Convergence threshold: a small multiple of the working epsilon, so it scales with precision.
    let mut tol = T::epsilon();
    for _ in 0..6 {
        tol = tol + tol; // epsilon · 64
    }
    let max_sweeps = 60usize;

    // v starts as the identity (cols × cols).
    let mut v = vec![T::zero(); cols * cols];
    for i in 0..cols {
        v[i * cols + i] = T::one();
    }

    for _sweep in 0..max_sweeps {
        let mut max_off = T::zero();
        for p in 0..cols {
            for q in (p + 1)..cols {
                let mut alpha = T::zero();
                let mut beta = T::zero();
                let mut gamma = T::zero();
                for i in 0..rows {
                    let uip = u[i * cols + p];
                    let uiq = u[i * cols + q];
                    alpha += uip * uip;
                    beta += uiq * uiq;
                    gamma += uip * uiq;
                }
                let denom = (alpha * beta).sqrt();
                if denom > T::zero() {
                    let rel = gamma.abs() / denom;
                    if rel > max_off {
                        max_off = rel;
                    }
                    // Already orthogonal to working precision: skip the rotation.
                    if rel <= tol {
                        continue;
                    }
                } else {
                    // A degenerate (zero-norm) column pair contributes nothing to rotate.
                    continue;
                }

                // Jacobi rotation that annihilates the (p, q) inner product.
                let zeta = (beta - alpha) / (two * gamma);
                let sign = if zeta < T::zero() {
                    -T::one()
                } else {
                    T::one()
                };
                let t = sign / (zeta.abs() + (T::one() + zeta * zeta).sqrt());
                let c = T::one() / (T::one() + t * t).sqrt();
                let s = c * t;

                for i in 0..rows {
                    let uip = u[i * cols + p];
                    let uiq = u[i * cols + q];
                    u[i * cols + p] = c * uip - s * uiq;
                    u[i * cols + q] = s * uip + c * uiq;
                }
                for i in 0..cols {
                    let vip = v[i * cols + p];
                    let viq = v[i * cols + q];
                    v[i * cols + p] = c * vip - s * viq;
                    v[i * cols + q] = s * vip + c * viq;
                }
            }
        }
        if max_off <= tol {
            break;
        }
    }

    // Singular values are the column norms; normalize the columns of u into the left factor.
    let mut sigma = vec![T::zero(); cols];
    for j in 0..cols {
        let mut norm_sq = T::zero();
        for i in 0..rows {
            let x = u[i * cols + j];
            norm_sq += x * x;
        }
        let norm = norm_sq.sqrt();
        sigma[j] = norm;
        if norm > T::zero() {
            // Normalize via reciprocal-multiply so the kernel needs only `MulAssign`
            // (available on `Real`), keeping the bound at `Scalar` for the dual-number path.
            let inv_norm = T::one() / norm;
            for i in 0..rows {
                u[i * cols + j] *= inv_norm;
            }
        }
    }

    (u, sigma, v)
}
