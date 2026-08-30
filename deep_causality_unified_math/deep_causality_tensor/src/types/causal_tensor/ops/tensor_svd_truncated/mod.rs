/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec;
use alloc::vec::Vec;

use crate::types::causal_tensor_network::causal_tensor_train::linalg::matmul;
use crate::types::causal_tensor_network::rng::gaussian_vec;
use crate::types::causal_tensor_network::truncation::{RoundStrategy, Truncation};
use crate::{CausalTensor, CausalTensorError};
use deep_causality_algebra::{ConjugateScalar, Real};
use deep_causality_linear::MatrixView;
use deep_causality_num::Zero;

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
        if self.shape()[0] == 0 || self.shape()[1] == 0 {
            return Err(CausalTensorError::EmptyTensor);
        }
        match trunc.strategy() {
            RoundStrategy::Deterministic => self.svd_jacobi(trunc),
            RoundStrategy::Randomized { oversample, seed } => {
                self.svd_randomized(trunc, oversample, seed)
            }
        }
    }

    /// Deterministic one-sided Jacobi truncated SVD — the default kernel behind
    /// [`CausalTensor::svd_truncated`]. See that method for the contract.
    /// Deterministic one-sided Jacobi truncated SVD — the default kernel behind
    /// [`CausalTensor::svd_truncated`]. See that method for the contract.
    ///
    /// The numerics live in `deep_causality_linear`; what stays here is the truncation policy.
    /// `svd_sorted` returns the factors at full rank `k = min(m, n)` with the singular values real
    /// and sorted descending, so every gate in [`Truncation`] is a prefix length and applying it is
    /// a slice.
    fn svd_jacobi(
        &self,
        trunc: &Truncation<<T as ConjugateScalar>::Real>,
    ) -> Result<(Self, CausalTensor<<T as ConjugateScalar>::Real>, Self), CausalTensorError> {
        let (u_full, sigma, vt_full) = deep_causality_linear::svd_sorted(self)
            .map_err(|_| CausalTensorError::ShapeMismatch)?;

        let k = trunc.retained_rank(&sigma);
        let (left_rows, k_full) = u_full.shape();
        let (_, right_rows) = vt_full.shape();

        // U keeps its leading k columns; Vt its leading k rows.
        let mut u_data = vec![T::zero(); left_rows * k];
        for row in 0..left_rows {
            for col in 0..k {
                u_data[row * k + col] = u_full.as_slice()[row * k_full + col];
            }
        }
        let vt_data = vt_full.as_slice()[..k * right_rows].to_vec();
        let s_data: Vec<<T as ConjugateScalar>::Real> = sigma.into_iter().take(k).collect();

        let u = CausalTensor::new(u_data, vec![left_rows, k])?;
        let s = CausalTensor::new(s_data, vec![k])?;
        let vt = CausalTensor::new(vt_data, vec![k, right_rows])?;
        Ok((u, s, vt))
    }

    /// Adaptive randomized range-finder truncated SVD — the opt-in kernel behind
    /// [`CausalTensor::svd_truncated`] when [`RoundStrategy::Randomized`] is selected.
    ///
    /// Sketches the column space of `A` with a Gaussian matrix `Ω`, orthonormalizes `Y = A·Ω` by QR
    /// to a basis `Q`, projects `B = Qᴴ·A`, takes the (small) deterministic SVD of `B`, and lifts the
    /// left factors back through `Q` (`U = Q·U_B`). The sketch size `ℓ` grows adaptively until the
    /// captured residual `‖A − Q·Qᴴ·A‖_F` meets the policy tolerance, or the full rank is reached.
    /// Cost is `O(rows·cols·ℓ)` — cheaper than the full Jacobi SVD whenever the retained rank
    /// `ℓ ≪ min(rows, cols)`, which is exactly the low-rank regime of TT rounding.
    ///
    /// # Reference
    /// N. Halko, P. G. Martinsson, J. A. Tropp, "Finding Structure with Randomness," *SIAM Review*
    /// 53(2), 217–288 (2011) — the randomized range-finder; H. Al Daas et al., "Randomized algorithms
    /// for rounding in the tensor-train format," *SIAM J. Sci. Comput.* (2023) — its use in TT rounding.
    fn svd_randomized(
        &self,
        trunc: &Truncation<Re<T>>,
        oversample: usize,
        seed: u64,
    ) -> Result<(Self, CausalTensor<Re<T>>, Self), CausalTensorError> {
        let rows = self.shape()[0];
        let cols = self.shape()[1];
        let a = self.as_slice();
        let maxr = rows.min(cols);

        // A real bond cap fixes the target rank; otherwise the rank is tolerance-driven and the
        // sketch grows adaptively from a modest start.
        let bond_capped = trunc.max_bond() < maxr;
        let mut ell = if bond_capped {
            trunc.max_bond().saturating_add(oversample).min(maxr).max(1)
        } else {
            oversample.max(1).saturating_mul(2).min(maxr).max(1)
        };

        // ‖A‖_F for the relative residual test.
        let mut norm_a_sq = Re::<T>::zero();
        for &x in a {
            norm_a_sq += x.modulus_squared();
        }
        let norm_a = norm_a_sq.sqrt();

        // Range-finding loop: grow ℓ until the projection residual meets the tolerance.
        let (q, qd, b) = loop {
            let omega = gaussian_vec::<T>(
                cols * ell,
                seed ^ (ell as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
            );
            let y = matmul(a, rows, cols, &omega, ell); // rows × ell
            let yt = CausalTensor::new(y, vec![rows, ell])?;
            let (q, _r) = yt.qr()?;
            let qd = q.shape()[1]; // ≤ ell
            let qh = conj_transpose(q.as_slice(), rows, qd);
            let b = matmul(&qh, qd, rows, a, cols); // qd × cols

            if ell < maxr {
                // Residual ‖A − Q·B‖_F: Q·B is the rank-ℓ projection of A onto the sketch range.
                let qb = matmul(q.as_slice(), rows, qd, &b, cols);
                let mut res_sq = Re::<T>::zero();
                for (av, qbv) in a.iter().zip(qb.iter()) {
                    res_sq += (*av - *qbv).modulus_squared();
                }
                let res = res_sq.sqrt();
                let rel = trunc.rel_tol() * norm_a;
                let threshold = if rel > trunc.abs_tol() {
                    rel
                } else {
                    trunc.abs_tol()
                };
                if res > threshold {
                    ell = ell.saturating_mul(2).min(maxr);
                    continue;
                }
            }
            break (q, qd, b);
        };

        // Deterministic truncated SVD of the small qd × cols matrix B, then lift U back through Q.
        let bt = CausalTensor::new(b, vec![qd, cols])?;
        let (ub, s, vt) = bt.svd_jacobi(trunc)?; // ub: qd×k, s: k, vt: k×cols
        let k = s.len();
        let u = matmul(q.as_slice(), rows, qd, ub.as_slice(), k); // rows × k
        let u = CausalTensor::new(u, vec![rows, k])?;
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
