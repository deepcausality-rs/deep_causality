/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::SimplicialComplex;
use alloc::sync::Arc;
use core::fmt::{Debug, Display, Formatter};
use deep_causality_sparse::CsrMatrix;

mod algebra;
mod arithmetic;

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

impl<T> Chain<T> {
    pub fn complex(&self) -> &Arc<SimplicialComplex> {
        &self.complex
    }

    pub fn grade(&self) -> usize {
        self.grade
    }

    pub fn weights(&self) -> &CsrMatrix<T> {
        &self.weights
    }
}

impl<T> Display for Chain<T>
where
    T: Debug + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Chain:")?;
        writeln!(f, "  Grade: {}", self.grade)?;
        writeln!(f, "  Weights: {:?}", self.weights)?; // Using Debug for CsrMatrix
        Ok(())
    }
}
