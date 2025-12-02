/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{SimplicialComplex, Skeleton};
use alloc::vec::Vec;
use deep_causality_sparse::CsrMatrix;

impl SimplicialComplex {
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
