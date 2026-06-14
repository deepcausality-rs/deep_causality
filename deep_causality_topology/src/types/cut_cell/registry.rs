/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Sparse registry of cut cells over a lattice, plus the cut-aware volume / dual-clip
//! accessors that ride the existing `CubicalReggeGeometry` dispatch.

use super::carrier::CutCell;
use crate::traits::neighborhood::CellId;
use crate::types::cubical_regge_geometry::CubicalReggeGeometry;
use crate::types::cubical_regge_geometry::SignatureMarker;
use crate::types::lattice_complex::{LatticeCell, LatticeComplex};
use deep_causality_num::RealField;
use std::collections::HashMap;

/// A sparse map from a top `D`-cell's index (in the lattice's `iter_cells(D)` ordering) to
/// its [`CutCell`] overlay.
///
/// Only cells the immersed surface intersects (cut) or that fall inside the body (solid)
/// are stored; every top cell absent from the registry is a full fluid cell on the existing
/// uniform fast path. The registry is therefore sized to the boundary, not the volume
/// (design D1). It carries no metric of its own — the accessors take the geometry and
/// lattice so a single registry composes with any `CubicalReggeGeometry` tier (unit,
/// uniform, per-axis, graded `PerEdge`).
#[derive(Debug, Clone, PartialEq)]
pub struct CutCellRegistry<const D: usize, R: RealField> {
    cells: HashMap<CellId, CutCell<D, R>>,
}

impl<const D: usize, R: RealField> Default for CutCellRegistry<D, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const D: usize, R: RealField> CutCellRegistry<D, R> {
    /// An empty registry — every cell is full fluid.
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
        }
    }

    /// Build a registry from a precomputed `top-cell-index -> CutCell` map.
    pub fn from_map(cells: HashMap<CellId, CutCell<D, R>>) -> Self {
        Self { cells }
    }

    /// Record `cut` for the top cell at index `cell_id` (in `iter_cells(D)` ordering),
    /// returning any previous entry.
    pub fn insert(&mut self, cell_id: CellId, cut: CutCell<D, R>) -> Option<CutCell<D, R>> {
        self.cells.insert(cell_id, cut)
    }

    /// Number of recorded (cut + solid) cells. Bounded by the immersed-boundary size.
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    /// `true` iff no cells are recorded (the whole domain is fluid).
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The recorded [`CutCell`] for the top cell at index `cell_id`, or `None` (full fluid).
    pub fn get(&self, cell_id: CellId) -> Option<&CutCell<D, R>> {
        self.cells.get(&cell_id)
    }

    /// Iterate over `(top-cell-index, &CutCell)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&CellId, &CutCell<D, R>)> {
        self.cells.iter()
    }
}

impl<const D: usize, R: RealField> CutCellRegistry<D, R> {
    /// Cut-aware `k`-cell volume (A3). For a registered top `D`-cell this returns the
    /// clipped fluid volume; for every other cell it falls through to the geometry's
    /// existing `cell_volume` fast path. This is the volume override the Hodge-star
    /// dispatch consumes — the fractional-aperture generalisation of the Stage-3 wall clip.
    pub fn clipped_cell_volume<S: SignatureMarker>(
        &self,
        geom: &CubicalReggeGeometry<D, R, S>,
        complex: &LatticeComplex<D, R>,
        cell: &LatticeCell<D>,
    ) -> R {
        if cell.cell_dim() == D
            && let Some(idx) = complex.cell_index(cell)
            && let Some(cut) = self.cells.get(&idx)
        {
            return cut.fluid_volume();
        }
        geom.cell_volume(complex, cell)
    }

    /// The fluid fraction of a top cell's volume in `[0, 1]`: the registered cut/solid
    /// fraction, or `1` for an unregistered fluid cell. The continuous generalisation of
    /// "is this cell inside the domain" that the dual clip averages over.
    pub fn top_cell_fluid_fraction(
        &self,
        complex: &LatticeComplex<D, R>,
        top_base: [usize; D],
    ) -> R {
        let all_axes_mask: u32 = if D >= 32 { u32::MAX } else { (1u32 << D) - 1 };
        let cell = LatticeCell::new(top_base, all_axes_mask);
        match complex
            .cell_index(&cell)
            .and_then(|idx| self.cells.get(&idx))
        {
            Some(cut) => cut.volume_fraction(),
            None => R::one(),
        }
    }

    /// Cut-aware dual clip factor for a `k`-cell (A3, A6): the fraction of the cell's dual
    /// `(D−k)`-cell that lies in the fluid, in `[0, 1]`.
    ///
    /// The dual of a primal `k`-cell is assembled from the `2^{D−k}` top cubes incident to
    /// it (one per choice of `±` side along each complement axis), exactly as in the
    /// per-edge Hodge-star dual construction. This averages those incident cubes' fluid
    /// fractions over the in-bounds corners. It **generalises the Stage-3 integer boundary
    /// clip**: when the immersed surface is an axis-aligned plane coincident with a wall,
    /// the outside cubes are solid (fraction `0`) and the inside cubes fluid (fraction `1`),
    /// so a `b`-fold wall incidence yields exactly `2^{-b}` — the `boundary_clip` value. On
    /// an empty registry every cube is fluid and the factor is `1` (no clip).
    pub fn dual_fluid_fraction(&self, complex: &LatticeComplex<D, R>, cell: &LatticeCell<D>) -> R {
        let orientation = cell.orientation();
        let position = *cell.position();
        let complement_axes: Vec<usize> = (0..D)
            .filter(|&a| (orientation & (1u32 << a)) == 0)
            .collect();
        let num_complement = complement_axes.len();
        let num_masks = 1u32 << num_complement;

        let shape = complex.shape();
        let periodic = complex.periodic();

        // Sum the fluid fraction over ALL 2^{D−k} dual corners; an out-of-bounds corner on
        // an open axis is outside the domain (a wall incidence) and contributes 0. Dividing
        // by the full corner count — not the survivor count — is what reproduces the integer
        // 2^{-b} boundary clip: b walls drop 2^{D−k}·(1 − 2^{-b}) corners to zero.
        let mut sum = R::zero();
        for mask_bits in 0..num_masks {
            // Resolve the base position of the incident top cube for this corner.
            let mut base = position;
            let mut in_bounds = true;
            for (bit_idx, &axis) in complement_axes.iter().enumerate() {
                let dim_len = shape[axis];
                if dim_len == 0 {
                    in_bounds = false;
                    break;
                }
                // Top cubes have every axis active: valid base range is 0..dim_len-1 on an
                // open axis, 0..dim_len (with wrap) on a periodic axis.
                let m_c = (mask_bits >> bit_idx) & 1;
                let b = if m_c == 0 {
                    // Cube on the + side: base = position[axis].
                    if periodic[axis] {
                        position[axis] % dim_len
                    } else if position[axis] >= dim_len - 1 {
                        in_bounds = false;
                        break;
                    } else {
                        position[axis]
                    }
                } else if position[axis] == 0 {
                    // Cube on the − side: base = position[axis] − 1.
                    if periodic[axis] {
                        dim_len - 1
                    } else {
                        in_bounds = false;
                        break;
                    }
                } else {
                    position[axis] - 1
                };
                base[axis] = b;
            }
            if in_bounds {
                sum += self.top_cell_fluid_fraction(complex, base);
            }
        }

        let mut divisor = R::zero();
        for _ in 0..num_masks {
            divisor += R::one();
        }
        sum / divisor
    }
}
