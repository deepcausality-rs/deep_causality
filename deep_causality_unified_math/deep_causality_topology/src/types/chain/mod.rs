/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::SimplicialComplex;
use core::fmt::Debug;
use deep_causality_linear::CsrMatrix;
use std::sync::Arc;

mod algebra;
mod arithmetic;
mod display;

/// A weighted collection of simplices: the chain group `C_k(K; G)`.
///
/// # The two parameters
///
/// `R` is the **metric precision** of the complex, and `G` is the **coefficient group** the weights
/// live in. They are independent, and the mathematics says so: `C_k(K; G)` is formal sums of
/// `k`-simplices with coefficients in an abelian group `G`, over a complex `K`. The functor is
/// `C_k(K; −)`, acting in the coefficient slot with `K` held fixed, and the Hodge ⋆ on `K` is
/// determined by the metric rather than by `G`.
///
/// Keeping them separate is what lets `fmap` act in the coefficient slot alone: it clones the
/// complex, so the functor identity law `fmap(id, c) == c` holds. A single parameter would force
/// `fmap` to change the complex's precision too, which it cannot do without rebuilding the complex
/// and dropping its Hodge ⋆ operators.
///
/// Use [`UniformChain`] where both are the same type, which is the common case.
#[derive(Debug, Clone, PartialEq)]
pub struct Chain<R, G> {
    pub(crate) complex: Arc<SimplicialComplex<R>>,
    pub(crate) grade: usize,
    /// Sparse vector of active simplices.
    /// Reuses CsrMatrix logic (1 row, N cols) for efficient sparse operations.
    pub(crate) weights: CsrMatrix<G>,
}

/// A chain whose coefficients share the complex's precision type, which is the common case.
pub type UniformChain<T> = Chain<T, T>;

impl<R, G> Chain<R, G> {
    pub fn new(complex: Arc<SimplicialComplex<R>>, grade: usize, weights: CsrMatrix<G>) -> Self {
        Self {
            complex,
            grade,
            weights,
        }
    }
}

impl<R, G> Chain<R, G> {
    pub fn complex(&self) -> &Arc<SimplicialComplex<R>> {
        &self.complex
    }

    pub fn grade(&self) -> usize {
        self.grade
    }

    pub fn weights(&self) -> &CsrMatrix<G> {
        &self.weights
    }
}
