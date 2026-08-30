/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::homology_error::{HomologyError, HomologyErrorEnum};
use crate::types::homology_field::HomologyField;
use alloc::borrow::Cow;
use deep_causality_linear::CsrMatrix;

/// A chain complex: graded groups `C_k` with boundary operators `∂ₖ : C_k → C_{k−1}`.
///
/// # The law every implementor owes
///
/// **`∂ₖ ∘ ∂ₖ₊₁ = 0`.** Equivalently `im ∂ₖ₊₁ ⊆ ker ∂ₖ`, which is what makes the quotient
/// `H_k = ker ∂ₖ / im ∂ₖ₊₁` a group at all.
///
/// It cannot be checked by the trait, and nothing in this crate assumes it silently: every Betti
/// number [`betti_number_over`](Self::betti_number_over) returns is meaningless without it, because
/// the identity that method relies on — `dim H_k = (n_k − rank ∂ₖ) − rank ∂ₖ₊₁` — holds only when
/// the image sits inside the kernel. It is machine-checked in
/// `lean/DeepCausalityFormal/Homology/ChainCondition.lean` and asserted for every implementor in
/// this crate's conformance harness.
///
/// For a complex built out of cells, the law follows from each cell's boundary being a cycle: a
/// face of a face is dropped twice with opposite signs. An implementor deriving its operators from
/// a cell type inherits the obligation from that type.
///
/// # No geometry
///
/// The trait names no cell, no metric and no coordinate. A quantum error-correcting code is a chain
/// complex with none of those: `H_X` and `H_Z` are parity-check matrices whose product vanishes over
/// 𝔽₂. `deep_causality_topology::CellComplex` adds the geometric half for complexes that have one.
///
/// # Coefficients
///
/// The boundary matrices carry `i8` because their entries are incidence numbers, lying in
/// `{−1, 0, 1}` by construction. That is an invariant of the boundary operator rather than a storage
/// choice, so the trait takes no coefficient parameter. The coefficient *field* belongs to the
/// computation, and [`HomologyField`] carries it where the choice is made.
///
/// Cache-rich implementors return `Cow::Borrowed`; compute-on-demand implementors return
/// `Cow::Owned`.
pub trait ChainComplex {
    /// The number of `k`-cells, which is `dim C_k`.
    ///
    /// Named for the cells because that is what an implementor counts, and read as a dimension
    /// because that is what the homology needs.
    fn num_cells(&self, k: usize) -> usize;

    /// The largest grade carrying cells.
    fn max_dim(&self) -> usize;

    /// The boundary matrix `∂ₖ`, with `num_cells(k − 1)` rows and `num_cells(k)` columns.
    ///
    /// The degenerate grades carry the shape their dimension implies rather than an empty matrix:
    /// `∂₀` is `(0, n₀)` and `∂_{max+1}` is `(n_max, 0)`. That keeps `cols(∂ₖ) == rows(∂ₖ₊₁)` true
    /// at every grade, so the composite in the law above is always formable.
    fn boundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;

    /// The coboundary matrix `δₖ`, the transpose of `∂ₖ₊₁`.
    fn coboundary_matrix(&self, k: usize) -> Cow<'_, CsrMatrix<i8>>;

    /// The `k`-th Betti number over `field`: `β_k = dim H_k`, `H_k = ker ∂ₖ / im ∂ₖ₊₁`.
    ///
    /// # The field is an argument, and there is no other way to set it
    ///
    /// A boundary matrix has a different rank over ℚ than over 𝔽₂, so `β_k` is not a number until
    /// the field is named. Real projective space has `β₁ = 0` over ℚ and `β₁ = 1` over 𝔽₂, from the
    /// same complex. It is named here and nowhere else — not by a feature, a builder option or a
    /// global. [`betti_number`](Self::betti_number) is this method at [`HomologyField::Rational`]
    /// and is defined as such, so reading a call still tells you which field you are getting.
    ///
    /// # Rank–nullity, over whichever field
    ///
    /// `dim C_k = rank ∂ₖ + dim ker ∂ₖ` holds over any field, which is what lets one body serve
    /// both: `β_k = (n_k − rank ∂ₖ) − rank ∂ₖ₊₁`. No kernel is built and no quotient is formed; the
    /// substitution of `n_k − rank ∂ₖ` for `dim ker ∂ₖ` is rank–nullity, proved in
    /// `lean/DeepCausalityFormal/Linear/RankNullity.lean` as `linear.gf2.betti_from_ranks`.
    ///
    /// The saturating subtractions are a floor at zero for the grades where a complex has no cells,
    /// not a correction to the identity — `linear.gf2.rank_le_cell_count` shows the floor is never
    /// reached at that step. The grade step is checked rather than saturating. At `k == usize::MAX`
    /// there is no `∂ₖ₊₁` to form and its image is trivial, so the next-boundary rank is zero;
    /// saturating would read `∂_MAX` a second time and subtract its rank twice.
    ///
    /// # Errors
    ///
    /// [`HomologyErrorEnum::LinearAlgebraError`] if
    /// the exact elimination overflows. See [`HomologyField::rank_of`].
    fn betti_number_over(&self, k: usize, field: HomologyField) -> Result<usize, HomologyError> {
        let n_k = self.num_cells(k);
        let rank_k = field.rank_of(&self.boundary_matrix(k))?;
        // There is no grade past `usize::MAX`, so `im ∂ₖ₊₁` is trivial there and its rank is zero.
        // Saturating would re-read `∂_MAX` and subtract its rank a second time.
        let rank_k_next = match k.checked_add(1) {
            Some(next) => field.rank_of(&self.boundary_matrix(next))?,
            None => 0,
        };
        let dim_ker = n_k.saturating_sub(rank_k);
        Ok(dim_ker.saturating_sub(rank_k_next))
    }

    /// The `k`-th Betti number over ℚ.
    ///
    /// Exactly [`betti_number_over(k, HomologyField::Rational)`](Self::betti_number_over), and kept
    /// because it is the shape every caller of this trait already has. Rank over ℚ is rank over ℝ
    /// for an integer matrix, so this is computed exactly, with no tolerance anywhere on the path.
    ///
    /// # Panics
    ///
    /// If the exact elimination overflows, which the fallible form reports instead.
    fn betti_number(&self, k: usize) -> usize {
        match self.betti_number_over(k, HomologyField::Rational) {
            Ok(b) => b,
            // The overflow arm names its cause. It is the only failure this path can produce from a
            // well-formed complex, and "elimination overflow" read without that context has been
            // mistaken for a malformed boundary matrix.
            Err(HomologyError(HomologyErrorEnum::LinearAlgebraError(msg))) => {
                panic!("exact rank over the rationals failed at grade {k}: {msg}")
            }
            Err(e) => panic!("Betti number at grade {k} failed: {e}"),
        }
    }
}
