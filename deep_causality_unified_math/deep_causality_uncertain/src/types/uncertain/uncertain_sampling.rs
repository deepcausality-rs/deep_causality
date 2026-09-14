/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::sampler::sampler_seed::next_sample_index;
use crate::{
    LeafOrdinals, ProbabilisticType, QmcSampler, SampleSession, Sampler, SamplerKind,
    SequentialSampler, Uncertain, UncertainError, with_global_cache,
};

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
        let key = (self.id, sample_index, SamplerKind::Mc);

        let computed_value = with_global_cache(|cache| {
            cache.get_or_compute(key, || {
                let sampler = SequentialSampler;
                Sampler::<T>::sample(&sampler, &self.root_node, sample_index)
            })
        })?;

        T::from_sampled_value(computed_value)
    }

    /// Draw a Quasi-Monte-Carlo sample at `sample_index` using a pre-built [`QmcSampler`].
    ///
    /// The Sobol point at `sample_index` makes the draw deterministic; the global cache stores
    /// it under a QMC-discriminated key so it never collides with a Monte-Carlo draw at the same
    /// index. The `sampler` must have been built from this `Uncertain`'s root node.
    pub fn sample_with_index_qmc(
        &self,
        sample_index: u64,
        sampler: &QmcSampler,
    ) -> Result<T, UncertainError> {
        let key = (self.id, sample_index, SamplerKind::Qmc);

        let computed_value = with_global_cache(|cache| {
            cache.get_or_compute(key, || {
                Sampler::<T>::sample(sampler, &self.root_node, sample_index)
            })
        })?;

        T::from_sampled_value(computed_value)
    }

    /// Draw a single sample at a random index (a reproducible index when `seed_sampler` is in
    /// effect on this thread).
    pub fn sample(&self) -> Result<T, UncertainError> {
        let sample_index = next_sample_index();
        self.sample_with_index(sample_index)
    }

    /// Draw `n` independent samples (reproducible when `seed_sampler` is in effect on this thread).
    pub fn take_samples(&self, n: usize) -> Result<Vec<T>, UncertainError> {
        (0..n)
            .map(|_| {
                let sample_index = next_sample_index();
                self.sample_with_index(sample_index)
            })
            .collect()
    }
}

// The addressed sampling surface. A draw is a function of the session's seed, the sample index and
// the leaf's ordinal, so nothing is stored between calls and nothing ambient is read.
impl<T: ProbabilisticType> Uncertain<T> {
    /// Draws this value at `index` under `session`.
    ///
    /// Reproducible from the session's seed alone: the same seed and index give the same value in
    /// a later process, and two graphs sharing a leaf agree about that leaf at the same index.
    ///
    /// Takes `&SampleSession` rather than `&mut`, because an explicit index needs nothing from the
    /// session's counter. [`Uncertain::sample_next`] is the one that advances it.
    ///
    /// # Cost
    ///
    /// One traversal to assign the ordinals, then one to evaluate — the same order as evaluating
    /// alone. A caller drawing many samples from one graph should build the ordinals once with
    /// [`LeafOrdinals::new`] and call [`Uncertain::sample_at_with`] instead.
    pub fn sample_at(&self, session: &SampleSession, index: u64) -> Result<T, UncertainError> {
        let ordinals = LeafOrdinals::from_root_node(&self.root_node);
        self.sample_at_with(session, index, &ordinals)
    }

    /// Draws at `index` using ordinals already assigned for this graph.
    ///
    /// The form to use in a loop: the ordinals do not change between samples, so assigning them
    /// once and reusing them makes `n` draws cost one traversal each rather than two.
    pub fn sample_at_with(
        &self,
        session: &SampleSession,
        index: u64,
        ordinals: &LeafOrdinals,
    ) -> Result<T, UncertainError> {
        let sampler = SequentialSampler;
        let value = sampler.sample_addressed(&self.root_node, ordinals, session.seed(), index)?;
        T::from_sampled_value(value)
    }

    /// Draws at the session's next index, advancing it.
    ///
    /// `n` calls on a fresh session cover indices `0..n`, and a session rebuilt from the same seed
    /// replays them exactly.
    pub fn sample_next(&self, session: &mut SampleSession) -> Result<T, UncertainError> {
        let index = session.next_index();
        self.sample_at(session, index)
    }
}
