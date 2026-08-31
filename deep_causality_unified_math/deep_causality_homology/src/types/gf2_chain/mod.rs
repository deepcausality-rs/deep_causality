/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A mod-2 chain: a bit-packed 𝔽₂ vector that knows its degree.

use crate::errors::homology_error::HomologyError;
#[cfg(doc)]
use crate::errors::homology_error::HomologyErrorEnum;
use alloc::format;
use deep_causality_linear::{PackedGf2, PackedGf2Vector};
use deep_causality_num::{Gf2, NaturalNumber};

/// A `k`-chain over 𝔽₂, carrying its degree with its data.
///
/// # Why the name is not `Chain`
///
/// `deep_causality_topology::Chain<R, G>` is a different object: a *weighted* simplicial chain holding
/// an `Arc<SimplicialComplex<T>>` and a `CsrMatrix<T>` of ring-valued weights. This one has
/// coefficients in 𝔽₂, one bit each, and needs no complex to be well formed. Two names for two
/// things.
///
/// # What identifies the chain group
///
/// The pair `(degree, len)`, and nothing else. `C_k = 𝔽₂^{n_k}` is fixed by the cell count, so two
/// complexes with the same number of `k`-cells have the *same* `C_k`, and a sum of two of its
/// elements is right whichever complex produced them. There is no further identity to check.
///
/// This is why the type holds no complex handle. Every operation it offers — the sum, the
/// intersection, the pairing, the support enumerations, the weight — belongs to the group rather
/// than to a complex. The complex enters with `∂`, and the compatibility check belongs there, made
/// against the complex being applied: `c.num_cells(degree) == len()`. A handle remembered at
/// construction cannot do that, because it can be stale.
///
/// Everything below the degree is [`PackedGf2Vector`], in `deep_causality_linear`, because a
/// bit-packed 𝔽₂ vector with a support, a pairing and an intersection is usable with no chain
/// complex anywhere in sight.
///
/// Both halves of the identity are checked in one place. A `1`-chain and a `2`-chain have no sum,
/// and neither do two `1`-chains of different lengths; `same_group` refuses
/// both with one error rather than letting the length mismatch surface from the packed vector
/// underneath.
///
/// # Where it is used
///
/// Haruna, *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction*
/// (arXiv:2511.15224) §2.14: a 1-chain `γ ∈ C₁` is a bit vector, and every logical gate in Table 1
/// is a product of physical gates over `supp(γ)` and its pairs and triples.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gf2Chain<W> {
    bits: PackedGf2Vector<W>,
    degree: usize,
}

impl<W: NaturalNumber> Gf2Chain<W> {
    /// The zero `k`-chain on a complex with `len` cells in degree `k`.
    pub fn zeros(len: usize, degree: usize) -> Self {
        Self {
            bits: PackedGf2Vector::zeros(len),
            degree,
        }
    }

    /// A chain from the cells it is supported on.
    ///
    /// A repeated index cancels, because the coefficients are in 𝔽₂.
    ///
    /// # Errors
    ///
    /// [`HomologyErrorEnum::LinearAlgebraError`] if
    /// an index is at or beyond `len`.
    pub fn from_support(
        len: usize,
        degree: usize,
        support: &[usize],
    ) -> Result<Self, HomologyError> {
        Ok(Self {
            bits: PackedGf2Vector::from_support(len, support).map_err(HomologyError::from)?,
            degree,
        })
    }

    /// A chain from one row of a packed 𝔽₂ matrix.
    ///
    /// This reads a row. A basis from `kernel_basis_gf2` or `image_basis_gf2` is stored down
    /// columns, so a generator becomes a chain through
    /// [`from_column`](Self::from_column) rather than this.
    ///
    /// # Errors
    ///
    /// [`HomologyErrorEnum::LinearAlgebraError`] if
    /// `row` is at or beyond the matrix's row count.
    pub fn from_row(m: &PackedGf2<W>, row: usize, degree: usize) -> Result<Self, HomologyError> {
        Ok(Self {
            bits: PackedGf2Vector::from_row(m, row).map_err(HomologyError::from)?,
            degree,
        })
    }

