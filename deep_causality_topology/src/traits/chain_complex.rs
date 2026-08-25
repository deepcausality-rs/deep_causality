/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::cell::Cell;
use crate::types::homology_field::HomologyField;
use crate::{TopologyError, TopologyErrorEnum};
use deep_causality_linear::CsrMatrix;
use std::borrow::Cow;

/// Types that form a chain complex (a CW-style cellular decomposition).
/// This generalizes simplicial complexes, cubical lattices, and arbitrary cellular decompositions.
///
/// The trait uses static dispatch end-to-end: cell iteration is exposed via a GAT-backed
/// `CellIter<'_>` and the boundary / coboundary matrices return `Cow<'_, CsrMatrix<i8>>`
/// so cache-rich implementors can vend `Cow::Borrowed` (zero copy) while compute-on-demand
/// implementors return `Cow::Owned`.
pub trait ChainComplex {
    /// The type of cells in this complex.
    type CellType: Cell;

    /// The concrete iterator type returned by `cells`.
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

    /// Iterate over all k-cells in the complex.
    fn cells(&self, k: usize) -> Self::CellIter<'_>;

    /// Get the total number of k-cells.
    fn num_cells(&self, k: usize) -> usize;

    /// The maximum dimension of cells in the complex.
    fn max_dim(&self) -> usize;

    /// Return the boundary matrix ∂_k as a sparse matrix.
    /// Rows correspond to (k-1)-cells, columns to k-cells.
    ///
    /// Cache-rich implementors return `Cow::Borrowed`. Compute-on-demand implementors
    /// return `Cow::Owned`.
    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;

    /// Return the coboundary matrix δ_k as a sparse matrix.
    /// δ_k is the transpose of ∂_{k+1}.
    ///
    /// Cache-rich implementors return `Cow::Borrowed`. Compute-on-demand implementors
    /// return `Cow::Owned`.
    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;

    /// The k-th Betti number over `field`: `β_k = dim(H_k)`, `H_k = ker(∂_k) / im(∂_{k+1})`.
    ///
    /// # The field is an argument, and there is no other way to set it
    ///
    /// A boundary matrix has a different rank over ℚ than over 𝔽₂, so `β_k` is not a number until
    /// the field is named. It is named here and nowhere else — not by a feature, a builder option
    /// or a global. [`betti_number`](ChainComplex::betti_number) is this method at
    /// [`HomologyField::Rational`] and is defined as such, so reading a call still tells you which
    /// field you are getting.
    ///
    /// # Rank–nullity, over whichever field
    ///
    /// `dim C_k = rank ∂_k + dim ker ∂_k` holds over any field, which is what lets one body serve
    /// both: `β_k = (n_k − rank ∂_k) − rank ∂_{k+1}`. The saturating subtractions are a floor at
    /// zero for the grades where a complex has no cells, not a correction to the identity.
    ///
    /// # Errors
    ///
    /// [`TopologyErrorEnum::LinearAlgebraError`] if the exact elimination overflows. See
    /// [`HomologyField::rank_of`].
    fn betti_number_over(&self, k: usize, field: HomologyField) -> Result<usize, TopologyError> {
        let n_k = self.num_cells(k);
        let rank_k = field.rank_of(&self.boundary_matrix(k))?;
        let rank_k_next = field.rank_of(&self.boundary_matrix(k + 1))?;
        let dim_ker = n_k.saturating_sub(rank_k);
        Ok(dim_ker.saturating_sub(rank_k_next))
    }

    /// The k-th Betti number over ℚ.
    ///
    /// Exactly [`betti_number_over(k, HomologyField::Rational)`](ChainComplex::betti_number_over),
    /// and kept because it is the shape every caller of this trait already has. Rank over ℚ is
    /// rank over ℝ for an integer matrix, so this is the number the retired thresholded-SVD path
    /// was approximating — now computed exactly, with no tolerance anywhere on the path.
    ///
    /// # Panics
    ///
    /// If the exact elimination overflows, which the fallible form reports instead. The retired
    /// path panicked here too, on `expect("SVD failed")`; this one at least names the cause.
    fn betti_number(&self, k: usize) -> usize {
        match self.betti_number_over(k, HomologyField::Rational) {
            Ok(b) => b,
            Err(TopologyError(TopologyErrorEnum::LinearAlgebraError(msg))) => {
                panic!("exact rank over the rationals failed at grade {k}: {msg}")
            }
            Err(e) => panic!("Betti number at grade {k} failed: {e}"),
        }
    }

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
