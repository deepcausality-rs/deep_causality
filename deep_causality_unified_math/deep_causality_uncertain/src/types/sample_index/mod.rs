/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The address of one draw, minus the part the graph supplies.

use crate::SampleSession;

/// Where a draw comes from: a session's seed and a sample index.
///
/// A leaf's entropy is a function of three numbers — the seed, the sample index and the leaf's
/// [ordinal](crate::LeafOrdinals). The ordinal belongs to the graph, so it is the graph that
/// supplies it; the other two belong to the caller, and this is them.
///
/// # Why it exists as a type
///
/// [`Arrow::run`](deep_causality_haft::Arrow::run) takes `&self` and one input, so an arrow has
/// nowhere to keep a mutable session and no way to consult an ambient one. Everything the draw
/// depends on has to arrive in the input. Passing a bare `u64` index would not do it — the same
/// index under two seeds is two different draws — so the input is the pair, and running a graph at
/// a `SampleIndex` is then a pure function of the graph and that pair.
///
/// That this type is small and `Copy` is the point rather than a convenience: an address is data,
/// and a draw is what you get by evaluating a graph at one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SampleIndex {
    seed: u64,
    index: u64,
}

impl SampleIndex {
    /// The address of sample `index` within `session`.
    ///
    /// Takes `&SampleSession` rather than `&mut`, because naming an index needs nothing from the
    /// session's counter. [`SampleIndex::sequence`] is the form that walks a range.
    pub fn at(session: &SampleSession, index: u64) -> Self {
        Self {
            seed: session.seed(),
            index,
        }
    }

    /// The addresses of samples `0..n` within `session`, in order.
    ///
    /// The shape an arrow is usually run over: one graph, many indices, nothing stored between.
    pub fn sequence(session: &SampleSession, n: u64) -> impl Iterator<Item = Self> + use<> {
        let seed = session.seed();
        (0..n).map(move |index| Self { seed, index })
    }

    /// The seed this address draws under.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// The sample index within that seed's sequence.
    pub fn index(&self) -> u64 {
        self.index
    }
}
