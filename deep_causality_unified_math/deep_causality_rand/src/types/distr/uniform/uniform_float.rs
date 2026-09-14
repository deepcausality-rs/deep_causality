/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The float tower's range sampler, bound once.
//!
//! Every scalar that is a real field draws from a range through [`UniformFloat`], and this file
//! names none of them. `f32`, `f64`, `Float106` and `BFloat16` are served by the single
//! implementation below, and so is whatever is added next.

use crate::StandardWord;
use crate::{Distribution, Rng, SampleBorrow, UniformDistributionError, UniformSampler};
use crate::{FloatKind, SampleUniform};
use core::fmt::Debug;
use core::marker::PhantomData;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<T: RealField + FromPrimitive> SampleUniform<FloatKind> for T {
    type Sampler = UniformFloat<T>;
}

/// Uniform over a half-open range of `X`.
///
/// `Kind` names the tower `X` belongs to and defaults to the float one, which is what every caller
/// of this type wants; an integer range is drawn through [`Rng::random_range`](crate::Rng) instead.
#[derive(Debug, Copy, Clone)]
pub struct Uniform<X, K = FloatKind>(X::Sampler, PhantomData<fn() -> K>)
where
    X: SampleUniform<K>;

impl<X: SampleUniform<K>, K> Uniform<X, K> {
    /// Create a new `Uniform` instance, which samples uniformly from the half
    /// open range `[low, high)` (excluding `high`).
    ///
    /// Samples are strictly below `high` at every scalar. A narrow significand can round the
    /// affine map up onto the bound, and such a result is rejected and redrawn rather than
    /// returned.
    ///
    /// Fails if `low >= high`, or if `low`, `high` or the range `high - low` is
    /// non-finite. In release mode, only the range is checked.
    pub fn new<B1, B2>(low: B1, high: B2) -> Result<Uniform<X, K>, UniformDistributionError>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        X::Sampler::new(low, high).map(|s| Uniform(s, PhantomData))
    }

    /// Create a new `Uniform` instance, which samples uniformly from the closed
    /// range `[low, high]` (inclusive).
    ///
    /// Fails if `low > high`, or if `low`, `high` or the range `high - low` is
    /// non-finite. In release mode, only the range is checked.
    pub fn new_inclusive<B1, B2>(
        low: B1,
        high: B2,
    ) -> Result<Uniform<X, K>, UniformDistributionError>
    where
        B1: SampleBorrow<X> + Sized,
        B2: SampleBorrow<X> + Sized,
    {
        X::Sampler::new_inclusive(low, high).map(|s| Uniform(s, PhantomData))
    }
}

impl<X: SampleUniform<K>, K> Distribution<X> for Uniform<X, K> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> X {
        self.0.sample(rng)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformFloat<F: RealField> {
    low: F,
    scale: F,
    /// The upper bound a draw may not reach, for a half-open range.
    ///
    /// `new_inclusive` leaves this `None`, because `high` is a legitimate result there.
    exclusive_high: Option<F>,
}

/// Every scalar the sampling layer works at.
///
/// Blanket-implemented, so a scalar joins by satisfying the algebra rather than by an entry in a
/// list here. There is no per-type fact in this crate to keep in step with the type.
///
/// # How wide a draw is, without being told
///
/// A draw is built from 53-bit generator words, and a wide scalar needs more than one: a
/// double-double handed a single word is an `f64` wearing a wider type, passing every bounds and
/// moment check while carrying half the entropy it claims.
///
/// The count is not declared anywhere. The loop below stops when the next word would land entirely
/// below the scalar's own resolution, which [`Real::epsilon`] already reports — so `f32`, `f64` and
/// `BFloat16` take one word, `Float106` takes two, and a scalar added tomorrow takes however many
/// it needs on the day it arrives.
///
/// An earlier design declared the count as a `WORDS` constant on this trait and again on a second
/// trait in `deep_causality_stats`. Both stated a fact the scalar was already reporting one call
/// away, and both had to be edited by hand for every new type.
pub trait RandScalar: RealField + FromPrimitive {
    /// A uniform draw on `[0, 1)`, carrying as much entropy as the scalar can hold.
    ///
    /// # Why the result is rejected and redrawn
    ///
    /// A value drawn from `[0, 1)` can round onto exactly `1.0` in a narrow significand, which
    /// leaves the interval this function promises and the one every inverse-CDF transform above it
    /// assumes. Measured at `BFloat16`, whose significand is 8 bits: 6 draws in 2 000. Rejecting
    /// costs a redraw at that rate and nothing at all for `f32` and wider, where the rate is
    /// `2^-25` or below; clamping instead would pile an atom of mass on one value.
    fn rand_float_gen<R: Rng + ?Sized>(rng: &mut R) -> Self {
        loop {
            let candidate = Self::rand_float_open(rng);
            if candidate < Self::one() {
                return candidate;
            }
        }
    }

    /// One accumulation pass, which may land on `1.0`. See [`RandScalar::rand_float_gen`].
    fn rand_float_open<R: Rng + ?Sized>(rng: &mut R) -> Self {
        const WORD_SCALE: f64 = 1.0 / ((1_u64 << 53) as f64);
        let word = |rng: &mut R| -> f64 {
            let w: u64 = StandardWord.sample(rng);
            (w >> 11) as f64 * WORD_SCALE
        };
        let word_scale = Self::from_f64(WORD_SCALE).expect("2^-53 converts to every scalar");

        let mut acc = Self::from_f64(word(rng)).expect("a unit value converts to every scalar");
        let mut scale = word_scale;
        // Each further word sits `2^-53` below the last. Once that weight is under the scalar's
        // epsilon the word could not change a single bit of `acc`, so drawing it would be waste.
        while scale > Self::epsilon() {
            acc +=
                Self::from_f64(word(rng)).expect("a unit value converts to every scalar") * scale;
            scale *= word_scale;
        }
        acc
    }
}

/// Blanket: the algebra is the only entry requirement.
impl<T: RealField + FromPrimitive> RandScalar for T {}

impl<F> UniformSampler for UniformFloat<F>
where
    F: RandScalar,
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
        Ok(UniformFloat {
            low,
            scale,
            exclusive_high: Some(high),
        })
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

        Ok(UniformFloat {
            low,
            scale,
            exclusive_high: None,
        })
    }

    /// A draw in `[low, high)`, or `[low, high]` when built by `new_inclusive`.
    ///
    /// # Why the half-open case rejects
    ///
    /// `u * scale + low` is an affine map, and a narrow significand can round its result up onto
    /// `high` even when `u` is strictly below one. At `BFloat16` that happened in 14 draws of
    /// 2 000; at `f32` and wider, in none of them — which is why the bound went unnoticed while
    /// this crate served only the wide scalars.
    ///
    /// Testing the produced value rather than the draw is what makes the guarantee hold for every
    /// scalar, including any added later: whatever the arithmetic does on the way, a result that
    /// reaches `high` is thrown away. `low` remains reachable, so the loop terminates.
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        loop {
            let value = F::rand_float_gen(rng) * self.scale + self.low;
            match self.exclusive_high {
                Some(high) if value >= high => continue,
                _ => return value,
            }
        }
    }
}
