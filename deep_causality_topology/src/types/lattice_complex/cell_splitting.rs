/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cubical splitting of a lattice cell.

use crate::traits::cell_splitting::{CellLayout, CellSplit, SplittableCell};
use crate::types::lattice_complex::lattice_cell::LatticeCell;

impl<const D: usize> SplittableCell for LatticeCell<D> {
    /// The cubical decomposition. For a `k`-cell with active axis set `A` and
    /// base position `p`, one term per subset `S_α ⊆ A` with `|S_α| = left_dim`,
    /// writing `S_β = A \ S_α`:
    ///
    /// - left cell: `{ position: p, directions: S_α }`
    /// - right cell: `{ position: p + Σ_{j ∈ S_α} e_j, directions: S_β }`
    /// - sign: `sgn(S_α ascending, then S_β ascending)`, the shuffle sign
    ///
    /// The right cell's position is wrapped per axis where `layout` marks the
    /// axis periodic, so a cell at the far edge of a torus pairs with the one
    /// that has wrapped around.
    ///
    /// In two dimensions this yields
    /// `+ (bottom x-edge, right y-edge) − (left y-edge, top x-edge)`, which
    /// reduces mod 2 to `α(□₀₁)β(□₁₃) + α(□₀₂)β(□₂₃)` of Chen & Tata
    /// (arXiv:2106.05274) Fig. 1.
    ///
    /// **Convention.** The left cell takes the leading directions from the base
    /// position and the right cell begins where the left one ends, in direct
    /// analogy with Alexander–Whitney. Chen & Tata's Fig. 4 and Definition 1
    /// use the mirror convention, placing the offset on the left factor. Both
    /// satisfy the Leibniz rule against this crate's coboundary operators and
    /// both give identical cohomology pairings, so this is a convention rather
    /// than the only possibility.
    fn split(&self, _left_dim: usize, _layout: Option<&CellLayout>) -> Vec<CellSplit<Self>> {
        todo!("cell-splitting: cubical split of a lattice cell")
    }
}
