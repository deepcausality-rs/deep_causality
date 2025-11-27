/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;
use deep_causality_sparse::CsrMatrix;

use crate::Skeleton;

mod display;
mod getters;
mod ops_boundary;

pub struct SimplicialComplex {
    /// Storage of geometric entities (Points, Lines, Triangles...)
    pub(crate) skeletons: Vec<Skeleton>,
    /// The Boundary Operators (∂).
    /// boundary[k] is a matrix of size (N_{k-1} x N_k).
    /// Maps a k-chain to a (k-1)-chain.
    ///
    /// Implementation: deep_causality_sparse::CsrMatrix<i8>
    /// Values are {-1, 0, 1} representing orientation.
    pub(crate) boundary_operators: Vec<CsrMatrix<i8>>,
    /// The Coboundary / Adjacency Cache (Optional but recommended for Comonad speed).
    /// Transpose of boundary operators.
    /// coboundary[k] is a matrix of size (N_{k+1} x N_k).
    /// Used to find "Who contains me?" efficiently.
    pub(crate) coboundary_operators: Vec<CsrMatrix<i8>>,
}

impl SimplicialComplex {
    pub fn new(
        skeletons: Vec<Skeleton>,
        boundary_operators: Vec<CsrMatrix<i8>>,
        coboundary_operators: Vec<CsrMatrix<i8>>,
    ) -> Self {
        Self {
            skeletons,
            boundary_operators,
            coboundary_operators,
        }
    }
}
