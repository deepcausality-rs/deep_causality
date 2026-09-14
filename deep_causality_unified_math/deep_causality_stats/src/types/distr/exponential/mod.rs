/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The exponential distribution.

use crate::{Open01, RandScalar, StatsError};
use deep_causality_algebra::{Real, RealField};
use deep_causality_rand::{Distribution, Rng};

/// The exponential distribution with rate `λ`, supported on `[0, ∞)`.
///
/// `mean = 1/λ`, `variance = 1/λ²`, density `λ·e^{−λx}`.
///
/// # Parameterisation
///
/// The parameter is the **rate** `λ`, not the scale `1/λ`. The two are both plausible readings of
/// a single argument and give different answers everywhere except `λ = 1`, so the choice is named
/// here and pinned by a test at a rate where they differ.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Exponential<T> {
    rate: T,
}

impl<T> Exponential<T>
where
    T: RealField,
{
    /// Construct from a rate.
    ///
    /// Refuses a non-positive or non-finite rate: the distribution is not defined for one, and
    /// returning a plausible number would hide a caller that computed its rate wrongly.
    pub fn new(rate: T) -> Result<Self, StatsError> {
        if !rate.is_finite() {
            return Err(StatsError::NonFiniteInput(
                "the exponential distribution is not defined at a non-finite rate",
            ));
        }
        if rate <= T::zero() {
            return Err(StatsError::NonPositiveScale(
                "the exponential distribution needs a positive rate",
            ));
        }
        Ok(Self { rate })
    }

    /// The rate `λ`.
    pub fn rate(&self) -> T {
        self.rate
    }
}

/// Inverse-CDF: `x = −ln(u) / λ` for `u` uniform on `(0, 1)`.
///
/// The draw comes from the **open** interval. `ln(0)` is an infinity, and a zero from a half-open
/// draw has probability `2^-53` — rare enough that no sampling test would find it and common
/// enough to matter over a long run.
impl<T> Distribution<T> for Exponential<T>
where
    T: RandScalar,
    Open01: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let u: T = Open01.sample(rng);
        -Real::ln(u) / self.rate
    }
}
