/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The cubical splitting of a lattice cell.

use crate::traits::cell_splitting::{CellLayout, CellSplit, SplittableCell};
use crate::types::lattice_complex::lattice_cell::LatticeCell;

/// The orientation bitmask for a set of axis indices.
fn axis_mask(axes: &[usize]) -> u32 {
    axes.iter().fold(0u32, |m, &a| m | (1 << a))
}

/// `sgn` of the permutation carrying the cell's ascending axes to
/// `(S_alpha ascending, then S_beta ascending)`, counted as inversions.
fn shuffle_sign(left: &[usize], right: &[usize]) -> i8 {
    let inversions: usize = left
        .iter()
        .map(|&i| right.iter().filter(|&&j| j < i).count())
        .sum();
    if inversions.is_multiple_of(2) { 1 } else { -1 }
}

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
    /// that has wrapped around. Terms are produced in ascending subset-bitmask
    /// order, so the enumeration is deterministic.
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
    fn split(&self, left_dim: usize, layout: Option<&CellLayout>) -> Vec<CellSplit<Self>> {
        let active: Vec<usize> = (0..D)
            .filter(|i| self.orientation() & (1 << i) != 0)
            .collect();
        let k = active.len();
        if left_dim > k {
            return Vec::new();
        }

        let base = *self.position();
        let mut terms = Vec::new();
        for subset in 0u32..(1 << k) {
            if subset.count_ones() as usize != left_dim {
                continue;
            }
            let left_axes: Vec<usize> = (0..k)
                .filter(|&b| subset & (1 << b) != 0)
                .map(|b| active[b])
                .collect();
            let right_axes: Vec<usize> = (0..k)
                .filter(|&b| subset & (1 << b) == 0)
                .map(|b| active[b])
                .collect();

            // The right cell begins where the left one ends.
            let mut right_pos = base;
            for &axis in &left_axes {
                right_pos[axis] += 1;
                // Wrap only where the ambient layout marks the axis periodic.
                if let Some((shape, periodic)) = layout
                    && periodic.get(axis).copied().unwrap_or(false)
                    && let Some(&extent) = shape.get(axis)
                    && extent > 0
                {
                    right_pos[axis] %= extent;
                }
            }

            terms.push(CellSplit::new(
                LatticeCell::new(base, axis_mask(&left_axes)),
                LatticeCell::new(right_pos, axis_mask(&right_axes)),
                shuffle_sign(&left_axes, &right_axes),
            ));
        }
        terms
    }
}
