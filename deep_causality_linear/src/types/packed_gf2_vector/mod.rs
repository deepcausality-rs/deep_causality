/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A bit-packed vector over 𝔽₂.

use crate::errors::linear_error::LinearError;
use crate::traits::matrix_view::MatrixView;
use crate::types::packed_gf2::PackedGf2;
use alloc::vec;
use alloc::vec::Vec;
use core::marker::PhantomData;
use deep_causality_num::{Gf2, NaturalNumber};

/// A vector over 𝔽₂, one bit per entry, packed into words of type `W`.
///
/// # Why this is not a one-row [`PackedGf2`]
///
/// It could be stored as one, and [`from_row`](Self::from_row) exists because the 𝔽₂ elimination
/// hands back its kernel and image bases exactly that way. What a one-row matrix cannot carry is
/// the meaning of its operators. `PackedGf2`'s `mul` is matrix multiplication; the product this
/// type needs is the entrywise **intersection**, and its `inner` is a scalar rather than a vector.
/// Giving a matrix type vector semantics would make `a * b` mean two different things depending on
/// the shape of its operands.
///
/// # The operations, and where they come from
///
/// Haruna, *Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction*
/// (arXiv:2511.15224). A 1-chain `γ ∈ C₁` is a bit vector (§2.14); `supp(γ)` and the enumeration of
/// its pairs and triples drive every gate decomposition in Table 1 (§3.17, §3.51, §3.59); and the
/// pairing `⟨γ₁, γ₂⟩ = Σᵢ γ₁ⁱγ₂ⁱ` with the intersection `γ₁ ∩ γ₂` are both mod 2 (after §2.15).
///
/// None of that needs a chain complex, which is why the type sits here rather than in
/// `deep_causality_topology`. The degree that makes it a *chain* is added there.
///
/// # The trailing bits
///
/// A vector of `len` bits occupies `ceil(len / W::BITS)` words, and the bits beyond `len` in the
/// last word are kept zero. [`weight`](Self::weight) and [`inner`](Self::inner) count whole words,
/// so padding that was ever set would be counted as data. Keeping it zero is an invariant of the
/// type, and the reason every mutation goes through [`set`](Self::set).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedGf2Vector<W> {
    words: Vec<W>,
    len: usize,
    _word: PhantomData<W>,
}

impl<W: NaturalNumber> PackedGf2Vector<W> {
    /// The number of bits in one word.
    #[inline]
    pub fn bits_per_word() -> usize {
        core::mem::size_of::<W>() * 8
    }

    /// The all-zero vector of `len` bits.
    pub fn zeros(len: usize) -> Self {
        let words = len.div_ceil(Self::bits_per_word());
        Self {
            words: vec![W::zero(); words],
            len,
            _word: PhantomData,
        }
    }

    /// The number of entries, which is not the number of words.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Whether the vector has no entries at all, which is not whether it is the zero vector.
    /// For that, see [`is_zero`](Self::is_zero).
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The packed words. Exposed because the operations here are word-parallel and a caller
    /// measuring them needs to see the same words they do.
    #[inline]
    pub fn as_words(&self) -> &[W] {
        &self.words
    }

    /// Builds from a support: the indices whose entry is one.
    ///
    /// Repeated indices cancel, because addition in 𝔽₂ is exclusive or. That is deliberate: it
    /// makes this the same function as summing the basis vectors named.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if any index is at or beyond `len`.
    pub fn from_support(len: usize, support: &[usize]) -> Result<Self, LinearError> {
        let mut v = Self::zeros(len);
        for &i in support {
            if i >= len {
                return Err(LinearError::IndexOutOfBounds((i, 0), (len, 1)));
            }
            v.flip(i);
        }
        Ok(v)
    }

    /// Builds from one row of a packed matrix.
    ///
    /// `kernel_basis_gf2` and `image_basis_gf2` return their bases as the rows of a
    /// [`PackedGf2`], so this is how a homology or cohomology generator becomes a vector.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if `row` is at or beyond the matrix's row count.
    pub fn from_row(m: &PackedGf2<W>, row: usize) -> Result<Self, LinearError> {
        let (rows, cols) = (MatrixView::rows(m), MatrixView::cols(m));
        if row >= rows {
            return Err(LinearError::IndexOutOfBounds((row, 0), (rows, cols)));
        }
        let wpr = m.words_per_row();
        let start = row * wpr;
        Ok(Self {
            words: m.as_words()[start..start + wpr].to_vec(),
            len: cols,
            _word: PhantomData,
        })
    }

    /// The entry at `i`.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if `i` is at or beyond `len`.
    pub fn get(&self, i: usize) -> Result<Gf2, LinearError> {
        if i >= self.len {
            return Err(LinearError::IndexOutOfBounds((i, 0), (self.len, 1)));
        }
        let bits = Self::bits_per_word();
        let mask = W::one() << ((i % bits) as u32);
        Ok(Gf2::new(!(self.words[i / bits] & mask).is_zero()))
    }

