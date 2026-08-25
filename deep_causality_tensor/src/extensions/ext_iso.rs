/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Feature-gated isomorphism between rank-2 [`CausalTensor<F>`] and
//! [`CsrMatrix<F>`].
//!
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
//! by the dependency direction. The iso ships in two pieces, both
//! rooted in this crate:
//!
//! - **Forward** (`CausalTensor<F>` -> `CsrMatrix<F>`): Tier 1
//!   `impl TryFrom<CausalTensor<F>> for CsrMatrix<F>`. Returns
//!   [`CsrFromTensorError`] on rank ≠ 2. Uses `TryFrom` (not `From`)
//!   because the conversion is intrinsically partial — only rank-2
//!   tensors are matrices.
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

use crate::CausalTensor;
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;
use deep_causality_algebra::CommutativeSemiring;
use deep_causality_algebra::iso::witness::Iso;
use deep_causality_linear::CsrMatrix;

/// Error returned by [`CsrMatrix::try_from`] when the input
/// [`CausalTensor`] has a rank other than 2.
///
/// The forward direction of the tensor / sparse iso is intrinsically
/// partial: only rank-2 tensors are matrices. `From` is reserved for
/// total conversions, so the surface uses `TryFrom` instead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsrFromTensorError {
    /// The rank of the input tensor (which must be 2 for the conversion
    /// to succeed).
    pub rank: usize,
}

impl fmt::Display for CsrFromTensorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CausalTensor -> CsrMatrix requires rank 2, got rank {}",
            self.rank
        )
    }
}

impl core::error::Error for CsrFromTensorError {}

// =============================================================================
// Forward (Tier 1): CausalTensor<F> -> CsrMatrix<F> via TryFrom
// =============================================================================

impl<F> TryFrom<CausalTensor<F>> for CsrMatrix<F>
where
    F: CommutativeSemiring + Copy + PartialEq,
{
    type Error = CsrFromTensorError;

    /// Materialise a rank-2 [`CausalTensor`] as a sparse matrix. Non-zero
    /// entries are stored in row-major order; zero entries are dropped.
    ///
    /// Returns [`CsrFromTensorError`] when `tensor.num_dim() != 2`. The
    /// forward direction is partial; `From` would lie about being total.
    fn try_from(tensor: CausalTensor<F>) -> Result<Self, Self::Error> {
        let shape = tensor.shape();
        if shape.len() != 2 {
            return Err(CsrFromTensorError { rank: shape.len() });
        }
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
        Ok(CsrMatrix::from_triplets(rows, cols, &triplets)
            .expect("triplets are within bounds by construction"))
    }
}

// =============================================================================
// Reverse (Tier 2): Iso<CsrMatrix<F>, CausalTensor<F>> on CsrMatrix as Self
// =============================================================================

impl<F> Iso<CsrMatrix<F>, CausalTensor<F>> for CsrMatrix<F>
where
    F: CommutativeSemiring + Copy + PartialEq,
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

    /// Delegate to the fallible forward [`TryFrom`] impl.
    ///
    /// # Panics
    ///
    /// The Tier 2 `Iso<S, T>` trait requires an infallible `to_source`.
    /// Since the forward direction is intrinsically partial (only rank-2
    /// tensors can become matrices), this method calls
    /// `CsrMatrix::try_from(t).expect(...)` and panics on rank ≠ 2.
    /// Callers wanting graceful failure should use
    /// `CsrMatrix::try_from(t)` directly instead of the `Iso` trait
    /// surface.
    fn to_source(t: CausalTensor<F>) -> CsrMatrix<F> {
        CsrMatrix::try_from(t).expect(
            "Iso::to_source requires a rank-2 CausalTensor; use TryFrom for graceful failure",
        )
    }
}

// =============================================================================
// Ergonomic alias, as an extension trait
// =============================================================================

/// `to_dense` for a sparse matrix, as an extension trait.
///
/// # Why a trait rather than an inherent method
///
/// `CsrMatrix` belongs to `deep_causality_linear` and this conversion belongs here — a crate cannot
/// write an inherent `impl` for a type it does not own (E0116). The trait impl is allowed because
/// `CausalTensor` is local, which is the same reason the [`TryFrom`] and [`Iso`] impls above can
/// live here. A caller brings the method into scope with a `use`.
pub trait ToDenseTensor<F> {
    /// Materialises this sparse matrix as a dense rank-2 [`CausalTensor`].
    ///
    /// Equivalent to `<Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)`. Missing
    /// entries become `F::zero()`.
    ///
    /// # Example
    ///
    /// ```
    /// use deep_causality_linear::CsrMatrix;
    /// use deep_causality_tensor::ToDenseTensor;
    ///
    /// let sparse = CsrMatrix::from_triplets(2, 3, &[(0, 0, 1.0), (1, 2, 6.0)]).unwrap();
    /// let dense = sparse.to_dense();
    /// assert_eq!(dense.shape(), &[2, 3]);
    /// assert_eq!(dense.as_slice(), &[1.0, 0.0, 0.0, 0.0, 0.0, 6.0]);
    /// ```
    fn to_dense(self) -> CausalTensor<F>;
}

impl<F> ToDenseTensor<F> for CsrMatrix<F>
where
    F: CommutativeSemiring + Copy + PartialEq,
{
    fn to_dense(self) -> CausalTensor<F> {
        <Self as Iso<CsrMatrix<F>, CausalTensor<F>>>::to_target(self)
    }
}
