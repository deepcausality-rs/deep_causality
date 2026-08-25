/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `ChainComplex` impl for `SimplicialComplex`.
//!
//! Vends pre-computed boundary and coboundary matrices as `Cow::Borrowed` (zero copy).
//! `boundary_matrix(k)` returns `&self.boundary_operators[k - 1]` for `k > 0`; an empty
//! matrix is returned for `k == 0` (consistent with `boundary_operator(0)` returning
//! `DimensionMismatch` today).

use crate::traits::cell::Cell;
use crate::traits::chain_complex::ChainComplex;
use crate::{Simplex, SimplicialComplex};
use deep_causality_linear::CsrMatrix;
use std::borrow::Cow;
use std::iter::Cloned;
use std::slice::Iter;

/// Concrete cell iterator for `SimplicialComplex`'s `ChainComplex` impl.
/// Wraps `Cloned<Iter<'a, Simplex>>` over the grade-`k` skeleton or returns nothing
/// when no skeleton exists at the requested grade.
pub struct SimplicialCellIter<'a> {
    inner: Option<Cloned<Iter<'a, Simplex>>>,
}

impl<'a> Iterator for SimplicialCellIter<'a> {
    type Item = Simplex;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.as_mut()?.next()
    }
}

impl<T: deep_causality_algebra::RealField> ChainComplex for SimplicialComplex<T> {
    type CellType = Simplex;
    type CellIter<'a>
        = SimplicialCellIter<'a>
    where
        Self: 'a;
    type Metric = crate::ReggeGeometry<T>;

    fn cells(&self, k: usize) -> Self::CellIter<'_> {
        let inner = self
            .skeletons
            .iter()
            .find(|s| s.dim == k)
            .map(|s| s.simplices.iter().cloned());
        SimplicialCellIter { inner }
    }

    fn num_cells(&self, k: usize) -> usize {
        self.skeletons
            .iter()
            .find(|s| s.dim == k)
            .map(|s| s.simplices.len())
            .unwrap_or(0)
    }

    fn max_dim(&self) -> usize {
        self.skeletons.iter().map(|s| s.dim).max().unwrap_or(0)
    }

    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        // Existing storage: boundary_operators[k - 1] holds ∂_k.
        if k == 0 {
            return Cow::Owned(CsrMatrix::new());
        }
        match self.boundary_operators.get(k - 1) {
            Some(m) => Cow::Borrowed(m),
            None => Cow::Owned(CsrMatrix::new()),
        }
    }

    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        match self.coboundary_operators.get(k) {
            Some(m) => Cow::Borrowed(m),
            None => Cow::Owned(CsrMatrix::new()),
        }
    }

}


// Touch unused trait import to satisfy lint when only the trait bound matters.
const _: fn() = || {
    fn _assert<C: Cell>() {}
    _assert::<Simplex>();
};