    /// Sets the entry at `i`.
    ///
    /// # Errors
    ///
    /// [`LinearError::IndexOutOfBounds`] if `i` is at or beyond `len`.
    pub fn set(&mut self, i: usize, value: Gf2) -> Result<(), LinearError> {
        if i >= self.len {
            return Err(LinearError::IndexOutOfBounds((i, 0), (self.len, 1)));
        }
        let bits = Self::bits_per_word();
        let mask = W::one() << ((i % bits) as u32);
        let w = i / bits;
        self.words[w] = if value.bit() {
            self.words[w] | mask
        } else {
            self.words[w] & !mask
        };
        Ok(())
    }

    /// Flips the entry at `i`. Callers have already bounds-checked.
    #[inline]
    fn flip(&mut self, i: usize) {
        let bits = Self::bits_per_word();
        let mask = W::one() << ((i % bits) as u32);
        let w = i / bits;
        self.words[w] = self.words[w] ^ mask;
    }

    /// Whether every entry is zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.words.iter().all(|w| w.is_zero())
    }

    /// The Hamming weight: the number of entries equal to one, which is `supp(γ).len()`.
    #[inline]
    pub fn weight(&self) -> usize {
        self.words.iter().map(|w| w.count_ones() as usize).sum()
    }

    /// The 𝔽₂ sum, entrywise exclusive or.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the lengths differ.
    pub fn add(&self, rhs: &Self) -> Result<Self, LinearError> {
        self.zip_with(rhs, |a, b| a ^ b)
    }

    /// The intersection `γ₁ ∩ γ₂`, entrywise conjunction.
    ///
    /// Over 𝔽₂ this is also the entrywise product, which is why the paper writes the two
    /// interchangeably. It is *not* [`add`](Self::add): a support present in both survives here and
    /// cancels there.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the lengths differ.
    pub fn intersect(&self, rhs: &Self) -> Result<Self, LinearError> {
        self.zip_with(rhs, |a, b| a & b)
    }

    fn zip_with<F>(&self, rhs: &Self, f: F) -> Result<Self, LinearError>
    where
        F: Fn(W, W) -> W,
    {
        if self.len != rhs.len {
            return Err(LinearError::ShapeMismatch((self.len, 1), (rhs.len, 1)));
        }
        let words = self
            .words
            .iter()
            .zip(rhs.words.iter())
            .map(|(&a, &b)| f(a, b))
            .collect();
        Ok(Self {
            words,
            len: self.len,
            _word: PhantomData,
        })
    }

    /// The 𝔽₂ pairing `⟨γ₁, γ₂⟩ = Σᵢ γ₁ⁱγ₂ⁱ`, which is the parity of the intersection's weight.
    ///
    /// Computed word by word, so it never builds the intersection.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the lengths differ.
    pub fn inner(&self, rhs: &Self) -> Result<Gf2, LinearError> {
        if self.len != rhs.len {
            return Err(LinearError::ShapeMismatch((self.len, 1), (rhs.len, 1)));
        }
        let ones: u32 = self
            .words
            .iter()
            .zip(rhs.words.iter())
            .map(|(&a, &b)| (a & b).count_ones())
            .sum();
        Ok(Gf2::new(ones % 2 == 1))
    }

    /// The support, ascending: every index whose entry is one.
    ///
    /// Walks the set bits rather than the entries, so the cost is the weight and not the length.
    pub fn support(&self) -> impl Iterator<Item = usize> + '_ {
        let bits = Self::bits_per_word();
        self.words
            .iter()
            .enumerate()
            .flat_map(move |(wi, &word)| SetBits { word }.map(move |b| wi * bits + b))
    }

    /// The unordered pairs of the support, `(i, j)` with `i < j`, ascending.
    ///
    /// Table 1 needs these for the two-qubit factors: `S̄(γ)` carries a `CZ` over every pair of
    /// `supp(γ)`.
    pub fn support_pairs(&self) -> impl Iterator<Item = (usize, usize)> {
        let s: Vec<usize> = self.support().collect();
        let mut out = Vec::with_capacity(s.len() * s.len().saturating_sub(1) / 2);
        for a in 0..s.len() {
            for b in (a + 1)..s.len() {
                out.push((s[a], s[b]));
            }
        }
        out.into_iter()
    }

    /// The unordered triples of the support, `(i, j, k)` with `i < j < k`, ascending.
    ///
    /// The `CCZ` factors of Table 1 range over these.
    pub fn support_triples(&self) -> impl Iterator<Item = (usize, usize, usize)> {
        let s: Vec<usize> = self.support().collect();
        let n = s.len();
        let mut out = Vec::new();
        for a in 0..n {
            for b in (a + 1)..n {
                for c in (b + 1)..n {
                    out.push((s[a], s[b], s[c]));
                }
            }
        }
        out.into_iter()
    }
}

/// Walks the set bits of one word, least significant first.
struct SetBits<W> {
    word: W,
}

impl<W: NaturalNumber> Iterator for SetBits<W> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        if self.word.is_zero() {
            return None;
        }
        let b = self.word.trailing_zeros() as usize;
        // Clear the lowest set bit: `x & (x − 1)`. `monus` rather than `-`, because ℕ has no
        // subtraction operator in this tower; the word is non-zero here, so the two agree.
        self.word = self.word & self.word.monus(W::one());
        Some(b)
    }
}
