/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{SimplicialComplex, Skeleton};
use alloc::vec::Vec;
use deep_causality_sparse::CsrMatrix;

impl SimplicialComplex {
    /// Returns the total count of all geometric entities (simplices) in the complex.
    /// This corresponds to the total dimension of the data vector in a `Manifold`.
    pub fn total_simplices(&self) -> usize {
        self.skeletons.iter().map(|s| s.simplices.len()).sum()
    }
    /// Returns the dimension of the highest-order simplex (e.g., 2 for a Triangle mesh, 3 for Tet).
    pub fn max_simplex_dimension(&self) -> usize {
        self.skeletons.len().saturating_sub(1)
    }
    pub fn skeletons(&self) -> &Vec<Skeleton> {
        &self.skeletons
    }

    pub fn boundary_operators(&self) -> &Vec<CsrMatrix<i8>> {
        &self.boundary_operators
    }

    pub fn coboundary_operators(&self) -> &Vec<CsrMatrix<i8>> {
        &self.coboundary_operators
    }

    pub fn hodge_star_operators(&self) -> &Vec<CsrMatrix<f64>> {
        &self.hodge_star_operators
    }
}
