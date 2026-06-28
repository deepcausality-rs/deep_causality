/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::causal_tensor_network::truncation::Truncation;
use crate::{CausalTensor, CausalTensorError};
use deep_causality_num::{ConjugateScalar, One, Real, Zero};

/// The real magnitude type of a conjugate scalar (`Self` for reals, the underlying real for complex).
type Re<T> = <T as ConjugateScalar>::Real;

impl<T> CausalTensor<T>
where
    T: ConjugateScalar,
{
    /// Robust truncated thin-SVD: `A ≈ U · diag(S) · Vᴴ`, retaining only the rank selected by
    /// `trunc`.
    ///
    /// This is the tensor-network numerical foundation (Stage 0). The decomposition is computed by
    /// **one-sided Jacobi rotations**, chosen over implicit-shift Golub–Kahan because it delivers
    /// orthonormal factors to high relative accuracy with a simple, branch-stable kernel — the
    /// property TT-SVD and rounding compound over many sweeps.
    ///
    /// For an `m × n` input it returns `(U, S, Vt)` where `U` is `m × k` with orthonormal columns,
    /// `S` is the length-`k` vector of **real** singular values (in `T::Real`) in non-increasing
    /// order, and `Vt` is `k × n` — the conjugate transpose `Vᴴ` of the right singular vectors —
    /// with `k` the retained rank under `trunc`. For a complex scalar this is the genuine Hermitian
    /// SVD (conjugated inner products, real singular values, unitary `U`/`V`); for a real scalar the
    /// conjugation is the identity and it reduces to the ordinary real SVD.
    ///
    /// # Reference
    /// J. Demmel and K. Veselić, "Jacobi's method is more accurate than QR," *SIAM J. Matrix Anal.
    /// Appl.* 13(4), 1204–1245 (1992). <https://doi.org/10.1137/0613074> — establishes the high
    /// relative accuracy of the one-sided Jacobi SVD used here.
    ///
    /// # Errors
    /// Returns [`CausalTensorError::DimensionMismatch`] if `self` is not 2-dimensional, or
    /// [`CausalTensorError::EmptyTensor`] if either dimension is zero.
    pub fn svd_truncated(
        &self,
        trunc: &Truncation<<T as ConjugateScalar>::Real>,
    ) -> Result<(Self, CausalTensor<<T as ConjugateScalar>::Real>, Self), CausalTensorError> {
        if self.shape().len() != 2 {
            return Err(CausalTensorError::DimensionMismatch);
        }
        let m = self.shape()[0];
        let n = self.shape()[1];
        if m == 0 || n == 0 {
            return Err(CausalTensorError::EmptyTensor);
        }

        // One-sided Jacobi requires rows ≥ cols. When the input is wide, decompose its conjugate
        // transpose and swap the roles of U and V on the way out.
        let transposed = m < n;
        let (rows, cols, work) = if transposed {
            (n, m, conj_transpose(self.as_slice(), m, n))
        } else {
            (m, n, self.as_slice().to_vec())
        };

        // jacobi returns: left factor (rows × cols, orthonormal columns), the singular values
        // (length cols, unsorted, real), and the right factor V (cols × cols, unitary).
        let (u_full, sigma, v_full) = jacobi_svd::<T>(work, rows, cols);

        // Sort singular triplets by magnitude, descending.
        let mut order: Vec<usize> = (0..cols).collect();
        order.sort_by(|&a, &b| {
            sigma[b]
                .partial_cmp(&sigma[a])
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        let sorted: Vec<<T as ConjugateScalar>::Real> = order.iter().map(|&j| sigma[j]).collect();

        let k = trunc.retained_rank(&sorted);

        // Assemble the retained factors in the *original* orientation.
        // Non-transposed: U = u_full (m×k), V = v_full (n×k), Vt = Vᴴ.
        // Transposed:     for Aᴴ = U' S V'ᴴ we have A = V' S U'ᴴ ⇒ U = V', Vt = U'ᴴ.
        let (left, right, left_rows, right_rows) = if transposed {
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
        // Vt is k × right_rows; row r is the conjugate of the r-th right singular vector (Vᴴ).
        let mut vt_data = vec![T::zero(); k * right_rows];
        for (r, &j) in order.iter().take(k).enumerate() {
            for c in 0..right_rows {
                vt_data[r * right_rows + c] = right[c * cols + j].conjugate();
            }
        }
        let s_data: Vec<<T as ConjugateScalar>::Real> = sorted.into_iter().take(k).collect();

        let u = CausalTensor::new(u_data, vec![left_rows, k])?;
        let s = CausalTensor::new(s_data, vec![k])?;
        let vt = CausalTensor::new(vt_data, vec![k, right_rows])?;
        Ok((u, s, vt))
    }
}

/// Conjugate-transposes a row-major `rows × cols` buffer into `cols × rows` (plain transpose for a
/// real scalar, Hermitian transpose for complex).
fn conj_transpose<T: ConjugateScalar>(data: &[T], rows: usize, cols: usize) -> Vec<T> {
    let mut out = vec![T::zero(); rows * cols];
    for i in 0..rows {
        for j in 0..cols {
            out[j * rows + i] = data[i * cols + j].conjugate();
        }
    }
    out
}

/// One-sided Jacobi SVD of a tall-or-square matrix (`rows ≥ cols`), conjugate-aware.
///
/// Returns `(u, sigma, v)` where `u` is `rows × cols` with orthonormal columns (for nonzero
/// singular values), `sigma` is the length-`cols` vector of **real** singular values (unsorted), and
/// `v` is the `cols × cols` unitary matrix of right singular vectors. Columns are orthogonalized
/// under the Hermitian inner product `⟨x|y⟩ = Σ x̄ᵢ yᵢ`; each `2×2` sub-problem is reduced to a real
/// Jacobi rotation by factoring the complex off-diagonal into modulus and phase.
fn jacobi_svd<T>(
    mut u: Vec<T>,
    rows: usize,
    cols: usize,
) -> (Vec<T>, Vec<<T as ConjugateScalar>::Real>, Vec<T>)
where
    T: ConjugateScalar,
{
    let two = Re::<T>::one() + Re::<T>::one();
    // Convergence threshold: a small multiple of the working epsilon, so it scales with precision.
    let mut tol = Re::<T>::epsilon();
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
        let mut max_off = Re::<T>::zero();
        for p in 0..cols {
            for q in (p + 1)..cols {
                // Hermitian Gram entries of columns p and q.
                let mut alpha = Re::<T>::zero(); // ⟨x_p|x_p⟩ (real)
                let mut beta = Re::<T>::zero(); // ⟨x_q|x_q⟩ (real)
                let mut gamma = T::zero(); // ⟨x_p|x_q⟩ (complex)
                for i in 0..rows {
                    let uip = u[i * cols + p];
                    let uiq = u[i * cols + q];
                    alpha += uip.modulus_squared();
                    beta += uiq.modulus_squared();
                    gamma += uip.conjugate() * uiq;
                }
                let gmod = gamma.modulus_squared().sqrt(); // |γ| (real)
                let denom = (alpha * beta).sqrt();
                if denom > Re::<T>::zero() {
                    let rel = gmod / denom;
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

                // Real Jacobi angle from (alpha, beta, |γ|); the complex phase is split off into `e`.
                let zeta = (beta - alpha) / (two * gmod);
                let sign = if zeta < Re::<T>::zero() {
                    -Re::<T>::one()
                } else {
                    Re::<T>::one()
                };
                let t = sign / (zeta.abs() + (Re::<T>::one() + zeta * zeta).sqrt());
                let c = Re::<T>::one() / (Re::<T>::one() + t * t).sqrt();
                let s = c * t;

                // Complex Givens rotation with phase ρ = γ/|γ| (unit modulus):
                //   x'_p = c·x_p − conj(ρ)·s·x_q,   x'_q = ρ·s·x_p + c·x_q.
                // For a real scalar ρ = ±1 and this reduces to the ordinary real Jacobi rotation.
                let rho = gamma * T::from_real(Re::<T>::one() / gmod);
                let ct = T::from_real(c);
                let es = rho * T::from_real(s); // ρ·s
                let conj_es = rho.conjugate() * T::from_real(s); // conj(ρ)·s

                for i in 0..rows {
                    let uip = u[i * cols + p];
                    let uiq = u[i * cols + q];
                    u[i * cols + p] = ct * uip - conj_es * uiq;
                    u[i * cols + q] = es * uip + ct * uiq;
                }
                for i in 0..cols {
                    let vip = v[i * cols + p];
                    let viq = v[i * cols + q];
                    v[i * cols + p] = ct * vip - conj_es * viq;
                    v[i * cols + q] = es * vip + ct * viq;
                }
            }
        }
        if max_off <= tol {
            break;
        }
    }

    // Singular values are the column norms; normalize the columns of u into the left factor.
    let mut sigma = vec![Re::<T>::zero(); cols];
    for j in 0..cols {
        let mut norm_sq = Re::<T>::zero();
        for i in 0..rows {
            norm_sq += u[i * cols + j].modulus_squared();
        }
        let norm = norm_sq.sqrt();
        sigma[j] = norm;
        if norm > Re::<T>::zero() {
            let inv_norm = T::from_real(Re::<T>::one() / norm);
            for i in 0..rows {
                u[i * cols + j] *= inv_norm;
            }
        }
    }

    (u, sigma, v)
}
