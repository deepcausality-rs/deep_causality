/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Uniform sampling over a range of unsigned integers, written once.
//!
//! Three near-identical files stood here — `u32`, `u64` and `usize` — differing only in the width
//! spelled into each. They disagreed, as copies do: the `usize` one drew from `next_u32`, so on a
//! 64-bit target any range wider than `2^32` silently returned only its bottom `2^32`. Measured
//! before the rewrite, over a range of `2^40`: 200 000 draws never exceeded 4 294 942 982.
//!
//! `deep_causality_num`'s integer tower makes the width a parameter like any other — `Integer`
//! supplies `BITS` as a `const`, `UnsignedInt` the power-of-two helpers, `Num` the arithmetic — so
//! one body serves `u8` through `u128`, and `u8`, `u16` and `u128` gain samplers they never had.

use crate::{
    Rng, SampleBorrow, SampleUniform, UniformDistributionError, UniformSampler, UnsignedKind,
};
use core::fmt::Debug;
use deep_causality_num::{FromPrimitive, NaturalNumber, Num};

/// A scalar this sampler serves: any unsigned integer the tower describes.
pub trait RandUnsigned: NaturalNumber + Num + FromPrimitive + Debug {}

/// Blanket: the tower is the only entry requirement.
impl<T: NaturalNumber + Num + FromPrimitive + Debug> RandUnsigned for T {}

impl<T: RandUnsigned> SampleUniform<UnsignedKind> for T {
    type Sampler = UniformUnsigned<T>;
}

/// Uniform over `[low, low + span)`, by bitmask rejection.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct UniformUnsigned<T> {
    low: T,
    /// How many values the range holds. Zero means the whole type, which `high - low` cannot
    /// express for an inclusive range ending at `MAX`.
    span: T,
}

impl<T: RandUnsigned> UniformSampler for UniformUnsigned<T> {
    type X = T;

    fn new<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = *low.borrow();
        let high = *high.borrow();
        if low >= high {
            return Err(UniformDistributionError::InvalidRange);
        }
        Ok(Self {
            low,
            span: high - low,
        })
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = *low.borrow();
        let high = *high.borrow();
        if low > high {
            return Err(UniformDistributionError::InvalidRange);
        }
        // `high - low + 1` is the count, and it overflows to zero exactly when the range is the
        // whole type. Zero is the sentinel for that, which `sample` reads.
        let span = high - low;
        Ok(Self {
            low,
            span: if span == T::MAX {
                T::zero()
            } else {
                span + T::one()
            },
        })
    }

    /// Bitmask rejection: draw the smallest number of bits that covers the span, and redraw when
    /// the value lands above it.
    ///
    /// Unbiased by construction, unlike the `% span` this replaces, which over-weights the low
    /// residues whenever the span does not divide a power of two. The mask is at most twice the
    /// span, so fewer than half the draws are rejected and the loop is expected to run under twice.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        if self.span == T::zero() {
            return random_bits::<T, R>(rng);
        }
        // A span above half the type's range has no power of two above it; every draw is then in
        // range after masking with the full width.
        let mask = match self.span.checked_next_power_of_two() {
            Some(p) => p - T::one(),
            None => !T::zero(),
        };
        loop {
            let candidate = random_bits::<T, R>(rng) & mask;
            if candidate < self.span {
                return self.low + candidate;
            }
        }
    }
}

/// `T::BITS` uniform bits, assembled from 64-bit generator words.
///
/// The first chunk is taken rather than shifted in, because `acc << 64` is an overflow for a
/// 64-bit `acc` and a panic in debug.
#[inline]
fn random_bits<T: RandUnsigned, R: Rng + ?Sized>(rng: &mut R) -> T {
    let mut acc = T::zero();
    let mut filled: u32 = 0;
    while filled < T::BITS {
        let take = core::cmp::min(64, T::BITS - filled);
        // The low `take` bits of the word. Either end of a generator word is equally uniform;
        // the low end is taken because it is what a narrower draw would have produced, so a
        // scripted word reads the same at every width.
        let word = rng.next_u64();
        let bits = if take == 64 {
            word
        } else {
            word & ((1_u64 << take) - 1)
        };
        let chunk = T::from_u64(bits).expect("a value below 2^take fits in a type of that width");
        acc = if filled == 0 {
            chunk
        } else {
            (acc << take) | chunk
        };
        filled += take;
    }
    acc
}
