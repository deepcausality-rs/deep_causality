/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Pearson product-moment correlation.

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
    let n_t = T::from_usize(n).ok_or_else(|| {
        StatsError::ConversionFailed("a pair count is not representable in the working scalar")
    })?;

    // Centred sums, in two passes. Forming the means first keeps the computation away from the
    // `Σx² − (Σx)²/n` form, which cancels catastrophically when the mean is large next to the
    // dispersion.
    let mean_x = x.iter().fold(T::zero(), |a, &v| a + v) / n_t;
    let mean_y = y.iter().fold(T::zero(), |a, &v| a + v) / n_t;

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

    // Zero variance in either sample: the denominator vanishes and `r` is undefined. The absorbed
    // implementation's convention is preserved deliberately — see this function's doc.
    if sxx <= T::zero() || syy <= T::zero() {
        return Ok((T::zero(), n));
    }
    // One square root where the product is representable, two where it is not.
    //
    // `(sxx · syy).sqrt()` rounds once, and for `y = ax + b` the product is an exact square, so a
    // perfectly correlated sample returns exactly ±1. The three-rounding form
    // `sqrt(sxx) · sqrt(syy)` returns `0.9999999999999998` there instead.
    //
    // But the product leaves the representable range at magnitudes each factor survives, at both
    // ends: centred sums near `1e200` square past the maximum, and denormal ones square to zero,
    // which would divide by it. So the exact form is used where the product is finite and
    // non-zero, and the scaled form — which cannot do either, because each root is taken before
    // the multiply — where it is not. Only the extremes pay the extra rounding.
    let product = sxx * syy;
    let r = if product.is_finite() && product > T::zero() {
        sxy / product.sqrt()
    } else {
        (sxy / sxx.sqrt()) / syy.sqrt()
    };
    Ok((r, n))
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
