/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructor API for SimplicialComplex.

use crate::{SimplicialComplex, Skeleton};
use deep_causality_sparse::CsrMatrix;

impl<T> SimplicialComplex<T> {
    /// Creates a new SimplicialComplex with the given components.
    ///
    /// # Arguments
    /// * `skeletons` - Vector of skeletons (vertices, edges, faces, etc.)
    /// * `boundary_operators` - Boundary matrices ∂
    /// * `coboundary_operators` - Coboundary matrices (transpose of ∂)
    /// * `hodge_star_operators` - Hodge star operators ⋆
    ///
    /// # Returns
    /// A new SimplicialComplex instance.
    pub fn new(
        skeletons: Vec<Skeleton>,
        boundary_operators: Vec<CsrMatrix<i8>>,
        coboundary_operators: Vec<CsrMatrix<i8>>,
        hodge_star_operators: Vec<CsrMatrix<T>>,
    ) -> Self {
        Self::new_cpu(
            skeletons,
            boundary_operators,
            coboundary_operators,
            hodge_star_operators,
        )
    }
}
