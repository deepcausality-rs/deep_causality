/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::cell::Cell;
use deep_causality_sparse::CsrMatrix;
use std::borrow::Cow;

/// Types that form a chain complex (a CW-style cellular decomposition).
/// This generalizes simplicial complexes, cubical lattices, and arbitrary cellular decompositions.
///
/// The trait uses static dispatch end-to-end: cell iteration is exposed via a GAT-backed
/// `CellIter<'_>` and the boundary / coboundary matrices return `Cow<'_, CsrMatrix<i8>>`
/// so cache-rich implementors can vend `Cow::Borrowed` (zero copy) while compute-on-demand
/// implementors return `Cow::Owned`.
pub trait ChainComplex {
    /// The type of cells in this complex.
    type CellType: Cell;

    /// The concrete iterator type returned by `cells`.
    type CellIter<'a>: Iterator<Item = Self::CellType>
    where
        Self: 'a;

    /// Iterate over all k-cells in the complex.
    fn cells(&self, k: usize) -> Self::CellIter<'_>;

    /// Get the total number of k-cells.
    fn num_cells(&self, k: usize) -> usize;

    /// The maximum dimension of cells in the complex.
    fn max_dim(&self) -> usize;

    /// Return the boundary matrix ∂_k as a sparse matrix.
    /// Rows correspond to (k-1)-cells, columns to k-cells.
    ///
    /// Cache-rich implementors return `Cow::Borrowed`. Compute-on-demand implementors
    /// return `Cow::Owned`.
    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;

    /// Return the coboundary matrix δ_k as a sparse matrix.
    /// δ_k is the transpose of ∂_{k+1}.
    ///
    /// Cache-rich implementors return `Cow::Borrowed`. Compute-on-demand implementors
    /// return `Cow::Owned`.
    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;

    /// Compute the k-th Betti number β_k = dim(H_k).
    /// H_k = ker(∂_k) / im(∂_{k+1})
    fn betti_number(&self, k: usize) -> usize;
}
