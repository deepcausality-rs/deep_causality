/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::UncertainScalar;
use crate::{
    LeafOrdinals, QmcSampler, SampleSession, Sampler, SequentialSampler, UncertainBool,
    UncertainError,
};

// The Boolean carrier's sampling surface. Identical to the real carrier's in every respect except
// which kind of sample it reads off the root — the address of a draw is the session seed, the
// sample index and the leaf ordinal, and none of those three knows what the root produces.
//
// The two are written out rather than shared through a trait: a carrier's sampling methods are its
// public surface, and a reader of `UncertainBool` should find them on `UncertainBool`. What is
// shared is what does the work — `SequentialSampler`, `QmcSampler` and `LeafOrdinals` are each one
// body over the graph.
impl<R: UncertainScalar> UncertainBool<R> {
    /// Draws this value at `index` under `session`.
    ///
    /// Reproducible from the session's seed alone: the same seed and index give the same value in
    /// a later process, and two graphs sharing a leaf agree about that leaf at the same index.
    pub fn sample_at(&self, session: &SampleSession, index: u64) -> Result<bool, UncertainError> {
        let ordinals = LeafOrdinals::from_root_node(self.root_node());
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
    ) -> Result<bool, UncertainError> {
        let sampler = SequentialSampler;
        sampler
            .sample_addressed(self.root_node(), ordinals, session.seed(), index)?
            .boolean()
    }

    /// Draws at the session's next index, advancing it.
    pub fn sample_next(&self, session: &mut SampleSession) -> Result<bool, UncertainError> {
        let index = session.next_index();
        self.sample_at(session, index)
    }

    /// Draws one value with no session of the caller's own.
    ///
    /// Nothing about the value can be reproduced afterwards; where that matters, hold a
    /// [`SampleSession`] and use [`UncertainBool::sample_next`].
    pub fn sample_from_entropy(&self) -> Result<bool, UncertainError> {
        self.sample_at(&SampleSession::from_entropy(), 0)
    }

    /// Draws `n` samples, advancing the session by `n` indices.
    pub fn take_samples(
        &self,
        session: &mut SampleSession,
        n: usize,
    ) -> Result<Vec<bool>, UncertainError> {
        let ordinals = LeafOrdinals::from_root_node(self.root_node());
        (0..n)
            .map(|_| {
                let index = session.next_index();
                self.sample_at_with(session, index, &ordinals)
            })
            .collect()
    }

    /// Draws `n` samples with no session of the caller's own.
    pub fn take_samples_from_entropy(&self, n: usize) -> Result<Vec<bool>, UncertainError> {
        self.take_samples(&mut SampleSession::from_entropy(), n)
    }

    /// Draws `n` samples at indices `0..n` of `session`, without advancing it.
    ///
    /// What the probability estimators use. Holding the indices fixed makes an estimate a function
    /// of the session's seed and the sample count alone.
    pub(crate) fn samples_from(
        &self,
        session: &SampleSession,
        n: usize,
    ) -> Result<Vec<bool>, UncertainError> {
        let ordinals = LeafOrdinals::from_root_node(self.root_node());
        (0..n)
            .map(|i| self.sample_at_with(session, i as u64, &ordinals))
            .collect()
    }

    /// Draws a Quasi-Monte-Carlo sample at `sample_index` using a pre-built [`QmcSampler`].
    pub fn sample_with_index_qmc(
        &self,
        sample_index: u64,
        sampler: &QmcSampler,
    ) -> Result<bool, UncertainError> {
        Sampler::<R>::sample(sampler, self.root_node(), sample_index)?.boolean()
    }
}
