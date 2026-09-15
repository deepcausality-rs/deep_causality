/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Where a leaf's entropy comes from.

use crate::UncertainScalar;
use crate::{DistributionEnum, LeafOrdinals, Sample, UncertainError, draw_seed};
use deep_causality_rand::{Rng, Xoshiro256};

/// The source of a leaf's draw, so one traversal body serves both the ambient and the addressed
/// path.
///
/// The two differ only in where the entropy for a leaf comes from; everything else about
/// evaluating a graph — the memo, the operators, the branch handling — is identical, and
/// duplicating two hundred lines of `match` to vary one line of it would be the larger mistake.
/// Static dispatch, so neither path pays for the other and AGENTS.md's ban on `dyn` is respected.
///
/// One method, not three. There were `draw_f64`, `draw_f106` and `draw_bool`, which said the
/// source had to know the leaf's type; it never did — it only had to know where the generator
/// comes from. The scalar is now the implementation's parameter and the Boolean case is a variant
/// of what a draw returns, so the trait has one method at any scalar.
pub(crate) trait LeafDraws<R: UncertainScalar> {
    /// Draws for the leaf identified by `node_id`.
    fn draw(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<R>,
    ) -> Result<Sample<R>, UncertainError>;
}

/// The superseded source: one stateful generator for the whole graph.
///
/// Every leaf draws from the same stream in traversal order, so a leaf's value depends on how many
/// leaves were visited before it. That is what makes the stream, rather than the leaf, the unit of
/// reproducibility — and why two graphs sharing a leaf disagree about it.
pub(crate) struct AmbientDraws<'r, G: Rng + ?Sized> {
    pub(crate) rng: &'r mut G,
}

impl<R: UncertainScalar, G: Rng + ?Sized> LeafDraws<R> for AmbientDraws<'_, G> {
    fn draw(
        &mut self,
        _node_id: usize,
        distribution: &DistributionEnum<R>,
    ) -> Result<Sample<R>, UncertainError> {
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

impl<R: UncertainScalar> LeafDraws<R> for AddressedDraws<'_> {
    fn draw(
        &mut self,
        node_id: usize,
        distribution: &DistributionEnum<R>,
    ) -> Result<Sample<R>, UncertainError> {
        distribution.sample(&mut self.generator(node_id)?)
    }
}
