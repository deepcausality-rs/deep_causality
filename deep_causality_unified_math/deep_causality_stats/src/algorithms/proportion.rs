/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Bernoulli proportions and their sampling error.

use crate::errors::stats_error::StatsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// The standard error of a Bernoulli proportion: `√(p(1 − p) / n)`.
///
/// The spread of the sampling distribution of `p̂` over `n` independent trials — the width to read
/// an estimated proportion against, and the quantity that shrinks as `1/√n`.
///
/// # What it refuses, and why
///
/// A proportion outside `[0, 1]` is not one, and `p(1 − p)` would be negative there, so the square
/// root has no real value. Zero trials give no sampling distribution to have a width. Both are
/// refused rather than returned as a `NaN` that a caller may not inspect.
///
/// The two endpoints `p = 0` and `p = 1` are *accepted* and both give exactly zero. That is the
/// honest answer to the question asked — the observed variance of a sample that never varied — and
/// not a claim that the true proportion is certain. A caller that wants an interval which stays
/// open at the endpoints wants a different estimator (Wilson, Agresti–Coull), not a different
/// standard error.
pub fn bernoulli_standard_error<T>(proportion: T, trials: u64) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    if !proportion.is_finite() {
        return Err(StatsError::NonFiniteInput(
            "a non-finite proportion has no sampling error",
        ));
    }
    if proportion < T::zero() || proportion > T::one() {
        return Err(StatsError::NegativeProbability(
            "a Bernoulli proportion lies in [0, 1]: outside it, p(1 − p) is negative and its \
             square root is not real",
        ));
    }
    if trials == 0 {
        return Err(StatsError::EmptyInput(
            "no trials, so there is no sampling distribution to have a width",
        ));
    }
    let n = T::from_u64(trials).ok_or_else(|| {
        StatsError::ConversionFailed("a trial count is not representable in the working scalar")
    })?;
    Ok((proportion * (T::one() - proportion) / n).sqrt())
}

/// A proportion and its standard error from counts: `p̂ = hits / trials`.
///
/// Separate from [`bernoulli_standard_error`] because the counts carry information the proportion
/// alone does not — `hits > trials` is a caller error that `p̂ > 1` can only report after the
/// division has already lost which count was wrong.
pub fn bernoulli_proportion<T>(hits: u64, trials: u64) -> Result<(T, T), StatsError>
where
    T: RealField + FromPrimitive,
{
    if trials == 0 {
        return Err(StatsError::EmptyInput(
            "a proportion of no trials is undefined",
        ));
    }
    if hits > trials {
        return Err(StatsError::DimensionMismatch(
            "more hits than trials: a proportion cannot exceed one",
        ));
    }
    let n = T::from_u64(trials).ok_or_else(|| {
        StatsError::ConversionFailed("a trial count is not representable in the working scalar")
    })?;
    let k = T::from_u64(hits).ok_or_else(|| {
        StatsError::ConversionFailed("a hit count is not representable in the working scalar")
    })?;
    let p = k / n;
    Ok((p, bernoulli_standard_error(p, trials)?))
}
