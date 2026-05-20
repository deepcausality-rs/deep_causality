/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Feature-gated isomorphism between rank-2 [`CausalTensor<F>`] and
//! [`CsrMatrix<F>`].
//!
//! This module is enabled by the `tensor-iso` Cargo feature. Off by
//! default so that downstream sparse users who don't need the tensor
//! interop don't pay the compile cost of pulling in
//! `deep_causality_tensor`.
//!
//! Lives under `src/extensions/` (alongside `ext_hkt`) following the
//! convention that anything bridging `deep_causality_sparse` to another
//! DC crate is an `ext_*` extension.
//!
//! ## Design (mixed-tier)
//!
//! `deep_causality_sparse` depends on `deep_causality_tensor` (via the
//! `tensor-iso` feature); the reverse is blocked by the orphan rule and
//! by the dependency direction. The iso ships in two pieces, both
//! rooted in this crate:
//!
//! - **Forward** (`CausalTensor<F>` -> `CsrMatrix<F>`): Tier 1
//!   `impl From<CausalTensor<F>> for CsrMatrix<F>`. Panics on rank ≠ 2;
//!   matches the rest of the tensor API which panics on shape
//!   preconditions. A `TryFrom` form is deferred to a follow-up.
//! - **Reverse** (`CsrMatrix<F>` -> `CausalTensor<F>`): Tier 2
//!   `impl Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>` with
//!   `CsrMatrix<F>` as `Self` (orphan-safe). An inherent ergonomic
//!   alias `CsrMatrix::to_dense()` delegates to the iso's `to_target`.
//!
//! ## What's NOT here
//!
//! No marker subtraits. Neither `CausalTensor<F>` nor `CsrMatrix<F>`
//! currently implements `Group`/`Ring`/`Field`, so `GroupIso`,
//! `RingIso`, etc. would not type-check. The base `Iso<S, T>` is the
//! right surface.

use crate::CsrMatrix;
use deep_causality_num::Zero;
use deep_causality_num::iso::witness::Iso;
use deep_causality_tensor::CausalTensor;

// =============================================================================
// Forward (Tier 1): CausalTensor<F> -> CsrMatrix<F>
// =============================================================================

impl<F> From<CausalTensor<F>> for CsrMatrix<F>
where
    F: Clone + Copy + Zero + PartialEq,
{
    /// Materialise a rank-2 [`CausalTensor`] as a sparse matrix. Non-zero
    /// entries are stored in row-major order; zero entries are dropped.
    ///
    /// # Panics
    ///
    /// Panics if `tensor.num_dim() != 2`. The forward direction is partial;
    /// callers handling unknown ranks should pattern-match `tensor.shape()`
    /// first.
    fn from(tensor: CausalTensor<F>) -> Self {
        let shape = tensor.shape();
        assert_eq!(
            shape.len(),
            2,
            "CausalTensor -> CsrMatrix requires rank 2, got rank {}",
            shape.len()
        );
        let rows = shape[0];
        let cols = shape[1];
        let data = tensor.data();

        let mut triplets: Vec<(usize, usize, F)> = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let v = data[r * cols + c];
                if v != F::zero() {
                    triplets.push((r, c, v));
                }
            }
        }
        // Construction cannot fail: row/col bounds are honoured by the
        // double loop above.
        CsrMatrix::from_triplets(rows, cols, &triplets)
            .expect("triplets are within bounds by construction")
    }
}

// =============================================================================
// Reverse (Tier 2): Iso<CsrMatrix<F>, CausalTensor<F>> on CsrMatrix as Self
// =============================================================================

impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>
where
    F: Clone + Copy + Zero + PartialEq,
{
    /// Materialise this sparse matrix as a dense rank-2
    /// [`CausalTensor`] of the matching shape. Missing entries are
    /// `F::zero()`.
    fn to_target(s: CsrMatrix<F>) -> CausalTensor<F> {
        let (rows, cols) = s.shape();
        let row_ptr = s.row_indices();
        let col_idx = s.col_indices();
        let vals = s.values();

        let mut dense = vec![F::zero(); rows * cols];
        for r in 0..rows {
            let start = row_ptr[r];
            let end = row_ptr[r + 1];
            for k in start..end {
                let c = col_idx[k];
                dense[r * cols + c] = vals[k];
            }
        }
        CausalTensor::new(dense, vec![rows, cols])
            .expect("rows * cols matches data length by construction")
    }

    /// Delegate to the forward [`From`] impl.
    fn to_source(t: CausalTensor<F>) -> CsrMatrix<F> {
        CsrMatrix::from(t)
    }
}

// =============================================================================
// Ergonomic alias
// =============================================================================

impl<F> CsrMatrix<F>
where
    F: Clone + Copy + Zero + PartialEq,
{
    /// Materialise this sparse matrix as a dense rank-2
    /// [`CausalTensor`]. Equivalent to
    /// `<Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)`.
    ///
    /// Available only with the `tensor-iso` feature enabled.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use deep_causality_sparse::CsrMatrix;
    /// let triplets = vec![(0, 0, 1.0), (1, 2, 6.0)];
    /// let sparse = CsrMatrix::from_triplets(2, 3, &triplets).unwrap();
    /// let dense = sparse.to_dense();
    /// assert_eq!(dense.shape(), &[2, 3]);
    /// ```
    pub fn to_dense(self) -> CausalTensor<F> {
        <Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)
    }
}
