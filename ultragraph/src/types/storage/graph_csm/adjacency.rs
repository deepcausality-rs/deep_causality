/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The one adjacency every algorithm reads. The frozen graph implements it as the simple
//! projection, every node and edge under `Merge`; a projection view implements it with its
//! tests applied and its own mode. Algorithm bodies are generic over it, so one body serves both.

use crate::{CsmGraph, EdgeKey, GraphView, ParallelEdges};

pub(crate) trait Adjacency<W> {
    /// The number of positions in the node table, tombstones and failing nodes included.
    fn slots(&self) -> usize;

    /// Whether the node at `v` is live and in the projection.
    fn passes(&self, v: usize) -> bool;

    /// How the algorithms count parallel edges.
    fn mode(&self) -> ParallelEdges;

    /// Every passing edge out of `v`, sorted by target then key; nothing when `v` does not pass.
    fn out_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a;

    /// Every passing edge into `v`, sorted by source then key; nothing when `v` does not pass.
    fn in_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a;

    /// The number of passing nodes.
    fn passing_count(&self) -> usize {
        (0..self.slots()).filter(|&v| self.passes(v)).count()
    }
}

impl<N, W> Adjacency<W> for CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    fn slots(&self) -> usize {
        self.node_slots()
    }

    fn passes(&self, v: usize) -> bool {
        self.contains_node(v)
    }

    fn mode(&self) -> ParallelEdges {
        ParallelEdges::Merge
    }

    fn out_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        self.forward_of(v)
    }

    fn in_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        self.backward_of(v)
    }

    fn passing_count(&self) -> usize {
        self.number_nodes()
    }
}
