/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GaugeGroup, LatticeCell, LatticeGaugeField, LinkVariable};
use deep_causality_algebra::Field;
use deep_causality_algebra::{ComplexField, DivisionAlgebra, RealField};
use deep_causality_num::{FromPrimitive, ToPrimitive};
use std::fmt::Debug;

impl<
    G: GaugeGroup,
    const D: usize,
    M: Field + Copy + Default + PartialOrd + Debug + ComplexField<R> + DivisionAlgebra<R>,
    R: RealField + FromPrimitive + ToPrimitive,
    S,
> LatticeGaugeField<G, D, M, R, S>
{
    /// Get a link, returning identity if not found.
    pub(crate) fn get_link_or_identity(&self, edge: &LatticeCell<D>) -> LinkVariable<G, M, R> {
        self.link(edge)
            .cloned()
            .unwrap_or_else(LinkVariable::identity)
    }
}

// ============================================================================
// Flat link indexing
//
// A 1-cell is a base position plus an orientation with exactly one bit set, which maps to
// `site_offset * D + mu`, `site_offset` being row-major over the lattice shape. The index is
// O(1) and needs no hashing, and storing links in it makes iteration ordered.
// ============================================================================

/// The flat index of an edge, or `None` if it is not a 1-cell of this lattice.
#[inline]
pub(crate) fn link_index<const D: usize>(
    shape: &[usize; D],
    edge: &LatticeCell<D>,
) -> Option<usize> {
    // A link extends in exactly one dimension. Anything else is a cell of another grade.
    if edge.orientation().count_ones() != 1 {
        return None;
    }
    let mu = edge.orientation().trailing_zeros() as usize;
    if mu >= D {
        return None;
    }

    let pos = edge.position();
    let mut offset = 0usize;
    for d in 0..D {
        if pos[d] >= shape[d] {
            return None;
        }
        offset = offset * shape[d] + pos[d];
    }
    Some(offset * D + mu)
}

/// The edge a flat index names. Inverse of [`link_index`] on the valid range.
#[inline]
pub(crate) fn link_cell<const D: usize>(shape: &[usize; D], index: usize) -> LatticeCell<D> {
    let mu = index % D;
    let mut site = index / D;
    let mut position = [0usize; D];
    for d in (0..D).rev() {
        position[d] = site % shape[d];
        site /= shape[d];
    }
    LatticeCell::edge(position, mu)
}

/// The number of slots a lattice of this shape needs: one per site per direction.
#[inline]
pub(crate) fn link_slots<const D: usize>(shape: &[usize; D]) -> usize {
    shape.iter().product::<usize>() * D
}

/// An empty slot table sized for this lattice.
#[inline]
pub(crate) fn alloc_slots<const D: usize, G: GaugeGroup, M, R: RealField>(
    shape: &[usize; D],
) -> Vec<Option<LinkVariable<G, M, R>>> {
    let mut v = Vec::new();
    v.resize_with(link_slots(shape), || None);
    v
}

/// Fold a caller-supplied edge map into the flat table. The map's own iteration order cannot
/// matter, because every entry is written to the slot its edge names.
#[inline]
pub(crate) fn slots_from_map<const D: usize, G: GaugeGroup, M, R: RealField>(
    shape: &[usize; D],
    map: std::collections::HashMap<LatticeCell<D>, LinkVariable<G, M, R>>,
) -> Vec<Option<LinkVariable<G, M, R>>> {
    let mut slots = alloc_slots(shape);
    for (cell, link) in map {
        if let Some(i) = link_index(shape, &cell) {
            slots[i] = Some(link);
        }
    }
    slots
}
