/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::stats_error::StatsError;
use crate::types::pairwise_sum::PairwiseSum;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// A mean over observations that arrive one at a time.
///
/// [`mean`](crate::mean) needs a slice, which a caller that *generates* its observations does not
/// have and should not be made to build: a Monte-Carlo estimator over a million draws would hold
/// eight megabytes to compute one scalar. This holds a bounded set of partial sums and a count
/// instead, so the memory is constant in the number of observations — one slot per bit of the
/// count, which is `1032` bytes at `f64` against the eight megabytes, and does not grow with the
/// millionth draw any more than with the second.
///
/// It is the same relationship [`fit_ridge_streaming`](crate::fit_ridge_streaming) has to
/// [`fit_ridge`](crate::fit_ridge): the statistic is identical, the shape of the input is not.
///
/// ```
/// use deep_causality_stats::MeanAccumulator;
///
/// let mut acc = MeanAccumulator::<f64>::new();
/// for x in [1.0, 2.0, 3.0, 4.0] {
///     acc.push(x);
/// }
/// assert_eq!(acc.mean().unwrap(), 2.5);
/// ```
///
/// # Accumulation order
///
/// The sum is formed as a balanced tree by [`PairwiseSum`], exactly as [`mean`](crate::mean) sums a
/// slice, so the same observations in the same order give the same answer to the last bit. Feeding
/// a slice through this type is a way of getting `mean` without the slice, not a different
/// estimator. That agreement is a property of the arrangement rather than a coincidence: which
/// partial sum lands in which slot is decided by the count, which both callers share.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeanAccumulator<T> {
    sum: PairwiseSum<T>,
    count: usize,
}

impl<T: RealField + FromPrimitive> Default for MeanAccumulator<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: RealField + FromPrimitive> MeanAccumulator<T> {
    /// An accumulator over no observations.
    pub fn new() -> Self {
        Self {
            sum: PairwiseSum::new(),
            count: 0,
        }
    }

    /// Adds one observation.
    pub fn push(&mut self, value: T) {
        self.sum.push(value);
        self.count += 1;
    }

    /// How many observations have been added.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The mean so far, or `EmptyInput` when nothing has been added.
    ///
    /// Refuses the empty case for the same reason [`mean`](crate::mean) does: the mean of no
    /// observations is undefined, and a caller that wants a zero there should say so itself.
    pub fn mean(&self) -> Result<T, StatsError> {
        if self.count == 0 {
            return Err(StatsError::EmptyInput(
                "the mean of no observations is undefined",
            ));
        }
        let n = T::from_usize(self.count).ok_or_else(|| {
            StatsError::ConversionFailed(
                "an observation count is not representable in the working scalar",
            )
        })?;
        Ok(self.sum.total() / n)
    }
}
