/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Unweighted choice over an integer range.

use crate::StatsError;
use deep_causality_rand::{Distribution, Rng, StandardWord};

/// Uniform over `0..n`, returning an index.
///
/// # Why this exists
///
/// [`Categorical`](crate::Categorical) covers weighted choice; nothing covered the unweighted case,
/// so every caller wrote `(rng.next_u64() as usize) % n` by hand. Picking a lattice edge, a graph
/// vertex or an array index is the same operation each time, and the hand-written form is wrong.
///
/// # Modulo is biased, and the bias is invisible
///
/// `w % n` for a uniform 64-bit `w` is not uniform unless `n` divides `2^64`. The residues below
/// `2^64 mod n` occur once more often than the rest. Every value is in range, so a bounds check
/// passes; only a frequency test at a size that does not divide a power of two can see it.
///
/// This type rejects the out-of-band words instead. The rejection probability is below
/// `n / 2^64` — for any range a caller would name, far below one draw in a lifetime of draws — so
/// the loop is expected to run once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UniformInt {
    len: u64,
}

impl UniformInt {
    /// Construct over `0..len`.
    ///
    /// Refuses an empty range: there is no value to return, and a distribution over no outcomes is
    /// not a distribution.
    pub fn new(len: u64) -> Result<Self, StatsError> {
        if len == 0 {
            return Err(StatsError::EmptyInput(
                "UniformInt: the range is empty, so there is no value to draw",
            ));
        }
        Ok(Self { len })
    }

    /// How many values the range holds.
    pub fn len(&self) -> u64 {
        self.len
    }

    /// Whether the range is empty. Never true for a constructed value.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Rejection sampling over the largest multiple of `len` that fits in a word.
///
/// The word is multiplied by the range and the high half taken, which maps the word range onto
/// `0..len` in `len` equal blocks plus one short block at the bottom. The low half says where in
/// its block the word fell, so a low half below `2^64 mod len` is exactly a word from the short
/// block, and those are redrawn. Only then is the map uniform.
impl Distribution<u64> for UniformInt {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u64 {
        let len = u128::from(self.len);
        let word: u64 = StandardWord.sample(rng);
        let mut product = u128::from(word) * len;
        let mut low = product as u64;
        if low < self.len {
            // `2^64 mod len`, written without a 128-bit division.
            let short_block = self.len.wrapping_neg() % self.len;
            while low < short_block {
                let word: u64 = StandardWord.sample(rng);
                product = u128::from(word) * len;
                low = product as u64;
            }
        }
        (product >> 64) as u64
    }
}

/// The same draw, as a `usize`, which is what an index caller wants.
///
/// This forwards rather than repeating the arithmetic: two hand-written copies of a rejection
/// loop are two chances to get the threshold wrong.
impl Distribution<usize> for UniformInt {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> usize {
        let index: u64 = self.sample(rng);
        index as usize
    }
}
