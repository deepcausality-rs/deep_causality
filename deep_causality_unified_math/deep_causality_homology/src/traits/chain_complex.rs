/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::homology_error::{HomologyError, HomologyErrorEnum};
use crate::types::gf2_chain::Gf2Chain;
use crate::types::homology_field::HomologyField;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use deep_causality_linear::{
    CsrMatrix, MatrixBuild, MatrixView, PackedGf2, csr_to_packed_gf2_mod2, image_basis_gf2,
    kernel_basis_gf2,
};
use deep_causality_num::{Gf2, NaturalNumber};

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

    /// A basis of `H_k = ker ∂ₖ / im ∂ₖ₊₁` over 𝔽₂, as chains of degree `k`.
    ///
    /// Where [`betti_number_over`](Self::betti_number_over) returns the dimension, this returns
    /// vectors: each is a cycle (`∂ₖ γ = 0`) and no 𝔽₂ combination of them is a boundary, so they
    /// descend to a basis of the quotient. The length of the result is `β_k` over 𝔽₂.
    ///
    /// # Over 𝔽₂ only, and why there is no field parameter
    ///
    /// A representative is a vector, and a vector over ℚ is not a vector over 𝔽₂. The bit-packed
    /// path is the one that has a chain type to return, so this method names its field in its
    /// return type rather than in an argument. Compare `betti_number_over`, where both fields yield
    /// the same kind of answer and the choice belongs in a parameter.
    ///
    /// # The basis is not canonical
    ///
    /// Any two bases of the quotient are related by an invertible change of basis, so which
    /// representatives come back is an artifact of the elimination order. What is fixed, and what
    /// callers may rely on, is the count, the cycle condition, and independence modulo boundaries.
    ///
    /// # How the quotient is taken
    ///
    /// `im ∂ₖ₊₁ ⊆ ker ∂ₖ` by the trait's law, so a basis of the quotient is any subset of a kernel
    /// basis that stays independent after the image is adjoined. The columns of `[B | Z]` are
    /// reduced, with the image basis `B` first, and the kernel columns holding a pivot are kept:
    /// a pivot beyond `B`'s columns is exactly a kernel vector no earlier column reproduces. The
    /// count that falls out is `rank[B | Z] − rank B`, which is `dim ker ∂ₖ − rank ∂ₖ₊₁`, which is
    /// `β_k`.
    ///
    /// # Errors
    ///
    /// [`HomologyErrorEnum::LinearAlgebraError`] if the 𝔽₂ elimination fails.
    fn homology_representatives<W: NaturalNumber>(
        &self,
        k: usize,
    ) -> Result<Vec<Gf2Chain<W>>, HomologyError> {
        let cycles = csr_to_packed_gf2_mod2::<W>(&self.boundary_matrix(k));
        // There is no grade past `usize::MAX`, so `im ∂ₖ₊₁` is trivial there, matching the
        // reasoning in `betti_number_over`.
        let boundaries: Option<PackedGf2<W>> = k
            .checked_add(1)
            .map(|next| csr_to_packed_gf2_mod2::<W>(&self.boundary_matrix(next)));
        quotient_basis(self.num_cells(k), k, &cycles, boundaries.as_ref())
    }

    /// A basis of `H^k = ker δₖ / im δₖ₋₁` over 𝔽₂, as chains of degree `k`.
    ///
    /// The dual of [`homology_representatives`](Self::homology_representatives), built from
    /// [`coboundary_matrix`](Self::coboundary_matrix) the same way. At `k == 0` there is no `δ₋₁`,
    /// so every cocycle is a class and the image is empty.
    ///
    /// The chains are degree-`k` because a `k`-cochain and a `k`-chain are indexed by the same
    /// cells; the pairing between them is [`Gf2Chain::inner`], which is what makes
    /// [`dual_representative`](Self::dual_representative) expressible.
    ///
    /// # Errors
    ///
    /// As [`homology_representatives`](Self::homology_representatives).
    fn cohomology_representatives<W: NaturalNumber>(
        &self,
        k: usize,
    ) -> Result<Vec<Gf2Chain<W>>, HomologyError> {
        let cocycles = csr_to_packed_gf2_mod2::<W>(&self.coboundary_matrix(k));
        let coboundaries: Option<PackedGf2<W>> = k
            .checked_sub(1)
            .map(|prev| csr_to_packed_gf2_mod2::<W>(&self.coboundary_matrix(prev)));
        quotient_basis(self.num_cells(k), k, &cocycles, coboundaries.as_ref())
    }

    /// A cohomology class pairing to one with `gamma`: some `γ̃ ∈ H^k` with `⟨γ, γ̃⟩ = 1`.
    ///
    /// `None` when `gamma` pairs to zero with every class, which is what happens when it is itself a
    /// boundary, or when the pairing is degenerate on it.
    ///
    /// # This needs no linear solve
    ///
    /// The pairing is 𝔽₂-linear in its second argument, so for `γ̃ = Σ aᵢ cᵢ` over a cohomology
    /// basis, `⟨γ, γ̃⟩ = Σ aᵢ⟨γ, cᵢ⟩`. That sum is one exactly when an odd number of the `⟨γ, cᵢ⟩`
    /// are one, and in particular it is achievable at all exactly when *some* `⟨γ, cᵢ⟩` is one. So
    /// scanning the basis both decides the question and produces the witness; there is no system to
    /// solve. The returned class is a single basis element rather than a combination, which is the
    /// simplest witness and not the only one.
    ///
    /// # Errors
    ///
    /// As [`cohomology_representatives`](Self::cohomology_representatives), plus
    /// [`HomologyErrorEnum::ChainGroupMismatch`] if `gamma` is not a degree-`k` chain over this
    /// complex's `k`-cells.
    fn dual_representative<W: NaturalNumber>(
        &self,
        gamma: &Gf2Chain<W>,
        k: usize,
    ) -> Result<Option<Gf2Chain<W>>, HomologyError> {
        for class in self.cohomology_representatives::<W>(k)? {
            if gamma.inner(&class)? == Gf2::ONE {
                return Ok(Some(class));
            }
        }
        Ok(None)
    }
}

