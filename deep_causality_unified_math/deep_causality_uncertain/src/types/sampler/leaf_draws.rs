/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where a leaf's entropy comes from.

use crate::{DistributionEnum, LeafOrdinals, UncertainError, draw_seed};
use deep_causality_num::Float106;
use deep_causality_rand::{Rng, Xoshiro256};

/// The source of a leaf's draw, so one traversal body serves both the ambient and the addressed
/// path.
///
/// The two differ only in where the entropy for a leaf comes from; everything else about
/// evaluating a graph — the memo, the operators, the branch handling — is identical, and
/// duplicating two hundred lines of `match` to vary one line of it would be the larger mistake.
/// Static dispatch, so neither path pays for the other and AGENTS.md's ban on `dyn` is respected.
pub(crate) trait LeafDraws {
    /// Draws for the real-valued leaf identified by `node_id`.
    fn draw_f64(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<f64>,
    ) -> Result<f64, UncertainError>;

    /// Draws for the double-double leaf identified by `node_id`.
    fn draw_f106(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<Float106>,
    ) -> Result<Float106, UncertainError>;

    /// Draws for the Boolean leaf identified by `node_id`.
    fn draw_bool(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<bool>,
    ) -> Result<bool, UncertainError>;
}

/// The superseded source: one stateful generator for the whole graph.
///
/// Every leaf draws from the same stream in traversal order, so a leaf's value depends on how many
/// leaves were visited before it. That is what makes the stream, rather than the leaf, the unit of
/// reproducibility — and why two graphs sharing a leaf disagree about it.
pub(crate) struct AmbientDraws<'r, R: Rng + ?Sized> {
    pub(crate) rng: &'r mut R,
}

impl<R: Rng + ?Sized> LeafDraws for AmbientDraws<'_, R> {
    fn draw_f64(
        &mut self,
        _node_id: usize,
        distribution: &DistributionEnum<f64>,
    ) -> Result<f64, UncertainError> {
        distribution.sample(self.rng)
    }

    fn draw_f106(
        &mut self,
        _node_id: usize,
        distribution: &DistributionEnum<Float106>,
    ) -> Result<Float106, UncertainError> {
        distribution.sample(self.rng)
    }

    fn draw_bool(
        &mut self,
        _node_id: usize,
        distribution: &DistributionEnum<bool>,
    ) -> Result<bool, UncertainError> {
        distribution.sample(self.rng)
    }
}

/// The addressed source: one generator per leaf, per sample.
///
/// A leaf's entropy is a function of the session seed, the sample index and the leaf's ordinal, so
/// it does not depend on what else the graph contains or on the order the traversal reached it.
/// Two graphs sharing a leaf therefore agree about that leaf at a given index, which the stream
/// could not provide at any price.
///
/// The cost is one generator construction per drawing leaf per sample — a few nanoseconds of
/// hashing — in exchange for dropping the cache that paid for the same property with unbounded
/// memory and did not deliver it across graphs.
pub(crate) struct AddressedDraws<'o> {
    pub(crate) seed: u64,
    pub(crate) index: u64,
    pub(crate) ordinals: &'o LeafOrdinals,
}

impl AddressedDraws<'_> {
    /// The generator for one leaf at this sample.
    ///
    /// A drawing leaf with no ordinal means the ordinals were built from a different graph, or the
    /// graph grew after they were built. Both are defects rather than conditions to paper over, so
    /// this reports rather than falling back to ambient entropy — a silent fallback would produce
    /// plausible values that no seed reproduces.
    fn generator(&self, node_id: usize) -> Result<Xoshiro256, UncertainError> {
        match self.ordinals.of_node(node_id) {
            Some(ordinal) => Ok(Xoshiro256::from_seed(draw_seed(
                self.seed, self.index, ordinal,
            ))),
            None => Err(UncertainError::SamplingError(
                "a drawing leaf carries no ordinal: the ordinals belong to a different graph, or \
                 the graph gained a leaf after they were assigned"
                    .into(),
            )),
        }
    }
}

impl LeafDraws for AddressedDraws<'_> {
    fn draw_f64(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<f64>,
    ) -> Result<f64, UncertainError> {
        distribution.sample(&mut self.generator(node_id)?)
    }

    fn draw_f106(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<Float106>,
    ) -> Result<Float106, UncertainError> {
        distribution.sample(&mut self.generator(node_id)?)
    }

    fn draw_bool(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<bool>,
    ) -> Result<bool, UncertainError> {
        distribution.sample(&mut self.generator(node_id)?)
    }
}
