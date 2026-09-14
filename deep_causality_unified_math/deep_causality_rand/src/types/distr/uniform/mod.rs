/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod standard_word;

use crate::{
    Distribution, Rng, SampleBorrow, SampleUniform, UniformDistributionError, UniformSampler,
};
use crate::StandardWord;
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

#[derive(Debug, Copy, Clone)]
pub struct Uniform<X: SampleUniform>(X::Sampler);

impl<X: SampleUniform> Uniform<X> {
    /// Create a new `Uniform` instance, which samples uniformly from the half
    /// open range `[low, high)` (excluding `high`).
    ///
    /// For discrete types (e.g. integers), samples will always be strictly less
    /// than `high`. For (approximations of) continuous types (e.g. `f32`, `f64`),
    /// samples may equal `high` due to loss of precision but may not be
    /// greater than `high`.
    ///
    /// Fails if `low >= high`, or if `low`, `high` or the range `high - low` is
    /// non-finite. In release mode, only the range is checked.
    pub fn new<B1, B2>(low: B1, high: B2) -> Result<Uniform<X>, UniformDistributionError>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        X::Sampler::new(low, high).map(Uniform)
    }

    /// Create a new `Uniform` instance, which samples uniformly from the closed
    /// range `[low, high]` (inclusive).
    ///
    /// Fails if `low > high`, or if `low`, `high` or the range `high - low` is
    /// non-finite. In release mode, only the range is checked.
    pub fn new_inclusive<B1, B2>(low: B1, high: B2) -> Result<Uniform<X>, UniformDistributionError>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        X::Sampler::new_inclusive(low, high).map(Uniform)
    }
}

impl<X: SampleUniform> Distribution<X> for Uniform<X> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> X {
        self.0.sample(rng)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformFloat<F: RealField> {
    low: F,
    scale: F,
}

/// A uniform `[0, 1)` for the range sampler, at the caller's scalar.
///
/// The body is written once here and every scalar inherits it; the only per-type fact is
/// [`RandFloat::WORDS`], how many 53-bit generator words the significand absorbs. A double-double
/// that took one word would be an `f64` wearing a wider type — it satisfies every bounds and
/// moment check and silently loses half its entropy.
///
/// This mirrors `deep_causality_stats::RandWidth`, which does the same job for the distributions.
/// The two are separate because the crates are: the range sampler owes nothing to the distribution
/// layer, and a type implementing one need not implement the other.
pub trait RandFloat: RealField + FromPrimitive {
    /// The number of 53-bit generator words this scalar's significand can absorb.
    const WORDS: u32 = 1;

    /// A uniform draw on `[0, 1)`.
    ///
    /// Computed in `f64` and converted once rather than accumulated in the target scalar, so a
    /// narrow significand cannot quantize the intermediate before the value is formed.
    fn rand_float_gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        const WORD_SCALE: f64 = 1.0 / ((1_u64 << 53) as f64);
        let word = |rng: &mut R| -> f64 {
            let w: u64 = StandardWord.sample(rng);
            (w >> 11) as f64 * WORD_SCALE
        };
        let mut acc = Self::from_f64(word(rng)).expect("a unit value converts to every scalar");
        let mut scale = Self::from_f64(WORD_SCALE).expect("2^-53 converts to every scalar");
        for _ in 1..Self::WORDS {
            acc += Self::from_f64(word(rng)).expect("a unit value converts to every scalar") * scale;
            scale *= Self::from_f64(WORD_SCALE).expect("2^-53 converts to every scalar");
        }
        acc
    }
}

impl<F> UniformSampler for UniformFloat<F>
where
    F: RealField + RandFloat + Debug,
{
    type X = F;

    fn new<B1, B2>(low_b: B1, high_b: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = *low_b.borrow();
        let high = *high_b.borrow();
        if !(low.is_finite() && high.is_finite()) {
            return Err(UniformDistributionError::NonFinite);
        }
        if low >= high {
            return Err(UniformDistributionError::EmptyRange);
        }
        let scale = high - low;
        if !scale.is_finite() {
            return Err(UniformDistributionError::NonFinite);
        }
        Ok(UniformFloat { low, scale })
    }

    fn new_inclusive<B1, B2>(low_b: B1, high_b: B2) -> Result<Self, UniformDistributionError>
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        let low = *low_b.borrow();
        let high = *high_b.borrow();
        if !(low.is_finite() && high.is_finite()) {
            return Err(UniformDistributionError::NonFinite);
        }
        if low > high {
            return Err(UniformDistributionError::EmptyRange);
        }

        let max_rand = F::one() - F::epsilon();
        let scale = (high - low) / max_rand;
        if !scale.is_finite() {
            return Err(UniformDistributionError::NonFinite);
        }

        Ok(UniformFloat { low, scale })
    }

    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        let value0_1 = F::rand_float_gen(rng);
        value0_1 * self.scale + self.low
    }
}
