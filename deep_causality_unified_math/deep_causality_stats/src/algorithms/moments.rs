/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Mean, variance and standard deviation over a sample.
//!
//! The variance here is the corrected (`n − 1`) form, and it is the only one built. Every variance
//! in this workspace applies Bessel's correction; there is no population `÷n` caller, and this
//! crate implements only what something calls.
//!
//! # An intermediate that leaves the type is a defect, not a limitation
//!
//! Every reduction below is written so that an answer the working scalar can hold is returned as a
//! number rather than as an infinity. The direct sum is kept as the primary form — it is the most
//! accurate and the cheapest — and a rescaled second pass runs only where it saturated. That
//! second pass costs nothing on the ordinary path, and on the extreme one it is the difference
//! between `mean([3e38, 3e38])` returning `3e38` at `BFloat16` and returning `+∞`.

use crate::errors::stats_error::StatsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The arithmetic mean.
///
/// Refuses the empty sample: the mean of no observations is not zero, it is undefined, and
/// returning zero would put a plausible number where there is no answer.
///
/// # The sum's reach is not the mean's
///
/// The mean of a finite sample is a convex combination of it, so it is bounded in magnitude by the
/// largest observation and is always representable. The left-to-right sum is not: two observations
/// at `3e38` overflow `BFloat16` while their mean does not. Where the sum saturates, the mean is
/// re-formed through the largest magnitude in the sample — every scaled observation then lies in
/// `[−1, 1]`, the scaled sum is bounded by `n`, and multiplying the scaled mean back by the scale
/// last cannot leave the type because the result is bounded by the scale itself.
///
/// A sample that carries a non-finite observation is a different case, and it keeps the saturated
/// sum: `±∞` and `NaN` are the answer there rather than an artefact of the intermediate, and the
/// rescaling would turn an honest infinity into a `NaN`.
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
    if sum.is_finite() || xs.iter().any(|x| !x.is_finite()) {
        return Ok(sum / n);
    }

    // The sum left the type although every observation is inside it. Re-form through the largest
    // magnitude. It is strictly positive here: an all-zero sample sums to zero, which is finite.
    let scale = max_abs_deviation(xs, T::zero());
    let scaled = xs.iter().fold(T::zero(), |acc, &x| acc + x / scale);
    Ok(scaled / n * scale)
}

/// A count on the real axis, or a typed error when the working scalar cannot hold it.
fn count<T: FromPrimitive>(n: usize) -> Result<T, StatsError> {
    T::from_usize(n).ok_or_else(|| {
        StatsError::ConversionFailed(
            "an observation count is not representable in the working scalar",
        )
    })
}

/// The largest `|xᵢ − centre|` in the sample, or zero for an empty one.
///
/// The scale a rescaled reduction divides by. Passing `centre = 0` gives the largest magnitude in
/// the sample itself.
pub(crate) fn max_abs_deviation<T>(xs: &[T], centre: T) -> T
where
    T: RealField,
{
    xs.iter().fold(T::zero(), |m, &x| {
        let d = (x - centre).abs();
        if d > m { d } else { m }
    })
}

/// `Σ(xᵢ − centre)² / denominator`, formed so that a representable answer comes back as one.
///
/// The direct sum first, because it is the accurate form. A single `d²` can leave the type while
/// the quotient stays well inside it — one deviation of `1e20` among a hundred observations
/// overflows `BFloat16`'s `3.4e38` when squared, and `Σd²/(n − 1)` is then about `1e38`, which the
/// type holds — so where the sum saturates the deviations are divided by the largest of them
/// before squaring. The scaled sum lies in `[1, n]`, and the two multiplications back are ordered
/// so the quotient is applied first: `(r/denominator · s) · s` stays inside the type whenever the
/// answer does.
fn dispersion<T>(xs: &[T], centre: T, denominator: T) -> T
where
    T: RealField,
{
    let ss = xs.iter().fold(T::zero(), |acc, &x| {
        let d = x - centre;
        acc + d * d
    });
    if ss.is_finite() || !centre.is_finite() || xs.iter().any(|x| !x.is_finite()) {
        return ss / denominator;
    }

    let scale = max_abs_deviation(xs, centre);
    if scale <= T::zero() {
        // Every deviation is zero, so the sum was zero and finite. Unreachable, and cheaper to
        // answer than to argue about.
        return ss / denominator;
    }
    let r = xs.iter().fold(T::zero(), |acc, &x| {
        let d = (x - centre) / scale;
        acc + d * d
    });
    r / denominator * scale * scale
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
    Ok(dispersion(xs, m, count::<T>(xs.len() - 1)?))
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
    Ok(dispersion(xs, m, count::<T>(xs.len())?))
}

/// The population standard deviation: the square root of [`population_variance`].
pub fn population_std_dev<T>(xs: &[T]) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    Ok(population_variance(xs)?.sqrt())
}
