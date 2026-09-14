/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The sampler's state, as a value the caller owns.

use crate::draw_seed;
use deep_causality_rand::Rng;

/// Which family of draws a session produces. Not public: a caller selects it by choosing a
/// constructor, and [`SampleSession::is_qmc`] reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Plain Monte Carlo.
    MonteCarlo,
    /// Quasi-Monte Carlo. The per-tree Sobol sequence is built by the sampler, from this
    /// session's seed.
    Qmc,
}

/// Everything a draw depends on, held in one value the caller constructs, passes and drops.
///
/// A session is the whole of the sampler's state. Nothing is read from a process-wide or
/// thread-local slot, so two sessions on one thread do not interfere, a test needs no process
/// isolation, and a run is reproducible from the seed alone.
///
/// # What it holds, and what it does not
///
/// A seed, a counter and a mode. It carries **no scalar parameter**: a draw's address is three
/// integers, and the scalar appears nowhere in it. One session therefore drives an
/// `Uncertain<f64>` and an `Uncertain<Float106>` together and correlates them at the same index,
/// which a scalar-parameterised session could not do.
///
/// It also holds **no Sobol sequence**. A Sobol sequence is built for a dimension count, and the
/// dimension count belongs to the tree rather than to the session; the sampler builds one per tree
/// from [`SampleSession::seed`].
///
/// # Monte Carlo or quasi-Monte Carlo by construction
///
/// The mode is fixed when the session is built, so a Monte-Carlo draw and a quasi-Monte-Carlo draw
/// at the same index cannot be confused for one another and nothing needs to carry a discriminant
/// to keep them apart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleSession {
    seed: u64,
    next_index: u64,
    mode: Mode,
}

impl SampleSession {
    /// A reproducible Monte-Carlo session.
    ///
    /// Every draw taken through it is a function of `seed`, the sample index and the leaf's
    /// ordinal, so the same seed replays the same values in a later process.
    pub const fn seeded(seed: u64) -> Self {
        Self {
            seed,
            next_index: 0,
            mode: Mode::MonteCarlo,
        }
    }

    /// A reproducible quasi-Monte-Carlo session.
    ///
    /// The seed drives the Sobol sequence's digital shift, which is what makes a randomized-QMC
    /// estimate repeatable and its variance estimable across independently seeded replicates.
    pub const fn qmc(seed: u64) -> Self {
        Self {
            seed,
            next_index: 0,
            mode: Mode::Qmc,
        }
    }

    /// A Monte-Carlo session seeded from the host's entropy, for a caller who wants a fresh stream
    /// per run and does not need to reproduce it.
    ///
    /// This is the one constructor that reads anything ambient, and what it reads belongs to
    /// `deep_causality_rand` rather than to this crate. It takes a single word and keeps nothing,
    /// so the session it returns is as self-contained as a seeded one — and printing
    /// [`SampleSession::seed`] is enough to reproduce the run afterwards.
    pub fn from_entropy() -> Self {
        Self::seeded(deep_causality_rand::rng().random_word::<u64>())
    }

    /// The seed every draw in this session is derived from.
    ///
    /// Worth recording next to a result: it is all that is needed to reproduce the run.
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Whether this session draws quasi-Monte-Carlo points.
    pub const fn is_qmc(&self) -> bool {
        matches!(self.mode, Mode::Qmc)
    }

    /// The index this session has reached.
    ///
    /// Equal to the number of indices [`SampleSession::next_index`] has handed out.
    pub const fn position(&self) -> u64 {
        self.next_index
    }

    /// Takes the next sample index, advancing the session.
    ///
    /// Indices run `0, 1, 2, …`, so `n` draws from a fresh session cover `0..n` and a session
    /// rebuilt from the same seed replays them. The predecessor drew a random index per sample,
    /// which made a seeded run reproducible only as long as nothing else consumed from the same
    /// generator in between.
    pub const fn next_index(&mut self) -> u64 {
        let index = self.next_index;
        self.next_index = self.next_index.wrapping_add(1);
        index
    }

    /// Rewinds to index zero, leaving the seed and the mode alone.
    ///
    /// Lets one session take a second pass that correlates with the first draw for draw, which is
    /// what pairing two quantities by index needs.
    pub const fn rewind(&mut self) {
        self.next_index = 0;
    }

    /// The generator seed for the leaf at `ordinal`, at sample `index`, in this session.
    ///
    /// The whole of a draw's address. Exposed so a caller can check reproducibility without
    /// running a sampler, and so the samplers share one definition rather than two.
    pub const fn draw_seed_at(&self, index: u64, ordinal: u64) -> u64 {
        draw_seed(self.seed, index, ordinal)
    }
}
