/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Mean, variance and standard deviation over a sample.
//!
//! The variance here is the corrected (`n − 1`) form, and it is the only one built. Every variance
//! in this workspace applies Bessel's correction; there is no population `÷n` caller, and this
//! crate implements only what something calls.

use crate::errors::stats_error::StatsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The arithmetic mean.
///
/// Refuses the empty sample: the mean of no observations is not zero, it is undefined, and
/// returning zero would put a plausible number where there is no answer.
pub fn mean<T>(xs: &[T]) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    if xs.is_empty() {
        return Err(StatsError::EmptyInput(
            "the mean of no observations is undefined",
        ));
    }
    let n = count::<T>(xs.len())?;
    let sum = xs.iter().fold(T::zero(), |acc, &x| acc + x);
    Ok(sum / n)
}

/// A count on the real axis, or a typed error when the working scalar cannot hold it.
fn count<T: FromPrimitive>(n: usize) -> Result<T, StatsError> {
    T::from_usize(n).ok_or_else(|| {
        StatsError::ConversionFailed(
            "an observation count is not representable in the working scalar",
        )
    })
}

/// The sample variance, with Bessel's correction: `Σ(xᵢ − x̄)² / (n − 1)`.
///
/// Refuses a sample of fewer than two observations. With one, the divisor is zero and there is no
/// dispersion to estimate.
pub fn variance<T>(xs: &[T]) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    if xs.is_empty() {
        return Err(StatsError::EmptyInput(
            "the variance of no observations is undefined",
        ));
    }
    if xs.len() < 2 {
        return Err(StatsError::InsufficientSamples(
            "the corrected variance needs two observations: with one the divisor n − 1 is zero",
        ));
    }

    // Two passes, deliberately. Forming the mean first and then the deviations keeps `Σxᵢ²` out of
    // the computation, so a sample whose squares overflow the type still has a variance, and a
    // small dispersion around a large mean survives the cancellation that a one-pass
    // `Σxᵢ² − nx̄²` would lose it to.
    let m = mean(xs)?;
    let ss = xs.iter().fold(T::zero(), |acc, &x| {
        let d = x - m;
        acc + d * d
    });
    Ok(ss / count::<T>(xs.len() - 1)?)
}

/// The sample standard deviation: the square root of [`variance`].
pub fn std_dev<T>(xs: &[T]) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    Ok(variance(xs)?.sqrt())
}

/// The population variance: `Σ(xᵢ − x̄)² / n`.
///
/// The uncorrected form, dividing by `n` rather than `n − 1`. It is the variance *of the sample
/// itself*, treated as the whole population, rather than an estimate of the variance of a
/// population the sample was drawn from.
///
/// # When this is the one you want
///
/// Standardising features to zero mean and unit scale: the divisor cancels out of the
/// standardisation, and the population form is the convention every library uses there. Also any
/// case where the data *is* the population — every measurement in a run, rather than a draw from
/// something larger.
///
/// Unlike [`variance`], a single observation is not refused: a population of one has zero spread,
/// which is a fact about it rather than a missing estimate. Only the empty sample is refused,
/// because there is nothing to take the variance of.
pub fn population_variance<T>(xs: &[T]) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    if xs.is_empty() {
        return Err(StatsError::EmptyInput(
            "the variance of no observations is undefined",
        ));
    }
    // Two passes, for the reason `variance` takes two: forming the mean first keeps `Σxᵢ²` out of
    // the computation, so a small spread around a large mean survives.
    let m = mean(xs)?;
    let ss = xs.iter().fold(T::zero(), |acc, &x| {
        let d = x - m;
        acc + d * d
    });
    Ok(ss / count::<T>(xs.len())?)
}

/// The population standard deviation: the square root of [`population_variance`].
pub fn population_std_dev<T>(xs: &[T]) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    Ok(population_variance(xs)?.sqrt())
}
