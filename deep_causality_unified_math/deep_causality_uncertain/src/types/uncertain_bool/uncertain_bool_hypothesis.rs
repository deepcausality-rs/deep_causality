/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Turning a distribution of truth values into one decision, and into a probability.

use crate::UncertainScalar;
use crate::{QmcSampler, SampleSession, UncertainBool, UncertainError, ratio, sprt_eval};

impl<R: UncertainScalar> UncertainBool<R> {
    /// Collapses the distribution to a single truth value by sequential hypothesis testing.
    ///
    /// Uses the Sequential Probability Ratio Test to decide whether the underlying probability of
    /// `true` exceeds `threshold`.
    ///
    /// Every probability here is stated in the caller's scalar and the test's arithmetic runs
    /// there: a probability is a ratio of two counts, and the scalar that states the threshold is
    /// the one the ratio is formed in.
    ///
    /// # Arguments
    /// * `threshold` - The probability threshold to test against.
    /// * `confidence` - The desired confidence level (e.g. `0.95`).
    /// * `epsilon` - The indifference region around the threshold.
    /// * `max_samples` - The maximum number of samples to draw.
    ///
    /// # Returns
    /// `Ok(bool)` if a decision can be made, or `Err(UncertainError)` if the test cannot draw.
    pub fn to_bool(
        &self,
        session: &SampleSession,
        threshold: R,
        confidence: R,
        epsilon: R,
        max_samples: usize,
    ) -> Result<bool, UncertainError> {
        // Sample index 0: the decision is about the distribution, not about a chosen sample.
        sprt_eval::evaluate_hypothesis(
            self,
            session,
            threshold,
            confidence,
            epsilon,
            max_samples,
            0,
        )
    }

    /// As [`Self::to_bool`], with no session of the caller's own.
    pub fn to_bool_from_entropy(
        &self,
        threshold: R,
        confidence: R,
        epsilon: R,
        max_samples: usize,
    ) -> Result<bool, UncertainError> {
        self.to_bool(
            &SampleSession::from_entropy(),
            threshold,
            confidence,
            epsilon,
            max_samples,
        )
    }

    /// Whether the probability of `true` exceeds `threshold`. The same test as [`Self::to_bool`],
    /// under the name a caller asking about a probability reaches for.
    pub fn probability_exceeds(
        &self,
        session: &SampleSession,
        threshold: R,
        confidence: R,
        epsilon: R,
        max_samples: usize,
    ) -> Result<bool, UncertainError> {
        self.to_bool(session, threshold, confidence, epsilon, max_samples)
    }

    /// As [`Self::probability_exceeds`], with no session of the caller's own.
    pub fn probability_exceeds_from_entropy(
        &self,
        threshold: R,
        confidence: R,
        epsilon: R,
        max_samples: usize,
    ) -> Result<bool, UncertainError> {
        self.probability_exceeds(
            &SampleSession::from_entropy(),
            threshold,
            confidence,
            epsilon,
            max_samples,
        )
    }

    /// Evaluates an implicit conditional: whether `true` is more likely than not.
    ///
    /// Calls [`Self::probability_exceeds`] with `threshold = 0.5`, `confidence = 0.95`,
    /// `epsilon = 0.05` and a budget of 1000 samples. The three constants are lifted into `R`
    /// rather than written as literals, so the defaults exist at every scalar.
    pub fn implicit_conditional(&self, session: &SampleSession) -> Result<bool, UncertainError> {
        let half = ratio::<R>(1, 2)?;
        let confidence = ratio::<R>(95, 100)?;
        let epsilon = ratio::<R>(5, 100)?;
        self.probability_exceeds(session, half, confidence, epsilon, 1000)
    }

    /// As [`Self::implicit_conditional`], with no session of the caller's own.
    pub fn implicit_conditional_from_entropy(&self) -> Result<bool, UncertainError> {
        self.implicit_conditional(&SampleSession::from_entropy())
    }

    /// Estimates `P(true)` from `num_samples` draws, in the caller's scalar.
    pub fn estimate_probability(
        &self,
        session: &SampleSession,
        num_samples: usize,
    ) -> Result<R, UncertainError> {
        let samples = self.samples_from(session, num_samples)?;
        let hits = samples.iter().filter(|&&x| x).count();
        ratio(hits, samples.len())
    }

    /// As [`Self::estimate_probability`], with no session of the caller's own.
    pub fn estimate_probability_from_entropy(
        &self,
        num_samples: usize,
    ) -> Result<R, UncertainError> {
        self.estimate_probability(&SampleSession::from_entropy(), num_samples)
    }

    /// Quasi-Monte-Carlo estimate of `P(true)` over `num_samples` Sobol draws (digitally shifted
    /// by `seed`). Converges faster than [`Self::estimate_probability`] on low-dimension static
    /// trees and reproduces under the same `seed`. Returns `UncertainError::SamplingError` if the
    /// tree is not statically structured (see [`QmcSampler`](crate::QmcSampler)).
    pub fn estimate_probability_qmc(
        &self,
        num_samples: usize,
        seed: u64,
    ) -> Result<R, UncertainError> {
        if num_samples == 0 {
            return Ok(R::zero());
        }
        let sampler = QmcSampler::for_bool(self, Some(seed))?;
        let mut count = 0usize;
        for i in 0..num_samples {
            if self.sample_with_index_qmc(i as u64, &sampler)? {
                count += 1;
            }
        }
        ratio(count, num_samples)
    }
}
