/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Discretisation into equal-width and equal-frequency bins.
//!
//! Both return the bin index of each observation, as a value in the working scalar, in the input's
//! order. Both use the half-open convention `[lower, upper)` for every bin except the last, which
//! is closed so the maximum falls inside the range rather than one bin past its end.

use crate::errors::stats_error::StatsError;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::{FromPrimitive, ToPrimitive};

/// Bins by dividing the observed range into `bins` intervals of equal width.
///
/// A constant column has zero range and no width to divide. It is not an error — a constant is a
/// legitimate column — and every observation falls in bin zero.
///
/// Fewer than two bins is refused: one bin is not a discretisation, and zero is not a partition.
/// A non-finite observation is refused, because it has no place on the range.
///
/// # A span the range cannot hold
///
/// Observations reaching both ends of the type — `f64::MIN` and `f64::MAX` in one column — are
/// each finite, and each has a bin, but `hi − lo` between them is `+∞`. The quotient
/// `(x − lo) / (hi − lo)` is then `∞/∞` at the maximum, so the observation that defines the top of
/// the range comes back as a `NaN` bin index. Halving both endpoints first is exact in binary
/// floating point and brings the span back inside the type, and the quotient is unchanged by it,
/// so the halved form is used exactly where the direct one overflows.
pub fn bin_equal_width<T>(data: &[T], bins: usize) -> Result<Vec<T>, StatsError>
where
    T: RealField + FromPrimitive,
{
    let n = validate(data, bins)?;

    let mut lo = data[0];
    let mut hi = data[0];
    for &x in data {
        if x < lo {
            lo = x;
        }
        if x > hi {
            hi = x;
        }
    }

    let span = hi - lo;
    // A constant column has no range to divide. Not an error: a constant is a legitimate column,
    // and every observation belongs to the same bin.
    if span <= T::zero() {
        return Ok(alloc::vec![T::zero(); n]);
    }

    // The span overflows only where the observations reach both ends of the type. Halving is exact
    // above the subnormals, and a column that wide has nothing near them.
    let two = T::one() + T::one();
    let halve = !span.is_finite();
    let (origin, width) = if halve {
        (lo / two, hi / two - lo / two)
    } else {
        (lo, span)
    };

    let bins_t = index::<T>(bins)?;
    let last = index::<T>(bins - 1)?;
    let mut out = alloc::vec::Vec::with_capacity(n);
    for &x in data {
        // Half-open `[lower, upper)` for every bin, so a value on an interior edge belongs to the
        // bin above it. The last bin is closed, which the clamp supplies: the maximum's quotient
        // is exactly `bins`, one past the end, and it belongs in `bins - 1`.
        let scaled = if halve { x / two } else { x };
        let q = ((scaled - origin) / width * bins_t).floor();
        out.push(if q > last { last } else { q });
    }
    Ok(out)
}

/// Shared validation. Returns the observation count.
fn validate<T>(data: &[T], bins: usize) -> Result<usize, StatsError>
where
    T: RealField + FromPrimitive,
{
    if data.is_empty() {
        return Err(StatsError::EmptyInput("no observations to bin"));
    }
    if bins < 2 {
        return Err(StatsError::InvalidBinCount(
            "a bin count below two is not a partition",
        ));
    }
    if bins > data.len() {
        return Err(StatsError::InvalidBinCount(
            "more bins than observations: n observations support at most n non-empty bins",
        ));
    }
    if data.iter().any(|x| !x.is_finite()) {
        return Err(StatsError::NonFiniteInput(
            "a non-finite observation has no place on the range",
        ));
    }
    Ok(data.len())
}

/// A bin index on the real axis, or a typed error when the scalar cannot hold it.
///
/// "Cannot hold it" means cannot hold it *exactly*. `from_usize` rounds rather than refusing, and a
/// rounded bin count is a different partition from the one asked for: `BFloat16` carries eight
/// significant bits, so 257 arrives as 256, and a caller asking for 257 equal-width bins would
/// silently get the boundaries of 256 of them. The round trip back to `usize` is what separates a
/// count the scalar represents from one it merely approximates.
fn index<T: FromPrimitive + ToPrimitive>(i: usize) -> Result<T, StatsError> {
    let v = T::from_usize(i).ok_or_else(|| {
        StatsError::ConversionFailed("a bin index is not representable in the working scalar")
    })?;
    if v.to_usize() == Some(i) {
        Ok(v)
    } else {
        Err(StatsError::ConversionFailed(
            "a bin index is not representable in the working scalar: it would round to a \
             different count and partition the range differently",
        ))
    }
}

/// Bins so that each bin holds as near as possible the same number of observations.
///
/// When `bins` divides the observation count the bins are exactly equal. When it does not, the
/// remainder is spread rather than concentrated in one bin.
///
/// Ties are the case that stops the bins being equal: identical values cannot be split across a
/// boundary, so a column with heavy repetition yields uneven bins by necessity, not by defect.
pub fn bin_equal_frequency<T>(data: &[T], bins: usize) -> Result<Vec<T>, StatsError>
where
    T: RealField + FromPrimitive,
{
    let n = validate(data, bins)?;

    // Rank each observation by its position in the sorted order, resolving ties to the FIRST
    // occurrence: identical values cannot straddle a boundary, so they share a bin by necessity.
    let mut order: alloc::vec::Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        data[a]
            .partial_cmp(&data[b])
            .expect("non-finite observations are refused above")
    });

    let mut out = alloc::vec![T::zero(); n];
    let mut start = 0usize;
    while start < n {
        // The block of equal values beginning at `start`.
        let mut end = start;
        while end + 1 < n && data[order[end + 1]] == data[order[start]] {
            end += 1;
        }

        // The block takes the bin holding the LARGEST SHARE of its ranks; a tie goes to the
        // lower bin.
        //
        // A block of equal values cannot be split across a boundary, so when it spans several
        // bins one of them has to take it, and the share rule is the choice that leaves the bins
        // least uneven. Neither simpler rule works on both of the cases that arise: taking the
        // block's first rank sends the four copies in `[1, 2, 2, 2, 2, 3]` into bin 0 beside the
        // minimum, and taking its midpoint sends the six copies in
        // `[1, 1, 1, 1, 1, 1, 2, 3, 4, 5]` out of bin 0 even though they contain the minimum.
        // `rank · bins / n` is strictly below `bins` for every `rank < n`, so no clamp is needed
        // here — unlike the equal-width path, where the maximum's quotient is exactly `bins`.
        // The block's first rank opens the first run, so the scan starts one past it. Seeding the
        // run from that rank rather than from an empty run keeps this the answer outright for a
        // block of one, instead of a value the first iteration would overwrite either way.
        let mut bin = start * bins / n;
        let mut share = 1usize;
        let mut best_bin = bin;
        let mut best_share = 1usize;
        for rank in (start + 1)..=end {
            let b = rank * bins / n;
            if b == bin {
                share += 1;
            } else {
                bin = b;
                share = 1;
            }
            // Strictly greater, so a tie keeps the lower bin already held.
            if share > best_share {
                best_share = share;
                best_bin = bin;
            }
        }
        let value = index::<T>(best_bin)?;
        for &idx in &order[start..=end] {
            out[idx] = value;
        }
        start = end + 1;
    }
    Ok(out)
}
