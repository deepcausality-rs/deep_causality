/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Poisson distribution.

use crate::{RandWidth, StandardUniform, StatsError};
use deep_causality_algebra::{Real, RealField};
use deep_causality_num::{FromPrimitive, ToPrimitive};
use deep_causality_rand::{Distribution, Rng};

/// The largest rate this sampler accepts.
///
/// Knuth's algorithm multiplies uniform draws until the product falls below `e^{-λ}`. Two things
/// bound it from above:
///
/// - **Termination.** `e^{-λ}` underflows to zero somewhere near `λ = 745` at `f64`, and far
///   sooner at a narrower scalar. Below a threshold of zero the product can never fall, so the
///   loop cannot end by its own condition.
/// - **Cost.** The expected iteration count is `λ + 1`, so the algorithm is linear in the rate. It
///   is right for the small rates a simulation asks for and wrong for large ones.
///
/// The cap is set well below the underflow point rather than at it, because the margin is what
/// makes the bound a property of the *distribution* rather than of the scalar it is sampled at:
/// `e^{-500}` is representable in `f64` and in `Float106`, and an `f32` caller is refused before
/// its own narrower limit is reached.
///
/// A rate above this is refused rather than approximated. Switching to a different algorithm above
/// the cap would be a reasonable alternative; refusing is chosen here because a silent change of
/// method is harder to reason about than an error, and no caller in this workspace needs it.
pub const MAX_RATE: f64 = 500.0;

/// The most iterations one draw will take before returning the count it has reached.
///
/// The loop is bounded in *expectation* — `λ + 1` iterations — but not in the worst case, and the
/// worst case is not merely slow. A generator returning a uniform draw of `1 - 2^-53` every time
/// decays the product by that factor per step, so falling below `e^{-4}` would take about
/// `4 · 2^53 ≈ 3.6e16` iterations. That is a hang, not a delay.
///
/// This was found by the test in this crate that samples from an all-ones generator: it did not
/// fail, it never returned.
///
/// A sound generator reaches this cap with probability far below any scale worth naming — at the
/// largest admissible rate the expected count is 501, and the tail of the iteration count decays
/// geometrically. So the cap costs nothing in practice and converts a hang into a value.
pub const MAX_ITERATIONS: u64 = 1_000_000;

/// The Poisson distribution with rate `λ`, supported on the non-negative integers.
///
/// `mean = variance = λ`. A sampler that gets the mean right and the spread wrong is a common
/// failure, so both are worth asserting.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Poisson<T> {
    rate: T,
}

impl<T> Poisson<T>
where
    T: RealField + ToPrimitive,
{
    /// Construct from a rate.
    ///
    /// Refuses a negative, non-finite, or out-of-range rate. See [`MAX_RATE`] for why the range is
    /// bounded above. A rate of exactly zero is accepted and gives a constant zero, which is the
    /// distribution's correct degenerate case.
    pub fn new(rate: T) -> Result<Self, StatsError> {
        if !rate.is_finite() {
            return Err(StatsError::NonFiniteInput(
                "the Poisson distribution is not defined at a non-finite rate",
            ));
        }
        if rate < T::zero() {
            return Err(StatsError::NegativeProbability(
                "the Poisson distribution needs a non-negative rate",
            ));
        }
        let as_f64 = rate
            .to_f64()
            .ok_or(StatsError::NonFiniteInput("the rate does not lower to f64"))?;
        if as_f64 > MAX_RATE {
            return Err(StatsError::NonPositiveScale(
                "the Poisson rate exceeds the range this sampler's algorithm serves; see MAX_RATE",
            ));
        }
        Ok(Self { rate })
    }

    /// The rate `λ`.
    pub fn rate(&self) -> T {
        self.rate
    }
}

/// Knuth's algorithm: multiply uniform draws until the product falls below `e^{-λ}`.
impl<T> Distribution<u64> for Poisson<T>
where
    T: RealField + FromPrimitive + RandWidth + ToPrimitive,
    StandardUniform: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> u64 {
        // The threshold is positive for every rate the constructor admits, so the product — which
        // strictly decreases, since every uniform draw is below one — must fall below it.
        let threshold = Real::exp(-self.rate);
        let mut k = 0_u64;
        let mut product = T::one();
        while k < MAX_ITERATIONS {
            product *= StandardUniform.sample(rng);
            if product <= threshold {
                return k;
            }
            k += 1;
        }
        // Only a degenerate generator reaches this. See `MAX_ITERATIONS`.
        k
    }
}