    /// A chain from one column of a packed 𝔽₂ matrix.
    ///
    /// `kernel_basis_gf2` and `image_basis_gf2` return their bases as columns, so this is how a
    /// generator becomes a chain of a stated degree.
    ///
    /// # Errors
    ///
    /// [`HomologyErrorEnum::LinearAlgebraError`] if
    /// `col` is at or beyond the matrix's column count.
    pub fn from_column(m: &PackedGf2<W>, col: usize, degree: usize) -> Result<Self, HomologyError> {
        Ok(Self {
            bits: PackedGf2Vector::from_column(m, col).map_err(HomologyError::from)?,
            degree,
        })
    }

    /// The degree.
    #[inline]
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// The underlying vector, without its degree.
    #[inline]
    pub fn bits(&self) -> &PackedGf2Vector<W> {
        &self.bits
    }

    /// The number of cells the chain ranges over, which is not its weight.
    #[inline]
    pub fn len(&self) -> usize {
        self.bits.len()
    }

    /// Whether the chain ranges over no cells at all. For the zero chain, see
    /// [`is_zero`](Self::is_zero).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// Whether every coefficient is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.bits.is_zero()
    }

    /// `|supp(γ)|`, the number of cells with coefficient one.
    #[inline]
    pub fn weight(&self) -> usize {
        self.bits.weight()
    }

    /// `supp(γ)`, ascending.
    pub fn support(&self) -> impl Iterator<Item = usize> + '_ {
        self.bits.support()
    }

    /// The unordered pairs of `supp(γ)`, which the two-qubit factors of Table 1 range over.
    pub fn support_pairs(&self) -> impl Iterator<Item = (usize, usize)> {
        self.bits.support_pairs()
    }

    /// The unordered triples of `supp(γ)`, which the `CCZ` factors range over.
    pub fn support_triples(&self) -> impl Iterator<Item = (usize, usize, usize)> {
        self.bits.support_triples()
    }

    /// The 𝔽₂ sum of two chains of the same degree.
    ///
    /// # Errors
    ///
    /// [`HomologyErrorEnum::ChainGroupMismatch`] if the
    /// degrees differ, and `LinearAlgebraError` if the lengths do.
    pub fn add(&self, rhs: &Self) -> Result<Self, HomologyError> {
        self.same_group(rhs)?;
        Ok(Self {
            bits: self.bits.add(&rhs.bits).map_err(HomologyError::from)?,
            degree: self.degree,
        })
    }

    /// The intersection `γ₁ ∩ γ₂`, entrywise.
    ///
    /// # Errors
    ///
    /// As [`add`](Self::add).
    pub fn intersect(&self, rhs: &Self) -> Result<Self, HomologyError> {
        self.same_group(rhs)?;
        Ok(Self {
            bits: self
                .bits
                .intersect(&rhs.bits)
                .map_err(HomologyError::from)?,
            degree: self.degree,
        })
    }

    /// The 𝔽₂ pairing `⟨γ₁, γ₂⟩ = Σᵢ γ₁ⁱγ₂ⁱ`.
    ///
    /// # Errors
    ///
    /// As [`add`](Self::add).
    pub fn inner(&self, rhs: &Self) -> Result<Gf2, HomologyError> {
        self.same_group(rhs)?;
        self.bits.inner(&rhs.bits).map_err(HomologyError::from)
    }

    /// Whether two chains live in the same chain group `C_k`, which is `(degree, len)`.
    ///
    /// Both halves are checked here so that one condition raises one error. Leaving the length to
    /// the packed vector underneath made a length mismatch and a degree mismatch two different
    /// error types for what is a single question.
    ///
    /// # Errors
    ///
    /// [`HomologyErrorEnum::ChainGroupMismatch`],
    /// naming both groups.
    fn same_group(&self, rhs: &Self) -> Result<(), HomologyError> {
        if self.degree != rhs.degree || self.bits.len() != rhs.bits.len() {
            return Err(HomologyError::ChainGroupMismatch(format!(
                "chains in C_{} of dimension {} and C_{} of dimension {} have no common operation",
                self.degree,
                self.bits.len(),
                rhs.degree,
                rhs.bits.len()
            )));
        }
        Ok(())
    }
}
