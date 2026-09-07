/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::stats_error::StatsError;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// A mean over observations that arrive one at a time.
///
/// [`mean`](crate::mean) needs a slice, which a caller that *generates* its observations does not
/// have and should not be made to build: a Monte-Carlo estimator over a million draws would hold
/// eight megabytes to compute one scalar. This holds a running sum and a count instead, so the
/// memory is constant in the number of observations.
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
/// The sum is formed left to right, exactly as [`mean`](crate::mean) folds a slice, so the same
/// observations in the same order give the same answer to the last bit. Feeding a slice through
/// this type is a way of getting `mean` without the slice, not a different estimator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeanAccumulator<T> {
    sum: T,
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
            sum: T::zero(),
            count: 0,
        }
    }

    /// Adds one observation.
    pub fn push(&mut self, value: T) {
        self.sum += value;
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
        Ok(self.sum / n)
    }
}
