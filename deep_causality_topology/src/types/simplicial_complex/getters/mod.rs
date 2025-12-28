/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Getter methods for SimplicialComplex fields.

use crate::{SimplicialComplex, Skeleton};
use deep_causality_sparse::CsrMatrix;

impl SimplicialComplex {
    /// Returns the total count of all geometric entities (simplices) in the complex.
    pub fn total_simplices(&self) -> usize {
        self.skeletons.iter().map(|s| s.simplices.len()).sum()
    }

    /// Returns the dimension of the highest-order simplex.
    pub fn max_simplex_dimension(&self) -> usize {
        self.skeletons.len().saturating_sub(1)
    }

    /// Returns a reference to all skeletons.
    pub fn skeletons(&self) -> &Vec<Skeleton> {
        &self.skeletons
    }

    /// Returns a reference to all boundary operators.
    pub fn boundary_operators(&self) -> &Vec<CsrMatrix<i8>> {
        &self.boundary_operators
    }

    /// Returns a reference to all coboundary operators.
    pub fn coboundary_operators(&self) -> &Vec<CsrMatrix<i8>> {
        &self.coboundary_operators
    }

    /// Returns a reference to all Hodge star operators.
    pub fn hodge_star_operators(&self) -> &Vec<CsrMatrix<f64>> {
        &self.hodge_star_operators
    }
}
