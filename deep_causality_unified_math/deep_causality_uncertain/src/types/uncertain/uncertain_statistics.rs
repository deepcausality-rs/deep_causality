/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ProbabilisticType, QmcSampler, Uncertain, UncertainError};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_stats::{MeanAccumulator, StatsError, std_dev};

// Precision-generic Monte-Carlo statistics. Unlike the sampling surface (which needs only
// `ProbabilisticType`), these reduce many samples into one scalar, so they require the value
// type to be a real field with arithmetic and a square root — which also keeps them off
// `Uncertain<bool>`, where a mean is meaningless. `FromPrimitive` supplies the sample-count
// divisor at the value type's precision (no narrowing through `f64`).
impl<T: ProbabilisticType + RealField + FromPrimitive> Uncertain<T> {
    /// Estimates the expected value (mean) by averaging `num_samples` draws.
    pub fn expected_value(&self, num_samples: usize) -> Result<T, UncertainError> {
        if num_samples == 0 {
            return Ok(T::zero());
        }
        // `MeanAccumulator` rather than collecting into a slice and calling `mean`: a Monte-Carlo
        // estimator's whole point is that it can take a great many draws, and holding them all to
        // average them would turn constant memory into `num_samples`. The accumulator folds left to
        // right exactly as `mean` does, so the answer is the same to the last bit.
        let mut acc = MeanAccumulator::new();
        for i in 0..num_samples {
            acc.push(self.sample_with_index(i as u64)?);
        }
        acc.mean().map_err(sampling_error)
    }

    /// Estimates the (sample) standard deviation from `num_samples` draws, using the
    /// `(n − 1)` Bessel-corrected denominator.
    pub fn standard_deviation(&self, num_samples: usize) -> Result<T, UncertainError> {
        if num_samples <= 1 {
            return Ok(T::zero());
        }

        let samples: Vec<T> = (0..num_samples)
            .map(|i| self.sample_with_index(i as u64))
            .collect::<Result<Vec<T>, UncertainError>>()?;

        std_dev(&samples).map_err(sampling_error)
    }

    /// Quasi-Monte-Carlo expected value: averages `num_samples` Sobol draws (digitally shifted
    /// by `seed`). Converges faster than [`Self::expected_value`] on low-dimension static trees;
    /// the same `seed` reproduces the estimate. Returns `UncertainError::SamplingError` if the
    /// tree is not statically structured (see [`QmcSampler`](crate::QmcSampler)).
    pub fn expected_value_qmc(&self, num_samples: usize, seed: u64) -> Result<T, UncertainError> {
        if num_samples == 0 {
            return Ok(T::zero());
        }
        let sampler = QmcSampler::new(self, Some(seed))?;
        let mut acc = MeanAccumulator::new();
        for i in 0..num_samples {
            acc.push(self.sample_with_index_qmc(i as u64, &sampler)?);
        }
        acc.mean().map_err(sampling_error)
    }

    /// Quasi-Monte-Carlo (sample) standard deviation over `num_samples` Sobol draws (digitally
    /// shifted by `seed`). The digital shift makes this a genuine sampling-error estimate, not a
    /// degenerate zero. Uses the `(n − 1)` Bessel-corrected denominator.
    pub fn standard_deviation_qmc(
        &self,
        num_samples: usize,
        seed: u64,
    ) -> Result<T, UncertainError> {
        if num_samples <= 1 {
            return Ok(T::zero());
        }
        let sampler = QmcSampler::new(self, Some(seed))?;
        let samples: Vec<T> = (0..num_samples)
            .map(|i| self.sample_with_index_qmc(i as u64, &sampler))
            .collect::<Result<Vec<T>, UncertainError>>()?;

        std_dev(&samples).map_err(sampling_error)
    }
}

/// Maps a refusal from the statistics crate onto this crate's sampling error.
///
/// Both callers short-circuit `num_samples <= 1` before drawing anything, so the refusals that can
/// reach here are the ones a sampler cannot produce — an empty draw list, or a count the value type
/// cannot hold. The message is carried through rather than replaced, since it says which.
fn sampling_error(error: StatsError) -> UncertainError {
    UncertainError::SamplingError(error.to_string())
}
