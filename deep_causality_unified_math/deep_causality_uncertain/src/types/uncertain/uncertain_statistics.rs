/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::UncertainScalar;
use crate::{QmcSampler, SampleSession, Uncertain, UncertainError};
use deep_causality_stats::{MeanAccumulator, StatsError, std_dev};

// Monte-Carlo statistics at the caller's scalar. These reduce many samples into one, so they need
// arithmetic and a square root — which `UncertainScalar` supplies — and they are on the real carrier
// only, because a mean of truth values is not a truth value. The Boolean carrier reduces to a
// probability instead, which is a different operation and lives on that type.
impl<R: UncertainScalar> Uncertain<R> {
    /// Estimates the expected value (mean) by averaging `num_samples` draws under `session`.
    ///
    /// The draws are taken at indices `0..num_samples`, so the estimate is a function of the
    /// session's seed and the count alone: asking twice gives the same answer, and a session
    /// rebuilt from the same seed reproduces it in a later process.
    pub fn expected_value(
        &self,
        session: &SampleSession,
        num_samples: usize,
    ) -> Result<R, UncertainError> {
        if num_samples == 0 {
            return Ok(R::zero());
        }
        // `MeanAccumulator` rather than collecting into a slice and calling `mean`: a Monte-Carlo
        // estimator's whole point is that it can take a great many draws, and holding them all to
        // average them would turn constant memory into `num_samples`. It sums as a balanced tree,
        // so the estimate does not stagnate once the running total outgrows an addend — which at a
        // narrow scalar is the difference between an answer and a plausible wrong one.
        let ordinals = crate::LeafOrdinals::from_root_node(self.root_node());
        let mut acc = MeanAccumulator::new();
        for i in 0..num_samples {
            acc.push(self.sample_at_with(session, i as u64, &ordinals)?);
        }
        acc.mean().map_err(sampling_error)
    }

    /// Estimates the expected value with no session of the caller's own.
    ///
    /// See [`Uncertain::sample_from_entropy`]: the estimate cannot be reproduced afterwards.
    pub fn expected_value_from_entropy(&self, num_samples: usize) -> Result<R, UncertainError> {
        self.expected_value(&SampleSession::from_entropy(), num_samples)
    }

    /// Estimates the (sample) standard deviation from `num_samples` draws, using the
    /// `(n − 1)` Bessel-corrected denominator.
    pub fn standard_deviation(
        &self,
        session: &SampleSession,
        num_samples: usize,
    ) -> Result<R, UncertainError> {
        if num_samples <= 1 {
            return Ok(R::zero());
        }

        let samples = self.samples_from(session, num_samples)?;

        std_dev(&samples).map_err(sampling_error)
    }

    /// Estimates the standard deviation with no session of the caller's own.
    pub fn standard_deviation_from_entropy(&self, num_samples: usize) -> Result<R, UncertainError> {
        self.standard_deviation(&SampleSession::from_entropy(), num_samples)
    }

    /// Quasi-Monte-Carlo expected value: averages `num_samples` Sobol draws (digitally shifted
    /// by `seed`). Converges faster than [`Self::expected_value`] on low-dimension static trees;
    /// the same `seed` reproduces the estimate. Returns `UncertainError::SamplingError` if the
    /// tree is not statically structured (see [`QmcSampler`](crate::QmcSampler)).
    pub fn expected_value_qmc(&self, num_samples: usize, seed: u64) -> Result<R, UncertainError> {
        if num_samples == 0 {
            return Ok(R::zero());
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
    ) -> Result<R, UncertainError> {
        if num_samples <= 1 {
            return Ok(R::zero());
        }
        let sampler = QmcSampler::new(self, Some(seed))?;
        let samples: Vec<R> = (0..num_samples)
            .map(|i| self.sample_with_index_qmc(i as u64, &sampler))
            .collect::<Result<Vec<R>, UncertainError>>()?;

        std_dev(&samples).map_err(sampling_error)
    }
}

/// Maps a refusal from the statistics crate onto this crate's sampling error.
///
/// Both callers short-circuit `num_samples <= 1` before drawing anything, so the refusals that can
/// reach here are the ones a sampler cannot produce — an empty draw list, or a count the value type
/// cannot hold. The message is carried through rather than replaced, since it says which.
pub(crate) fn sampling_error(error: StatsError) -> UncertainError {
    UncertainError::SamplingError(error.to_string())
}
