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
/// alone does not, and it carries it in two places.
///
/// The first is the refusal: `hits > trials` is a caller error that `p̂ > 1` can only report after
/// the division has already lost which count was wrong.
///
/// The second is accuracy, at a scalar narrower than the counts. Converting each count separately
/// and dividing rounds twice, and the two roundings do not cancel: at `BFloat16`, which carries
/// eight significant bits, `257 / 259` becomes `256 / 260` and the quotient lands four places from
/// the correctly rounded one. So the ratio is formed from the counts and rounded once wherever the
/// scalar cannot hold them exactly — and where it can, which is every scalar as wide as the counts
/// are, the exact route is kept rather than routed through an `f64` that would *lose* precision
/// for `Float106`.
///
/// The complement `1 − p̂` is formed from the counts for the same reason, and it is the one that
/// matters most: 1001 of 1002 rounds to exactly one at `BFloat16`, correctly, and `1 − 1` is zero,
/// so a standard error taken from the rounded proportion is zero for a sample that plainly has
/// one. `(trials − hits) / trials` is `1/1002` there, and the width comes back as `1.0e-3`.
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
    let p = quotient::<T>(hits, trials)?;
    let q = quotient::<T>(trials - hits, trials)?;
    Ok((p, (p * q / n).sqrt()))
}

/// `numerator / denominator`, rounded into `T` once.
///
/// Both counts convert exactly when the scalar's significand is at least as wide as they are, and
/// then the quotient carries a single rounding already. Where one of them does not — a count past
/// `2^8` at `BFloat16`, past `2^24` at `f32` — the division is taken at `f64` first, which holds
/// every count under `2^53` exactly, and the result is rounded into `T` once. That double rounding
/// is innocuous for these targets: the intermediate carries more than twice the significand bits
/// of either, which is the condition under which rounding twice agrees with rounding once
/// (S. A. Figueroa, *When is double rounding innocuous?*, ACM SIGNUM Newsletter 30(3), 1995).
fn quotient<T>(numerator: u64, denominator: u64) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    let exact = |v: u64| T::from_u64(v).filter(|t| t.to_u64() == Some(v));
    if let (Some(a), Some(b)) = (exact(numerator), exact(denominator)) {
        return Ok(a / b);
    }
    T::from_f64(numerator as f64 / denominator as f64).ok_or_else(|| {
        StatsError::ConversionFailed("a count is not representable in the working scalar")
    })
}
