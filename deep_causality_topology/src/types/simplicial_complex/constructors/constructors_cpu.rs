/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU implementation of SimplicialComplex constructors.

use crate::{SimplicialComplex, Skeleton};
use deep_causality_sparse::CsrMatrix;

impl SimplicialComplex {
    /// CPU implementation of SimplicialComplex constructor.
    pub(crate) fn new_cpu(
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
