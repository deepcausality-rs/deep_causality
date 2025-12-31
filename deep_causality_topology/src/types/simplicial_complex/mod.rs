/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! SimplicialComplex type for representing simplicial structures.

use crate::Skeleton;
use deep_causality_sparse::CsrMatrix;

// Submodule declarations (folder-based)
mod api;
mod boundary;
mod builder;
mod constructors;
mod display;
mod getters;

mod ops;
mod topology;

// Re-export public API
pub use builder::SimplicialComplexBuilder;

/// A simplicial complex representing a geometric structure.
///
/// Contains skeletons (vertices, edges, faces, etc.) and the associated
/// boundary, coboundary, and Hodge star operators.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct SimplicialComplex {
    /// Storage of geometric entities (Points, Lines, Triangles...)
    pub(crate) skeletons: Vec<Skeleton>,
    /// The Boundary Operators (∂).
    pub(crate) boundary_operators: Vec<CsrMatrix<i8>>,
    /// The Coboundary / Adjacency Cache.
    pub(crate) coboundary_operators: Vec<CsrMatrix<i8>>,
    /// The Hodge Star Operators (⋆).
    pub(crate) hodge_star_operators: Vec<CsrMatrix<f64>>,
}
