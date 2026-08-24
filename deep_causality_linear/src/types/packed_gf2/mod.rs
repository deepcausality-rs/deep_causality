/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A bit-packed matrix over 𝔽₂.

pub mod algebra;
pub mod ops;

use crate::errors::linear_error::LinearError;
use alloc::vec::Vec;
use core::marker::PhantomData;
use deep_causality_num::{Gf2, NaturalNumber};

/// A matrix over 𝔽₂ storing one bit per entry, packed into words of type `W`.
///
/// # Storage against element type
///
/// These are separate decisions and this type makes them separately. The **storage** is packed
/// bits, because the measurement says so: against a byte-per-entry `Gf2` scalar, packed `u64`
/// elimination runs 1.7× faster at n=128 rising to 3.2× at n=2048, on one eighth the memory, and
/// the ratio grows with n as cache pressure does.
///
/// The **element type** is [`Gf2`] from `deep_causality_num`. A packed matrix still has to answer
/// `get` with a value, and that value is an element of 𝔽₂. `deep_causality_linear` defines no scalar
/// of its own, so the tower carries 𝔽₂ and this type names it.
///
/// # Generic over the word
///
/// `W` is bounded on [`NaturalNumber`], which supplies the bit primitives elimination needs. Two
/// reasons not to fix it to `u64`. A caller may pick the width that suits the target. And the
/// crate's own suite can run at a narrow width, where an edge case at a word boundary shows up in a
/// matrix small enough to read — a column count that is not a multiple of the word width is a
/// correctness question, and at `u64` the smallest matrix that exercises it has 65 columns.
///
/// The same matrix packed at two widths must report the same rank and the same pivot columns. That
/// is a test rather than an assumption.
///
/// # The trailing bits
///
/// A row occupies `ceil(cols / W::BITS)` words, so the last word of each row holds padding bits
/// beyond `cols`. They are kept zero. Elimination combines whole words, so padding that was ever
/// non-zero would leak into the result and change the rank; keeping it zero is an invariant of the
/// type and the reason `set` is the only way in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedGf2<W> {
    /// Row-major, `words_per_row` words per row.
    words: Vec<W>,
    rows: usize,
    cols: usize,
    /// `ceil(cols / W::BITS)`, cached because every index computation needs it.
    words_per_row: usize,
    _word: PhantomData<W>,
}

impl<W: NaturalNumber> PackedGf2<W> {
    /// The number of bits in one word.
    pub fn bits_per_word() -> usize {
        core::mem::size_of::<W>() * 8
    }

    /// The number of words one row occupies, which is `ceil(cols / bits_per_word)`.
    pub fn words_per_row(&self) -> usize {
        self.words_per_row
    }

    /// The packed words, row-major.
    ///
    /// Exposed because the 𝔽₂ elimination is word-parallel and a caller measuring it needs to see
    /// the same words the algorithm does.
    pub fn as_words(&self) -> &[W] {
        &self.words
    }

    /// The packed words, mutably. Internal: the padding invariant is this type's to keep.
    pub(crate) fn as_mut_words(&mut self) -> &mut [W] {
        &mut self.words
    }

    /// The shape, as `(rows, cols)`.
    pub(crate) fn dims(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    /// An all-zero matrix of the given shape, with the padding invariant established.
    pub(crate) fn allocate(rows: usize, cols: usize) -> Self {
        let bits = core::mem::size_of::<W>() * 8;
        let words_per_row = cols.div_ceil(bits);
        Self {
            words: alloc::vec![W::zero(); rows * words_per_row],
            rows,
            cols,
            words_per_row,
            _word: PhantomData,
        }
    }

    /// Builds from entries in `{0, 1}` given as integers, reducing each modulo 2.
    ///
    /// This is the conversion `deep_causality_topology`'s boundary operators need. Their entries are
    /// `{-1, 0, 1}`, and `-1` and `1` are both the 𝔽₂ one.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the buffer length is not `rows * cols`.
    pub fn from_i64_mod2(data: &[i64], rows: usize, cols: usize) -> Result<Self, LinearError> {
        if data.len() != rows * cols {
            return Err(LinearError::ShapeMismatch {
                left: (rows, cols),
                right: (data.len(), 1),
            });
        }
        let mut m = Self::allocate(rows, cols);
        for i in 0..rows {
            for j in 0..cols {
                if data[i * cols + j] % 2 != 0 {
                    m.set_bit(i, j);
                }
            }
        }
        Ok(m)
    }

    /// Sets the bit at `(row, col)`. Callers have already bounds-checked.
    pub(crate) fn set_bit(&mut self, row: usize, col: usize) {
        let bits = core::mem::size_of::<W>() * 8;
        let idx = row * self.words_per_row + col / bits;
        let mask = W::one() << ((col % bits) as u32);
        self.words[idx] = self.words[idx] | mask;
    }

    /// Clears the bit at `(row, col)`.
    pub(crate) fn clear_bit(&mut self, row: usize, col: usize) {
        let bits = core::mem::size_of::<W>() * 8;
        let idx = row * self.words_per_row + col / bits;
        let mask = W::one() << ((col % bits) as u32);
        self.words[idx] = self.words[idx] & !mask;
    }

    /// Reads the bit at `(row, col)`.
    pub(crate) fn bit_at(&self, row: usize, col: usize) -> bool {
        let bits = core::mem::size_of::<W>() * 8;
        let idx = row * self.words_per_row + col / bits;
        let mask = W::one() << ((col % bits) as u32);
        !(self.words[idx] & mask).is_zero()
    }

    /// Builds from 𝔽₂ entries given row-major.
    ///
    /// # Errors
    ///
    /// [`LinearError::ShapeMismatch`] if the buffer length is not `rows * cols`.
    pub fn from_slice(data: &[Gf2], rows: usize, cols: usize) -> Result<Self, LinearError> {
        if data.len() != rows * cols {
            return Err(LinearError::ShapeMismatch {
                left: (rows, cols),
                right: (data.len(), 1),
            });
        }
        let mut m = Self::allocate(rows, cols);
        for i in 0..rows {
            for j in 0..cols {
                if data[i * cols + j].bit() {
                    m.set_bit(i, j);
                }
            }
        }
        Ok(m)
    }
}
