/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Alexander–Whitney splitting of a simplex.

use crate::traits::cell_splitting::{CellLayout, CellSplit, SplittableCell};
use crate::types::simplex::Simplex;

impl SplittableCell for Simplex {
    /// The Alexander–Whitney decomposition: a single term whose left cell is the
    /// leading vertices `(0 → p)` and whose right cell is the trailing vertices
    /// `(p → p+q)`, sharing the vertex at position `p`, with sign `+1`.
    ///
    /// This is Chen & Tata (arXiv:2106.05274) Eq. (5):
    /// `(α_p ∪ β_q)(0,…,p+q) = α_p(0 → p) · β_q(p → p+q)`.
    ///
    /// The decomposition depends on the simplex's vertices being strictly
    /// increasing, which [`Simplex::vertices`] guarantees. `layout` is unused:
    /// a simplicial splitting is purely combinatorial.
    fn split(&self, _left_dim: usize, _layout: Option<&CellLayout>) -> Vec<CellSplit<Self>> {
        todo!("cell-splitting: Alexander-Whitney split of a simplex")
    }
}
