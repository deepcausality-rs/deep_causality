/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Gaussian log-density.

use crate::errors::stats_error::StatsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The log-density of a normal distribution at `x`.
///
/// # Parameterisation
///
/// The third parameter is the **variance** `σ²`, not the standard deviation. The two are both
/// plausible readings of a scale argument and they give different answers everywhere except
/// `σ = 1`, so the choice is named here and pinned by a test at a point where they differ.
///
/// `log N(x | μ, σ²) = −½·log(2πσ²) − (x − μ)² / (2σ²)`
///
/// A non-positive variance is refused: the density is not defined for one, and returning a
/// plausible number would hide a caller that computed its scale wrongly.
pub fn gaussian_log_density<T>(x: T, mean: T, variance: T) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    if !variance.is_finite() || !mean.is_finite() || !x.is_finite() {
        return Err(StatsError::NonFiniteInput(
            "the normal density is not defined at a non-finite argument or parameter",
        ));
    }
    if variance <= T::zero() {
        return Err(StatsError::NonPositiveScale(
            "the normal density needs a positive variance",
        ));
    }

    // log N(x | μ, σ²) = −½·log(2πσ²) − (x − μ)² / (2σ²)
    //
    // The logarithm is taken of the variance rather than the density, which is the reason this
    // function exists: at a large deviation the density underflows to zero while its logarithm
    // stays finite and informative.
    let two = T::one() + T::one();
    let d = x - mean;
    let half = T::one() / two;
    Ok(-half * (two * T::pi() * variance).ln() - (d * d) / (two * variance))
}
