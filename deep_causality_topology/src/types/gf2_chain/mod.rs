/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A mod-2 chain: a bit-packed 𝔽₂ vector that knows its degree.

use crate::errors::topology_error::TopologyError;
use deep_causality_linear::{PackedGf2, PackedGf2Vector};
use deep_causality_num::{Gf2, NaturalNumber};

/// A `k`-chain over 𝔽₂, carrying its degree with its data.
///
/// # Why the name is not `Chain`
///
/// [`Chain<T>`](crate::Chain) is taken, by a different object: a *weighted* simplicial chain
/// holding an `Arc<SimplicialComplex<T>>` and a `CsrMatrix<T>` of ring-valued weights, with the
/// group and module structure that implies. This one has coefficients in 𝔽₂, one bit each, and
/// needs no complex to be well formed. Two names for two things.
///
/// # What the degree is for
///
/// Everything below the degree is [`PackedGf2Vector`], in `deep_causality_linear`, because a
/// bit-packed 𝔽₂ vector with a support, a pairing and an intersection is usable with no chain
/// complex anywhere in sight. The degree is the part that makes it a chain, and it is the part that
/// lives here.
///
/// It is also the part that catches mistakes. A `1`-chain and a `2`-chain have no sum and no
/// pairing, and passing one where the other belongs is the error a bare `&[bool]` with a separate
/// `usize` cannot refuse. Every binary operation here checks the degree first.
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
    /// [`TopologyErrorEnum::LinearAlgebraError`](crate::TopologyErrorEnum::LinearAlgebraError) if
    /// an index is at or beyond `len`.
    pub fn from_support(
        len: usize,
        degree: usize,
        support: &[usize],
    ) -> Result<Self, TopologyError> {
        Ok(Self {
            bits: PackedGf2Vector::from_support(len, support).map_err(TopologyError::from)?,
            degree,
        })
    }

    /// A chain from one row of a packed 𝔽₂ matrix.
    ///
    /// `kernel_basis_gf2` and `image_basis_gf2` return their bases as rows, so this is how a
    /// generator becomes a chain of a stated degree.
    ///
    /// # Errors
    ///
    /// [`TopologyErrorEnum::LinearAlgebraError`](crate::TopologyErrorEnum::LinearAlgebraError) if
    /// `row` is at or beyond the matrix's row count.
    pub fn from_row(m: &PackedGf2<W>, row: usize, degree: usize) -> Result<Self, TopologyError> {
        Ok(Self {
            bits: PackedGf2Vector::from_row(m, row).map_err(TopologyError::from)?,
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
    /// [`TopologyErrorEnum::DimensionMismatch`](crate::TopologyErrorEnum::DimensionMismatch) if the
    /// degrees differ, and `LinearAlgebraError` if the lengths do.
    pub fn add(&self, rhs: &Self) -> Result<Self, TopologyError> {
        self.same_degree(rhs)?;
        Ok(Self {
            bits: self.bits.add(&rhs.bits).map_err(TopologyError::from)?,
            degree: self.degree,
        })
    }

    /// The intersection `γ₁ ∩ γ₂`, entrywise.
    ///
    /// # Errors
    ///
    /// As [`add`](Self::add).
    pub fn intersect(&self, rhs: &Self) -> Result<Self, TopologyError> {
        self.same_degree(rhs)?;
        Ok(Self {
            bits: self
                .bits
                .intersect(&rhs.bits)
                .map_err(TopologyError::from)?,
            degree: self.degree,
        })
    }

    /// The 𝔽₂ pairing `⟨γ₁, γ₂⟩ = Σᵢ γ₁ⁱγ₂ⁱ`.
    ///
    /// # Errors
    ///
    /// As [`add`](Self::add).
    pub fn inner(&self, rhs: &Self) -> Result<Gf2, TopologyError> {
        self.same_degree(rhs)?;
        self.bits.inner(&rhs.bits).map_err(TopologyError::from)
    }

    fn same_degree(&self, rhs: &Self) -> Result<(), TopologyError> {
        if self.degree != rhs.degree {
            return Err(TopologyError::DimensionMismatch(format!(
                "chains of degree {} and {} have no common operation",
                self.degree, rhs.degree
            )));
        }
        Ok(())
    }
}
