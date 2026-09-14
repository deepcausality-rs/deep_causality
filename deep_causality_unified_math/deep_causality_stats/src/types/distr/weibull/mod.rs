/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Weibull distribution.

use crate::{Open01, RandScalar, StatsError};
use deep_causality_algebra::{Real, RealField};
use deep_causality_rand::{Distribution, Rng};

/// The Weibull distribution with shape `k` and scale `λ`, supported on `[0, ∞)`.
///
/// `mean = λ·Γ(1 + 1/k)`.
///
/// # The exponential is the `k = 1` case
///
/// At `k = 1` the Weibull **is** the exponential with rate `1/λ`. That identity is analytic rather
/// than a second implementation, which makes it the one legitimate cross-check available in this
/// group: agreement there is evidence, not circularity.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weibull<T> {
    shape: T,
    scale: T,
}

impl<T> Weibull<T>
where
    T: RealField,
{
    /// Construct from a shape and a scale.
    pub fn new(shape: T, scale: T) -> Result<Self, StatsError> {
        if !shape.is_finite() || !scale.is_finite() {
            return Err(StatsError::NonFiniteInput(
                "the Weibull distribution is not defined at a non-finite parameter",
            ));
        }
        if shape <= T::zero() || scale <= T::zero() {
            return Err(StatsError::NonPositiveScale(
                "the Weibull distribution needs a positive shape and scale",
            ));
        }
        Ok(Self { shape, scale })
    }

    /// The shape `k`.
    pub fn shape(&self) -> T {
        self.shape
    }

    /// The scale `λ`.
    pub fn scale(&self) -> T {
        self.scale
    }
}

/// Inverse-CDF: `λ·(−ln u)^{1/k}` for `u` uniform on `(0, 1)`.
///
/// The exponent is `1/k`, not `k`. The two agree only at `k = 1`, which is why the suite checks a
/// second shape.
impl<T> Distribution<T> for Weibull<T>
where
    T: RandScalar,
    Open01: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let u: T = Open01.sample(rng);
        self.scale * Real::powf(-Real::ln(u), T::one() / self.shape)
    }
}