/// A basis of `ker / im` given the two operators, as degree-`degree` chains over `n` cells.
///
/// `cycles` is the operator whose kernel is the numerator; `boundaries` is the one whose image is
/// the denominator, absent where the grade has none. Shared by
/// [`ChainComplex::homology_representatives`] and
/// [`ChainComplex::cohomology_representatives`], which differ only in which pair they hand it.
fn quotient_basis<W: NaturalNumber>(
    n: usize,
    degree: usize,
    cycles: &PackedGf2<W>,
    boundaries: Option<&PackedGf2<W>>,
) -> Result<Vec<Gf2Chain<W>>, HomologyError> {
    let kernel = kernel_basis_gf2(cycles).map_err(HomologyError::from)?;
    let image = match boundaries {
        Some(b) => image_basis_gf2(b).map_err(HomologyError::from)?,
        None => PackedGf2::<W>::zeros(n, 0),
    };

    let n_image = image.cols();
    let n_kernel = kernel.cols();
    if n_kernel == 0 {
        return Ok(Vec::new());
    }

    // `[B | Z]`: the image columns first, so a pivot at or beyond `n_image` names a kernel vector
    // that the image and the earlier kernel vectors together do not reproduce. There is no
    // column-concatenation primitive in `deep_causality_linear`, so the block is built by hand.
    let mut joined = PackedGf2::<W>::zeros(n, n_image + n_kernel);
    for c in 0..n_image {
        for r in 0..n {
            let v = image.get(r, c).map_err(HomologyError::from)?;
            if v.bit() {
                joined.set(r, c, Gf2::ONE).map_err(HomologyError::from)?;
            }
        }
    }
    for c in 0..n_kernel {
        for r in 0..n {
            let v = kernel.get(r, c).map_err(HomologyError::from)?;
            if v.bit() {
                joined
                    .set(r, n_image + c, Gf2::ONE)
                    .map_err(HomologyError::from)?;
            }
        }
    }

    let mut work = joined;
    let reduced = deep_causality_linear::rref(&mut work).map_err(HomologyError::from)?;

    let mut out = Vec::new();
    for &p in reduced.pivot_columns() {
        if p >= n_image {
            out.push(Gf2Chain::from_column(&kernel, p - n_image, degree)?);
        }
    }
    Ok(out)
}
