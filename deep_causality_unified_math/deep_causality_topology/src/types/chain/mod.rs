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
/// They used to be one parameter. `fmap` maps the coefficients, and with a single `T` that forced it
/// to change the complex's precision as well, which it could not do meaningfully: it rebuilt the
/// complex with `..Default::default()` and dropped the Hodge ⋆ operators, so `fmap(id, c)` was not
/// `c`. Separating them lets `fmap` clone the complex, and the functor identity law holds.
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
