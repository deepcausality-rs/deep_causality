/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shannon entropy and conditional entropy over a discrete distribution.

use crate::errors::stats_error::StatsError;
use crate::types::entropy_config::EntropyConfig;
use crate::types::log_base::LogBase;
use crate::types::normalisation::Normalisation;
use crate::types::zero_policy::ZeroPolicy;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Shannon entropy `H = −Σ pᵢ log pᵢ` of a discrete distribution.
///
/// The base, the cutoff for a negligible entry and whether the input is normalised first are all
/// carried by `config`; see [`EntropyConfig`] for why each is a parameter.
///
/// Empty input is refused, as are non-finite and negative entries. A negative value is not a
/// probability and no zero policy interprets one; a `NaN` or an infinity is not a number the
/// surprisal is defined for, and letting either through would return it as though it were an
/// answer.
pub fn entropy<T>(p: &[T], config: &EntropyConfig<T>) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    let (total, rescale, scale) = prepare(p, config)?;
    if total {
        return Ok(T::zero());
    }
    Ok(surprisal_sum(p, config, rescale, scale))
}

/// Validates the input and resolves the normalisation.
///
/// Returns `(degenerate, rescale, scale)`: `degenerate` when the distribution carries no mass and
/// the entropy is zero, otherwise every entry passes through `raw / rescale / scale`.
///
/// # Why the divisor comes in two parts
///
/// `H` is invariant under a positive scaling of the weights, which is the whole point of
/// normalising by the sum — and the sum is the one quantity in this function that can leave the
/// type while every weight is inside it. Two weights at `1e308` are ordinary `f64`s whose entropy
/// is exactly one bit; their sum is `+∞`, every entry divided by it is zero, and the answer comes
/// back as zero entropy from a distribution that has the most a two-outcome distribution can have.
///
/// So an overflowing sum is re-formed through the largest weight: `rescale` is that weight and
/// `scale` is the sum of the weights divided by it, which lies in `[1, n]`. On the ordinary path
/// `rescale` is one, and division by one is exact in binary floating point, so nothing about the
/// ordinary answer changes.
fn prepare<T>(p: &[T], config: &EntropyConfig<T>) -> Result<(bool, T, T), StatsError>
where
    T: RealField + FromPrimitive,
{
    if p.is_empty() {
        return Err(StatsError::EmptyInput(
            "the entropy of no outcomes is undefined",
        ));
    }
    // Checked before the sign test, because `NaN < 0` is false and a NaN would otherwise reach
    // the sum and come back as an answer. `±inf` lands here too: "not a number at all" is a
    // different complaint from "a number that is not a probability".
    if p.iter().any(|&x| !x.is_finite()) {
        return Err(StatsError::NonFiniteInput(
            "a non-finite entry has no surprisal, and no distribution contains one",
        ));
    }
    if p.iter().any(|&x| x < T::zero()) {
        return Err(StatsError::NegativeProbability(
            "a negative entry is not a probability, and no zero policy interprets one",
        ));
    }

    match config.normalisation {
        Normalisation::None => Ok((false, T::one(), T::one())),
        Normalisation::BySum { floor } => {
            let sum = p.iter().fold(T::zero(), |acc, &x| acc + x);
            // No mass to measure: there is no distribution here, and dividing by a sum this
            // small would manufacture one out of rounding.
            if sum <= floor {
                return Ok((true, T::one(), T::one()));
            }
            if sum.is_finite() {
                return Ok((false, T::one(), sum));
            }
            // The mass is beyond the type's reach although every weight is inside it. The largest
            // weight is strictly positive here: entries are non-negative and an all-zero slice
            // sums to zero, which is finite.
            let largest = p.iter().fold(T::zero(), |m, &x| if x > m { x } else { m });
            let scaled = p.iter().fold(T::zero(), |acc, &x| acc + x / largest);
            Ok((false, largest, scaled))
        }
    }
}

/// `−Σ pᵢ log pᵢ` over the entries the zero policy keeps.
///
/// Every entry passes through `raw / rescale / scale`; see [`prepare`] for why the divisor arrives
/// in two parts, and why dividing twice costs the ordinary path nothing.
fn surprisal_sum<T>(p: &[T], config: &EntropyConfig<T>, rescale: T, scale: T) -> T
where
    T: RealField + FromPrimitive,
{
    let ln_base = match config.base {
        // `log2 x = ln x / ln 2`, and `ln 2` is taken from the scalar's own logarithm rather than
        // from a literal, so it carries the working precision rather than `f64`'s.
        LogBase::Bits => (T::one() + T::one()).ln(),
        LogBase::Nats => T::one(),
    };

    let acc = p.iter().fold(T::zero(), |acc, &raw| {
        let q = raw / rescale / scale;
        // `lim(p → 0) p·log p = 0`, so an entry at zero contributes nothing whatever the policy
        // says. The test is outside the policy rather than inside `SkipBelow`, because a
        // *negative* threshold admits the exact zeros — it is the reading of "omit entries at or
        // below the threshold" that omits none of them — and `0 · ln 0` is `0 · (−∞)`, which is a
        // `NaN` and not the limit. A caller passing a negative threshold is saying "keep
        // everything", not "return me a NaN".
        if q <= T::zero() {
            return acc;
        }
        let keep = match config.zero_policy {
            // An entry at the cutoff contributes nothing, by the same limit.
            ZeroPolicy::SkipZero => true,
            ZeroPolicy::SkipBelow(threshold) => q > threshold,
        };
        if keep { acc - q * q.ln() } else { acc }
    });
    acc / ln_base
}

/// Conditional entropy `H(X | Y) = H(X, Y) − H(Y)`.
///
/// Takes the joint distribution and the conditioning marginal, both already reduced to slices.
/// Marginalising a joint over chosen axes is the caller's job: this crate is over slices, and the
/// axis machinery belongs with the container that has axes.
///
/// Both are measured under the same `config`, so the difference is in one unit.
pub fn conditional_entropy<T>(
    joint: &[T],
    conditioning: &[T],
    config: &EntropyConfig<T>,
) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    let h_joint = entropy(joint, config)?;
    let h_conditioning = entropy(conditioning, config)?;
    Ok(h_joint - h_conditioning)
}
