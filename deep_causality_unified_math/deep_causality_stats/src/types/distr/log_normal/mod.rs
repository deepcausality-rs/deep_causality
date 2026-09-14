/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The log-normal distribution.

use crate::{RandScalar, StandardNormal, StatsError};
use deep_causality_algebra::{Real, RealField};
use deep_causality_rand::{Distribution, Rng};

/// The log-normal distribution: `exp(X)` where `X ~ N(μ, σ²)`. Supported on `(0, ∞)`.
///
/// `mean = e^{μ + σ²/2}`, `median = e^μ`.
///
/// # The two are not the same
///
/// The mean and the median differ for every `σ > 0`, and confusing them is the usual error with
/// this distribution — the mean sits above the median by a factor of `e^{σ²/2}`. Both are named
/// here and both are pinned by tests, at a `σ` where they differ.
///
/// # Parameterisation
///
/// `mu` and `sigma` are the mean and **standard deviation of the underlying normal**, not of the
/// log-normal itself. `sigma` is the standard deviation rather than the variance, matching
/// [`Normal`](crate::Normal); note that `gaussian_log_density` in this crate takes a variance, and
/// the two conventions are deliberate rather than accidental — a density is usually written with
/// `σ²` and a sampler parameterised with `σ`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogNormal<T> {
    mu: T,
    sigma: T,
}

impl<T> LogNormal<T>
where
    T: RealField,
{
    /// Construct from the underlying normal's mean and standard deviation.
    pub fn new(mu: T, sigma: T) -> Result<Self, StatsError> {
        if !mu.is_finite() || !sigma.is_finite() {
            return Err(StatsError::NonFiniteInput(
                "the log-normal distribution is not defined at a non-finite parameter",
            ));
        }
        if sigma <= T::zero() {
            return Err(StatsError::NonPositiveScale(
                "the log-normal distribution needs a positive standard deviation",
            ));
        }
        Ok(Self { mu, sigma })
    }

    /// The underlying normal's mean `μ`.
    pub fn mu(&self) -> T {
        self.mu
    }

    /// The underlying normal's standard deviation `σ`.
    pub fn sigma(&self) -> T {
        self.sigma
    }
}

/// `exp(μ + σ·Z)` for `Z` standard normal.
impl<T> Distribution<T> for LogNormal<T>
where
    T: RandScalar,
    StandardNormal: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let z: T = StandardNormal.sample(rng);
        Real::exp(self.mu + self.sigma * z)
    }
}
