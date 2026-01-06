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
#[derive(Debug, Clone, PartialEq)]
pub struct SimplicialComplex<T> {
    /// Storage of geometric entities (Points, Lines, Triangles...)
    pub(crate) skeletons: Vec<Skeleton>,
    /// The Boundary Operators (∂).
    pub(crate) boundary_operators: Vec<CsrMatrix<i8>>,
    /// The Coboundary / Adjacency Cache.
    pub(crate) coboundary_operators: Vec<CsrMatrix<i8>>,
    /// The Hodge Star Operators (⋆).
    pub(crate) hodge_star_operators: Vec<CsrMatrix<T>>,
}

impl<T> SimplicialComplex<T> {
    /// Maps the underlying scalar type of the SimplicialComplex to a new type U.
    ///
    /// This transforms:
    /// - `hodge_star_operators: Vec<CsrMatrix<T>>` -> `Vec<CsrMatrix<U>>`
    ///
    /// The structural components (skeletons, boundary operators) remain unchanged.
    ///
    /// # Arguments
    /// * `f` - A function or closure that transforms `T` to `U`.
    pub fn map<U, F>(self, mut f: F) -> SimplicialComplex<U>
    where
        F: FnMut(T) -> U,
    {
        let new_hodge_ops = self
            .hodge_star_operators
            .into_iter()
            .map(|op| {
                // Deconstruct CsrMatrix, map values, reconstruct.
                // Assuming CsrMatrix has methods to decompose or we can access fields if crate-local.
                // Since we are in the same crate (types/simplicial_complex), but CsrMatrix is generic from another crate...
                // We rely on public API: row_indices(), col_indices(), values().
                // And from_parts (unsafe) or from_triplets.
                // Using safe from_triplets is slower. Using unsafe from_parts is better if structure matches.
                // Let's rely on mapping values directly if possible.

                let (cols, rows, values, shape) = op.into_parts();
                let new_values: Vec<U> = values.into_iter().map(&mut f).collect();

                // Safety: We preserve the structure (indices and shape), only changing values.
                // If T and U have same structural implication for 0, this is safe.
                unsafe { CsrMatrix::from_parts(cols, rows, new_values, shape) }
            })
            .collect();

        SimplicialComplex {
            skeletons: self.skeletons,
            boundary_operators: self.boundary_operators,
            coboundary_operators: self.coboundary_operators,
            hodge_star_operators: new_hodge_ops,
        }
    }
}

impl<T> Default for SimplicialComplex<T> {
    fn default() -> Self {
        Self {
            skeletons: Vec::new(),
            boundary_operators: Vec::new(),
            coboundary_operators: Vec::new(),
            hodge_star_operators: Vec::new(),
        }
    }
}
