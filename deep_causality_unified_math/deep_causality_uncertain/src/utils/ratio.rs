/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A probability, in the caller's scalar.

use crate::UncertainError;
use deep_causality_rand::RandScalar;

/// The fraction `hits / total`, formed at `R` rather than at a fixed precision and lowered.
///
/// A probability is a ratio of two counts, which is why the scalar bound has to be a field:
/// `Real` in `deep_causality_algebra` is a commutative ring with an order and carries no `Div`, so
/// the weakest structure that can state a frequency is `RealField`. That is the whole reason
/// `RandScalar` and not `Real` is the bound on this crate.
///
/// # Errors
///
/// `SamplingError` when a count does not convert to `R`. Both counts are `usize`, and a scalar
/// whose range cannot hold the sample count cannot express the estimate either — reporting it is
/// what stops a silently saturated count from becoming a plausible probability.
pub(crate) fn ratio<R: RandScalar>(hits: usize, total: usize) -> Result<R, UncertainError> {
    if total == 0 {
        return Ok(R::zero());
    }
    Ok(lift_count::<R>(hits)? / lift_count::<R>(total)?)
}

/// One count, as a scalar.
///
/// Separate from [`ratio`] because the SPRT needs its two counts individually — the ratio it forms
/// is not `hits / n` — and because a count that does not convert is the same defect either way.
///
/// # The refusal is unreachable at every shipped scalar, and is kept anyway
///
/// No scalar in the workspace can fail this conversion: `BFloat16::from_usize` returns `Some`
/// unconditionally, and `f32`, `f64` and `Float106` delegate to conversions that accept any
/// `usize` — rounding a large one rather than refusing it. So the `None` arm is not covered by the
/// test suite and cannot be without a scalar written to fail it.
///
/// It stays because `FromPrimitive::from_usize` returns an `Option` and the honest response to one
/// is to handle it. The alternative is `unwrap`, which turns a scalar added tomorrow with a
/// narrower integer range into a panic in the middle of an estimator, or a silent fallback, which
/// turns it into a plausible wrong probability. Reporting is the only one of the three that stays
/// truthful for a type that does not exist yet.
pub(crate) fn lift_count<R: RandScalar>(n: usize) -> Result<R, UncertainError> {
    R::from_usize(n).ok_or_else(|| {
        UncertainError::SamplingError(format!(
            "a sample count of {n} does not convert to the graph's scalar"
        ))
    })
}
