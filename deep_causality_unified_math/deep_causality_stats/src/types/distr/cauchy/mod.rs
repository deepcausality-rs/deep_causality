/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Cauchy distribution.

use crate::{RandScalar, StandardUniform, StatsError};
use deep_causality_algebra::{Real, RealField};
use deep_causality_rand::{Distribution, Rng};

/// The Cauchy distribution with location `x₀` and scale `γ`, supported on all of `ℝ`.
///
/// # It has no mean and no variance
///
/// Both defining integrals diverge. A sample mean of Cauchy draws does **not** converge as the
/// sample grows — it wanders, and its own distribution is Cauchy again with the same parameters.
///
/// That has a practical consequence for anyone testing this type: an assertion on a sample mean is
/// not a weak test but a **meaningless** one. It passes or fails according to the seed, so it
/// survives review and then fails intermittently in CI. This crate's suite therefore asserts
/// quantiles and carries no moment assertion for this distribution; see the test module's
/// documentation.
///
/// The location is the median and the scale is half the interquartile range, so both parameters
/// are recoverable — just not through moments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cauchy<T> {
    location: T,
    scale: T,
}

impl<T> Cauchy<T>
where
    T: RealField,
{
    /// Construct from a location and a scale.
    pub fn new(location: T, scale: T) -> Result<Self, StatsError> {
        if !location.is_finite() || !scale.is_finite() {
            return Err(StatsError::NonFiniteInput(
                "the Cauchy distribution is not defined at a non-finite parameter",
            ));
        }
        if scale <= T::zero() {
            return Err(StatsError::NonPositiveScale(
                "the Cauchy distribution needs a positive scale",
            ));
        }
        Ok(Self { location, scale })
    }

    /// The location `x₀`, which is the distribution's median.
    pub fn location(&self) -> T {
        self.location
    }

    /// The scale `γ`, which is half the interquartile range.
    pub fn scale(&self) -> T {
        self.scale
    }
}

/// Inverse-CDF: `x₀ + γ·tan(π(u − ½))` for `u` uniform on `[0, 1)`.
///
/// The half-open interval is correct here rather than the open one. `tan` is finite everywhere
/// except at `±π/2`, which `u` reaches only at `0` and `1`; `u = 0` gives `tan(−π/2)`, which in
/// floating point is a large finite number rather than an infinity, and `u = 1` never occurs.
impl<T> Distribution<T> for Cauchy<T>
where
    T: RandScalar,
    StandardUniform: Distribution<T>,
{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let two = T::from_u64(2).expect("2 is representable in every supported scalar");
        let half = T::one() / two;
        let u: T = StandardUniform.sample(rng);
        self.location + self.scale * Real::tan(T::pi() * (u - half))
    }
}
