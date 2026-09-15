/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    LeafOrdinals, QmcSampler, SampleSession, Sampler, SequentialSampler, Uncertain, UncertainError,
};
use deep_causality_rand::RandScalar;

// The sampling surface. Every draw is a function of the session's seed, the sample index and the
// leaf's ordinal, so nothing is stored between calls and nothing ambient is read.
//
// # Why the session is a parameter rather than a thread-local
//
// It was a thread-local slot, which made a draw depend on which thread reached it and on whatever
// had seeded that thread earlier. A session in the signature is visible at the call site, cannot be
// displaced by an unrelated caller, and travels across threads with the work rather than being left
// behind by it.
//
// Where no session is available, the `_from_entropy` forms build one and say so in their name. They
// are one line each, and the naming is the point: a reader can see which draws in a program are
// reproducible and which are not.
//
// The body is shared with `UncertainBool<R>` down to the last line except for which kind of sample
// it reads off the root. See `UncertainBool`'s sampling surface for the mirror.
impl<R: RandScalar> Uncertain<R> {
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
    /// alone. A caller drawing many samples from one graph should assign the ordinals once with
    /// [`LeafOrdinals::new`] and call [`Uncertain::sample_at_with`] instead.
    pub fn sample_at(&self, session: &SampleSession, index: u64) -> Result<R, UncertainError> {
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
    ) -> Result<R, UncertainError> {
        let sampler = SequentialSampler;
        sampler
            .sample_addressed(self.root_node(), ordinals, session.seed(), index)?
            .real()
    }

    /// Draws at the session's next index, advancing it.
    ///
    /// `n` calls on a fresh session cover indices `0..n`, and a session rebuilt from the same seed
    /// replays them exactly.
    pub fn sample_next(&self, session: &mut SampleSession) -> Result<R, UncertainError> {
        let index = session.next_index();
        self.sample_at(session, index)
    }

    /// Draws one value with no session of the caller's own.
    ///
    /// Builds a session from host entropy and draws its first sample, so successive calls are
    /// independent. Nothing about the value can be reproduced afterwards; where that matters, hold
    /// a [`SampleSession`] and use [`Uncertain::sample_next`].
    pub fn sample_from_entropy(&self) -> Result<R, UncertainError> {
        self.sample_at(&SampleSession::from_entropy(), 0)
    }

    /// Draws `n` samples, advancing the session by `n` indices.
    pub fn take_samples(
        &self,
        session: &mut SampleSession,
        n: usize,
    ) -> Result<Vec<R>, UncertainError> {
        let ordinals = LeafOrdinals::from_root_node(self.root_node());
        (0..n)
            .map(|_| {
                let index = session.next_index();
                self.sample_at_with(session, index, &ordinals)
            })
            .collect()
    }

    /// Draws `n` samples with no session of the caller's own. See
    /// [`Uncertain::sample_from_entropy`].
    pub fn take_samples_from_entropy(&self, n: usize) -> Result<Vec<R>, UncertainError> {
        self.take_samples(&mut SampleSession::from_entropy(), n)
    }

    /// Draws `n` samples at indices `0..n` of `session`, without advancing it.
    ///
    /// What the statistical reducers use. Holding the indices fixed makes an estimate a function of
    /// the session's seed and the sample count alone, so the same session gives the same estimate
    /// however many times it is asked — which is what lets a test assert on one.
    pub(crate) fn samples_from(
        &self,
        session: &SampleSession,
        n: usize,
    ) -> Result<Vec<R>, UncertainError> {
        let ordinals = LeafOrdinals::from_root_node(self.root_node());
        (0..n)
            .map(|i| self.sample_at_with(session, i as u64, &ordinals))
            .collect()
    }

    /// Draws a Quasi-Monte-Carlo sample at `sample_index` using a pre-built [`QmcSampler`].
    ///
    /// Takes no session: a Sobol point is a function of its index and the sampler's digital shift,
    /// and the shift comes from the seed given to [`QmcSampler::new`]. This path was reproducible
    /// before the session existed and is unchanged by it.
    pub fn sample_with_index_qmc(
        &self,
        sample_index: u64,
        sampler: &QmcSampler,
    ) -> Result<R, UncertainError> {
        Sampler::<R>::sample(sampler, self.root_node(), sample_index)?.real()
    }
}
