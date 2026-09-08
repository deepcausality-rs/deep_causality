/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Pearson product-moment correlation.

use crate::algorithms::moments::{max_abs_deviation, mean};
use crate::errors::stats_error::StatsError;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Pearson's `r` between two equal-length samples, with the count of pairs used.
///
/// # Zero variance
///
/// When either sample has zero variance the correlation is undefined — the denominator vanishes —
/// and this returns `Ok((0, n))` rather than an error. That is the absorbed implementation's
/// convention, preserved deliberately: its caller ranks features by `|r|` and a zero rank means
/// "carries no information", which is the right answer for a constant column.
///
/// A sample whose *centred sums* leave the type is not that case and does not take that exit; see
/// the reach note on [`correlate`].
pub fn pearson<T>(x: &[T], y: &[T]) -> Result<(T, usize), StatsError>
where
    T: RealField + FromPrimitive,
{
    if x.len() != y.len() {
        return Err(StatsError::DimensionMismatch(
            "a correlation needs two samples of the same length",
        ));
    }
    correlate(x, y)
}

/// The shared body, over slices already reduced to the complete pairs.
///
/// # Reach
///
/// `r` is invariant under a positive scaling of either sample, and that is what makes the extremes
/// answerable rather than merely survivable. The centred sums `Σdx²`, `Σdy²` and `Σdx·dy` leave the
/// type at both ends well before `r` does: deviations near `1e200` square past `f64`'s maximum, and
/// deviations near `1e-200` square to zero. Either would come back as a `NaN` or as the
/// zero-variance sentinel, and both are wrong for a sample whose correlation is exactly one.
///
/// So the direct sums are formed first, because they are the accurate form, and where they
/// saturate or collapse the deviations are divided by the largest of them before accumulating. The
/// scaled sums lie in `[1, n]`, the scale cancels out of `r`, and only the extremes pay the extra
/// rounding.
fn correlate<T>(x: &[T], y: &[T]) -> Result<(T, usize), StatsError>
where
    T: RealField + FromPrimitive,
{
    if x.is_empty() {
        return Err(StatsError::EmptyInput(
            "a correlation over no pairs is undefined",
        ));
    }
    if x.len() < 2 {
        return Err(StatsError::InsufficientSamples(
            "a correlation needs two pairs: one pair has no dispersion to relate",
        ));
    }
    if x.iter().chain(y.iter()).any(|v| !v.is_finite()) {
        return Err(StatsError::NonFiniteInput(
            "a non-finite observation has no correlation",
        ));
    }

    let n = x.len();

    // Centred sums, in two passes. Forming the means first keeps the computation away from the
    // `Σx² − (Σx)²/n` form, which cancels catastrophically when the mean is large next to the
    // dispersion. `mean` is the same left-to-right sum over the count, and carries its own reach:
    // a column of `f64::MAX` has a representable mean, and an infinite one here would centre the
    // column on nothing and return `NaN` where the documented answer is zero.
    let mean_x = mean(x)?;
    let mean_y = mean(y)?;

    let mut sxy = T::zero();
    let mut sxx = T::zero();
    let mut syy = T::zero();
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }

    let usable = sxx.is_finite() && syy.is_finite() && sxy.is_finite();
    if usable && sxx > T::zero() && syy > T::zero() {
        return Ok((ratio(sxy, sxx, syy), n));
    }

    // Either a constant column, or centred sums that left the type. The largest deviation
    // separates the two: it is zero exactly when the column never varied.
    let scale_x = max_abs_deviation(x, mean_x);
    let scale_y = max_abs_deviation(y, mean_y);
    if scale_x <= T::zero() || scale_y <= T::zero() {
        // Zero variance in one of the samples: the denominator vanishes and `r` is undefined. The
        // absorbed implementation's convention is preserved deliberately — see `pearson`'s doc.
        return Ok((T::zero(), n));
    }

    let mut txy = T::zero();
    let mut txx = T::zero();
    let mut tyy = T::zero();
    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = (xi - mean_x) / scale_x;
        let dy = (yi - mean_y) / scale_y;
        txy += dx * dy;
        txx += dx * dx;
        tyy += dy * dy;
    }
    if txx <= T::zero() || tyy <= T::zero() {
        return Ok((T::zero(), n));
    }
    Ok((ratio(txy, txx, tyy), n))
}

/// `sxy / √(sxx · syy)`, with one square root where the product is representable and two where it
/// is not.
///
/// `(sxx · syy).sqrt()` rounds once, and for `y = ax + b` the product is an exact square, so a
/// perfectly correlated sample returns exactly ±1. The three-rounding form
/// `sqrt(sxx) · sqrt(syy)` returns `0.9999999999999998` there instead.
///
/// But the product leaves the representable range at magnitudes each factor survives, at both
/// ends: centred sums near `1e200` square past the maximum, and denormal ones square to zero,
/// which would divide by it. So the exact form is used where the product is finite and non-zero,
/// and the scaled form — which cannot do either, because each root is taken before the multiply —
/// where it is not. Only the extremes pay the extra rounding.
fn ratio<T>(sxy: T, sxx: T, syy: T) -> T
where
    T: RealField,
{
    let product = sxx * syy;
    if product.is_finite() && product > T::zero() {
        sxy / product.sqrt()
    } else {
        (sxy / sxx.sqrt()) / syy.sqrt()
    }
}

/// Pearson's `r` over the pairs where both samples are present.
///
/// The missing-data policy is pairwise-complete deletion: a pair contributes only when both
/// entries are `Some`, and the returned count says how many did. A caller comparing correlations
/// across column pairs needs that count, because different pairs survive different amounts of
/// deletion and an `r` over ten pairs is not the same evidence as an `r` over a thousand.
pub fn pearson_pairwise_complete<T>(
    x: &[Option<T>],
    y: &[Option<T>],
) -> Result<(T, usize), StatsError>
where
    T: RealField + FromPrimitive,
{
    if x.len() != y.len() {
        return Err(StatsError::DimensionMismatch(
            "a correlation needs two samples of the same length",
        ));
    }
    // Pairwise-complete deletion: a pair survives only when both entries are present.
    let mut xs: Vec<T> = Vec::new();
    let mut ys: Vec<T> = Vec::new();
    for (a, b) in x.iter().zip(y.iter()) {
        if let (Some(&a), Some(&b)) = (a.as_ref(), b.as_ref()) {
            xs.push(a);
            ys.push(b);
        }
    }
    correlate(&xs, &ys)
}
