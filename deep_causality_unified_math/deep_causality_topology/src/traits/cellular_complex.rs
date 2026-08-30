/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::cell::Cell;
use deep_causality_homology::ChainComplex;

/// A chain complex whose chain groups are spanned by geometric cells.
///
/// # What this adds, and what it inherits
///
/// [`ChainComplex`], in `deep_causality_homology`, carries the homology: cell counts, boundary and
/// coboundary matrices, and Betti numbers over a chosen field. None of that mentions geometry, and
/// a quantum error-correcting code is a chain complex with no cells at all.
///
/// This trait adds the half that does mention geometry — the cells themselves, their type, and the
/// metric the complex carries. A complex implements both; a caller that needs only homology depends
/// only on the crate below.
///
/// # Why the name is not `CellComplex`
///
/// [`CellComplex<C>`](crate::CellComplex) is a struct in this crate, and Rust puts types and traits
/// in one namespace. `CellularComplex` is the term of art for what this describes — a CW structure
/// on a chain complex — and it leaves that published type alone.
pub trait CellularComplex: ChainComplex {
    /// The type of cells in this complex.
    type CellType: Cell;

    /// The concrete iterator type returned by [`cells`](Self::cells).
    type CellIter<'a>: Iterator<Item = Self::CellType>
    where
        Self: 'a;

    /// The metric type associated with this complex.
    ///
    /// Precision-carrying complexes (e.g. `SimplicialComplex<R: RealField>`,
    /// `LatticeComplex<const D, R: RealField>`) bind this to a concrete metric type
    /// at their own `R`: `type Metric = ReggeGeometry<R>;`, `type Metric =
    /// CubicalReggeGeometry<D, R>;`. The combinatorial `CellComplex<C>` has no metric
    /// and binds `type Metric = ();`. The metric precision flows from the complex's
    /// own type parameters, not from a generic argument on this associated type.
    ///
    /// See `design.md` Decision 1 of `generalize-topology-over-realfield` for the
    /// rationale for picking a plain associated type over a GAT.
    type Metric;

    /// Iterate over all `k`-cells in the complex.
    fn cells(&self, k: usize) -> Self::CellIter<'_>;

    /// Layout of a uniform axis-aligned lattice, if this complex is one:
    /// `(shape, periodic)` per axis, in the complex's own axis order.
    ///
    /// Defaults to `None`. The cubical lattice overrides it; the spectral
    /// grade-0 Poisson fast path consumes it to decide whether the
    /// Laplacian diagonalizes under the DFT (it does exactly when every
    /// axis is periodic).
    fn uniform_lattice_layout(&self) -> Option<(Vec<usize>, Vec<bool>)> {
        None
    }
}
