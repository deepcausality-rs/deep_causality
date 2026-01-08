/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU implementation of SimplicialComplex constructors.

use crate::{SimplicialComplex, Skeleton};
use deep_causality_sparse::CsrMatrix;

impl<T> SimplicialComplex<T> {
    /// CPU implementation of SimplicialComplex constructor.
    pub(crate) fn new_cpu(
        skeletons: Vec<Skeleton>,
        boundary_operators: Vec<CsrMatrix<i8>>,
        coboundary_operators: Vec<CsrMatrix<i8>>,
        hodge_star_operators: Vec<CsrMatrix<T>>,
    ) -> Self {
        Self {
            skeletons,
            boundary_operators,
            coboundary_operators,
            hodge_star_operators,
        }
    }
}
