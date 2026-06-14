/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ProbabilisticType, Sampler, SequentialSampler, Uncertain, UncertainError, with_global_cache,
};
use deep_causality_rand::Rng;

// Precision-generic sampling surface for `Uncertain<T>`. The draw runs the shared
// `SequentialSampler` (which already impls `Sampler<T>` for every `ProbabilisticType`)
// and converts the cached `SampledValue` back to `T` through `T::from_sampled_value` — the
// supertrait of `ProbabilisticType` built for exactly this. Every per-type extraction
// (the `f64`/`bool` matches and the `Float106` f64→double-double widening) lives in that
// conversion, so this one impl reproduces all of them with no narrowing.
impl<T: ProbabilisticType> Uncertain<T> {
    /// Draw a sample for a specific sample index; the global cache makes the draw at a
    /// given `(id, index)` reproducible.
    pub fn sample_with_index(&self, sample_index: u64) -> Result<T, UncertainError> {
        let key = (self.id, sample_index);

        let computed_value = with_global_cache(|cache| {
            cache.get_or_compute(key, || {
                let sampler = SequentialSampler;
                Sampler::<T>::sample(&sampler, &self.root_node)
            })
        })?;

        T::from_sampled_value(computed_value)
    }

    /// Draw a single sample at a random index.
    pub fn sample(&self) -> Result<T, UncertainError> {
        let sample_index = deep_causality_rand::rng().random::<u64>();
        self.sample_with_index(sample_index)
    }

    /// Draw `n` independent samples.
    pub fn take_samples(&self, n: usize) -> Result<Vec<T>, UncertainError> {
        (0..n)
            .map(|_| {
                let sample_index = deep_causality_rand::rng().random::<u64>();
                self.sample_with_index(sample_index)
            })
            .collect()
    }
}
