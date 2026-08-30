/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `CellularComplex` and `ChainComplex` impls for `SimplicialComplex`.
//!
//! Vends pre-computed boundary and coboundary matrices as `Cow::Borrowed` (zero copy).
//! `boundary_matrix(k)` borrows `&self.boundary_operators[k - 1]` where that operator
//! exists. The degenerate grades are returned owned, carrying the shape their dimension
//! implies rather than a shapeless empty matrix: `∂₀` is `0 × num_cells(0)`, and a grade
//! above the top is `num_cells(k - 1) × num_cells(k)`. That keeps
//! `cols(∂ₖ) == rows(∂ₖ₊₁)` at both ends.

use crate::traits::cell::Cell;
use crate::traits::cellular_complex::CellularComplex;
use crate::traits::chain_complex::ChainComplex;
use crate::{Simplex, SimplicialComplex};
use deep_causality_linear::CsrMatrix;
use std::borrow::Cow;
use std::iter::Cloned;
use std::slice::Iter;

/// Concrete cell iterator for `SimplicialComplex`'s `CellularComplex` impl.
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

impl<T: deep_causality_algebra::RealField> CellularComplex for SimplicialComplex<T> {
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
}

impl<T: deep_causality_algebra::RealField> ChainComplex for SimplicialComplex<T> {
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
        //
        // The degenerate grades carry the shape their dimension implies rather than an empty
        // matrix: `∂₀` has no rows and one column per vertex, and any grade above the top has one
        // row per cell below it and no columns. That keeps `cols(∂ₖ) == rows(∂ₖ₊₁)` at both ends,
        // so the composite the `∂∘∂ = 0` law speaks about is formable there.
        if k == 0 {
            return Cow::Owned(
                CsrMatrix::from_triplets(0, self.num_cells(0), &[])
                    .expect("an empty matrix of a stated shape"),
            );
        }
        match self.boundary_operators.get(k - 1) {
            Some(m) => Cow::Borrowed(m),
            None => Cow::Owned(
                CsrMatrix::from_triplets(self.num_cells(k - 1), self.num_cells(k), &[])
                    .expect("an empty matrix of a stated shape"),
            ),
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
