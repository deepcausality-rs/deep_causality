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
    let (total, scale) = prepare(p, config)?;
    if total {
        return Ok(T::zero());
    }
    Ok(surprisal_sum(p, config, scale))
}

/// Validates the input and resolves the normalisation.
///
/// Returns `(degenerate, scale)`: `degenerate` when the distribution carries no mass and the
/// entropy is zero, otherwise `scale` is the divisor every entry passes through.
fn prepare<T>(p: &[T], config: &EntropyConfig<T>) -> Result<(bool, T), StatsError>
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
        Normalisation::None => Ok((false, T::one())),
        Normalisation::BySum { floor } => {
            let sum = p.iter().fold(T::zero(), |acc, &x| acc + x);
            // No mass to measure: there is no distribution here, and dividing by a sum this
            // small would manufacture one out of rounding.
            if sum <= floor {
                Ok((true, T::one()))
            } else {
                Ok((false, sum))
            }
        }
    }
}

/// `−Σ pᵢ log pᵢ` over the entries the zero policy keeps.
fn surprisal_sum<T>(p: &[T], config: &EntropyConfig<T>, scale: T) -> T
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
        let q = raw / scale;
        let keep = match config.zero_policy {
            // `lim(p → 0) p·log p = 0`, so an entry at the cutoff contributes nothing.
            ZeroPolicy::SkipZero => q > T::zero(),
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
