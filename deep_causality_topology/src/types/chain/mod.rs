/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::simplicial_complex::SimplicialComplex;
use alloc::sync::Arc;
use deep_causality_sparse::CsrMatrix;

mod display;
mod getters;

/// Represents a weighted collection of simplices.
/// (e.g., A path is a Chain<f64> on the 1-skeleton where weights are 1.0).
#[derive(Debug, Clone, PartialEq)]
pub struct Chain<T> {
    pub(crate) complex: Arc<SimplicialComplex>,
    pub(crate) grade: usize,
    /// Sparse vector of active simplices.
    /// Reuses CsrMatrix logic (1 row, N cols) for efficient sparse operations.
    pub(crate) weights: CsrMatrix<T>,
}

impl<T> Chain<T> {
    pub fn new(complex: Arc<SimplicialComplex>, grade: usize, weights: CsrMatrix<T>) -> Self {
        Self {
            complex,
            grade,
            weights,
        }
    }
}
