/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_sparse::CsrMatrix;

use crate::Skeleton;

mod base_topology;
mod builder;
mod display;
mod getters;
mod ops_boundary;
mod simplicial_topology;

pub use builder::SimplicialComplexBuilder;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SimplicialComplex {
    /// Storage of geometric entities (Points, Lines, Triangles...)
    pub(crate) skeletons: Vec<Skeleton>,
    /// The Boundary Operators (∂).
    /// `boundary_operators[k]` is a matrix of size `(N_k x N_{k+1})`.
    /// It maps a `(k+1)`-chain to a `k`-chain.
    ///
    /// Implementation: `deep_causality_sparse::CsrMatrix<i8>`
    /// Values are `{-1, 0, 1}` representing orientation.
    pub(crate) boundary_operators: Vec<CsrMatrix<i8>>,
    /// The Coboundary / Adjacency Cache (Optional but recommended for Comonad speed).
    /// Transpose of boundary operators.
    /// coboundary[k] is a matrix of size (N_{k+1} x N_k).
    /// Used to find "Who contains me?" efficiently.
    pub(crate) coboundary_operators: Vec<CsrMatrix<i8>>,
    /// The Hodge Star Operators (⋆).
    /// `hodge_star_operators[k]` is a matrix mapping k-forms to (n-k)-forms.
    /// Its dimensions are `(N_{n-k} x N_k)`.
    pub(crate) hodge_star_operators: Vec<CsrMatrix<f64>>,
}

impl SimplicialComplex {
    pub fn new(
        skeletons: Vec<Skeleton>,
        boundary_operators: Vec<CsrMatrix<i8>>,
        coboundary_operators: Vec<CsrMatrix<i8>>,
        hodge_star_operators: Vec<CsrMatrix<f64>>,
    ) -> Self {
        Self {
            skeletons,
            boundary_operators,
            coboundary_operators,
            hodge_star_operators,
        }
    }
}
