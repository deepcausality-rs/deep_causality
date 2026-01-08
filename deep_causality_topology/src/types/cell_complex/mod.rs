/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod boundary_operator;

use crate::traits::cw_complex::{CWComplex, Cell};
pub use boundary_operator::BoundaryOperator;
use deep_causality_sparse::CsrMatrix;
use std::collections::HashMap;

/// A CW complex with arbitrary cell types.
///
/// This structure holds the explicit collection of cells and their incidence relations.
/// It is more general than a Lattice or SimplicialComplex, allowing for any topology
/// built from cells implementing the `Cell` trait.
pub struct CellComplex<C: Cell> {
    /// cells[k] = all k-cells
    /// Ordered vector to establish matrix indices.
    /// cells[k] = all k-cells
    /// Ordered vector to establish matrix indices.
    cells: Vec<Vec<C>>,
}

impl<C: Cell> CellComplex<C> {
    // --- Constructors ---

    /// Build from a collection of cells.
    /// Automatically computes dimensions and organizes cells.
    /// Note: This assumes the collection is closed under boundary operations
    /// (i.e. if a cell is included, its boundary cells must also be included).
    /// If not, it will panic or error? Let's assume valid input or filter.
    pub fn from_cells(all_cells: Vec<C>) -> Self {
        // Sort cells by dimension
        let max_dim = all_cells.iter().map(|c| c.dim()).max().unwrap_or(0);
        let mut cells_by_dim: Vec<Vec<C>> = vec![Vec::new(); max_dim + 1];

        for cell in all_cells {
            cells_by_dim[cell.dim()].push(cell);
        }

        // Ensure canonical ordering (if Cell implements Ord? Spec says Eq+Hash).
        // If not Ord, we rely on insertion order.
        // It's better if we deduplicate.
        for dim_cells in cells_by_dim.iter_mut() {
            // Deduplication requires Hash/Eq
            // Efficient: convert to HashSet then back to Vec?
            // Order matters for matrix indices.
            let mut seen = std::collections::HashSet::new();
            dim_cells.retain(|c| seen.insert(c.clone()));
        }

        Self {
            cells: cells_by_dim,
        }
    }

    // --- Getters ---

    pub fn cells_vec(&self, k: usize) -> &[C] {
        if k < self.cells.len() {
            &self.cells[k]
        } else {
            &[]
        }
    }

    pub fn dimension(&self) -> usize {
        self.cells.len().saturating_sub(1)
    }

    /// Compute (or retrieve) the boundary matrix ∂_k.
    /// Since we can't mutate `self` in `boundary_matrix` (Common pattern),
    /// we might compute it on demand and return it, or use interior mutability.
    /// Given `cells` are fixed, we can compute it deterministically.
    /// The trait `CWComplex` requires `boundary_matrix(&self)`.
    pub fn compute_boundary_matrix(&self, k: usize) -> CsrMatrix<i8> {
        if k == 0 || k >= self.cells.len() {
            return CsrMatrix::new();
        }

        let rows = self.num_cells(k - 1);
        let cols = self.num_cells(k);

        // Map (k-1)-cells to indices
        let mut row_map = HashMap::new();
        for (i, cell) in self.cells_vec(k - 1).iter().enumerate() {
            row_map.insert(cell, i);
        }

        let mut triplets = Vec::new();

        for (j, cell) in self.cells_vec(k).iter().enumerate() {
            let boundary = cell.boundary();
            // boundary is Vec<(Cell, i8)>
            for (term_cell, coeff) in boundary {
                if let Some(&i) = row_map.get(&term_cell) {
                    triplets.push((i, j, coeff));
                } else {
                    // Start logging?
                }
            }
        }

        CsrMatrix::from_triplets(rows, cols, &triplets).unwrap_or_else(|_| CsrMatrix::new())
    }
}

impl<C: Cell> CWComplex for CellComplex<C> {
    type CellType = C;

    fn cells(&self, k: usize) -> Box<dyn Iterator<Item = Self::CellType> + '_> {
        Box::new(self.cells_vec(k).iter().cloned())
    }

    fn num_cells(&self, k: usize) -> usize {
        self.cells_vec(k).len()
    }

    fn max_dim(&self) -> usize {
        self.dimension()
    }

    fn boundary_matrix(&self, k: usize) -> CsrMatrix<i8> {
        self.compute_boundary_matrix(k)
    }

    fn betti_number(&self, k: usize) -> usize {
        let n_k = self.num_cells(k);
        // Rank of boundary_k: C_k -> C_{k-1}
        let rank_k = self.rank_of_matrix(k);
        // Rank of boundary_{k+1}: C_{k+1} -> C_k
        let rank_k_next = self.rank_of_matrix(k + 1);

        // dim(Ker ∂_k) = n_k - rank(∂_k)
        // dim(Im ∂_{k+1}) = rank(∂_{k+1})
        // b_k = dim(Ker ∂_k) - dim(Im ∂_{k+1})

        let dim_ker = n_k.saturating_sub(rank_k);
        dim_ker.saturating_sub(rank_k_next)
    }
}

impl<C: Cell> CellComplex<C> {
    fn rank_of_matrix(&self, k: usize) -> usize {
        // We use CpuBackend for SVD computation to determine rank
        // This makes CellComplex depend on Tensor capability for homology
        use deep_causality_tensor::{CpuBackend, LinearAlgebraBackend, TensorBackend};

        let matrix = self.boundary_matrix(k);
        let (rows, cols) = matrix.shape();
        if rows == 0 || cols == 0 {
            return 0;
        }

        // Convert CSR to Dense Vector for Tensor creation
        // We assume numeric type i8 from boundary is mapped to f64 for SVD
        let mut data = vec![0.0f64; rows * cols];

        // Iterate using CSR structure
        let row_ptrs = matrix.row_indices();
        let col_idxs = matrix.col_indices();
        let vals = matrix.values();

        for r in 0..rows {
            let start = row_ptrs[r];
            let end = row_ptrs[r + 1];
            for idx in start..end {
                let c = col_idxs[idx];
                let v = vals[idx];
                data[r * cols + c] = v as f64;
            }
        }

        let tensor = <CpuBackend as TensorBackend>::create(&data, &[rows, cols]);

        // SVD: M = U S V^T
        // Sigmas are in S (vector)
        // SVD: M = U S V^T
        // Sigmas are in S (vector)
        // CpuBackend::svd panics on error, returns tuple directly
        let (_, s, _) = <CpuBackend as LinearAlgebraBackend>::svd(&tensor);

        // Count non-zero singular values
        let s_vec: Vec<f64> = <CpuBackend as TensorBackend>::to_vec(&s);
        let tolerance = 1e-5;
        s_vec.iter().filter(|x| x.abs() > tolerance).count()
    }
}
