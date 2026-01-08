/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;
use std::hash::Hash;

/// Marker trait for cell types in a CW complex.
pub trait Cell: Clone + Eq + Hash + Send + Sync + 'static {
    /// Dimension of this cell.
    fn dim(&self) -> usize;

    /// Boundary as signed sum of lower-dimensional cells.
    /// This provides the algebraic topology structure.
    fn boundary(&self) -> Vec<(Self, i8)>;
}

/// Types that form a CW complex (Closure-Finite Weak Topology).
/// This generalizes simplicial complexes, cubic lattices, and arbitrary cellular decompositions.
pub trait CWComplex {
    /// The type of cells in this complex.
    type CellType: Cell;

    /// Iterate over all k-cells in the complex.
    fn cells(&self, k: usize) -> Box<dyn Iterator<Item = Self::CellType> + '_>;

    /// Get the total number of k-cells.
    fn num_cells(&self, k: usize) -> usize;

    /// The maximum dimension of cells in the complex.
    fn max_dim(&self) -> usize;

    /// Compute the boundary matrix ∂_k as a sparse matrix.
    /// Rows correspond to (k-1)-cells, columns to k-cells.
    fn boundary_matrix(&self, k: usize) -> CsrMatrix<i8>;

    /// Compute the k-th Betti number β_k = dim(H_k).
    /// H_k = ker(∂_k) / im(∂_{k+1})
    fn betti_number(&self, k: usize) -> usize;
}
