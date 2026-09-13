/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::borrow::Cow;
use alloc::vec::Vec;
use deep_causality_homology::ChainComplex;
use deep_causality_linear::CsrMatrix;

/// A chain complex given by its boundary matrices, for small code fixtures no lattice produces.
///
/// `boundaries[k]` is `∂ₖ`, with `num_cells(k − 1)` rows and `num_cells(k)` columns; `∂₀` has no
/// rows. Grades past the top have no cells and their boundary is the empty matrix of the shape the
/// dimension implies, so `∂ₖ ∂ₖ₊₁` is always formable.
#[derive(Debug, Clone, PartialEq)]
pub struct HandBuiltComplex {
    cells: Vec<usize>,
    boundaries: Vec<CsrMatrix<i8>>,
}

impl HandBuiltComplex {
    /// A complex from its cell counts per grade and its boundary matrices `∂₁, …, ∂_top` as
    /// `(row, column, coefficient)` triplets. Panics on a malformed matrix or when some
    /// `∂ₖ ∂ₖ₊₁` is not zero, because a fixture that is not a chain complex is a defect in the
    /// test and not a run-time condition.
    pub fn new(cells: Vec<usize>, boundaries: &[&[(usize, usize, i8)]]) -> Self {
        assert_eq!(
            boundaries.len() + 1,
            cells.len(),
            "one boundary matrix per grade above zero"
        );
        let mut mats = Vec::with_capacity(cells.len());
        mats.push(CsrMatrix::from_triplets(0, cells[0], &[]).expect("∂₀ has no rows"));
        for (k, triplets) in boundaries.iter().enumerate() {
            let m = CsrMatrix::from_triplets(cells[k], cells[k + 1], triplets)
                .unwrap_or_else(|e| panic!("∂_{} is not well formed: {e}", k + 1));
            mats.push(m);
        }
        for k in 1..mats.len().saturating_sub(1) {
            let product = mats[k]
                .mat_mult(&mats[k + 1])
                .unwrap_or_else(|e| panic!("∂_{k} ∂_{} cannot be formed: {e}", k + 1));
            if product.values().iter().any(|v| *v != 0) {
                panic!(
                    "∂_{k} ∂_{} is not zero; the fixture is not a chain complex",
                    k + 1
                );
            }
        }
        Self {
            cells,
            boundaries: mats,
        }
    }
}

impl ChainComplex for HandBuiltComplex {
    fn num_cells(&self, k: usize) -> usize {
        self.cells.get(k).copied().unwrap_or(0)
    }

    fn max_dim(&self) -> usize {
        self.cells.len().saturating_sub(1)
    }

    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        match self.boundaries.get(k) {
            Some(m) => Cow::Borrowed(m),
            None => {
                let rows = if k == 0 { 0 } else { self.num_cells(k - 1) };
                Cow::Owned(CsrMatrix::from_triplets(rows, 0, &[]).expect("an empty matrix"))
            }
        }
    }

    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>> {
        match k.checked_add(1) {
            Some(next) => Cow::Owned(self.boundary_matrix(next).transpose()),
            None => Cow::Owned(
                CsrMatrix::from_triplets(0, self.num_cells(k), &[]).expect("an empty matrix"),
            ),
        }
    }
}
