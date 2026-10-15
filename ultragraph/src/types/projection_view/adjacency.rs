/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{EdgeKey, GraphView, ParallelEdges, Projection, ProjectionView};

impl<N, W, P> Adjacency<W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn slots(&self) -> usize {
        self.graph.node_slots()
    }

    /// Live and passing the node test.
    fn passes(&self, v: usize) -> bool {
        self.graph
            .get_node(v)
            .is_some_and(|node| self.projection.node(node))
    }

    fn mode(&self) -> ParallelEdges {
        self.mode
    }

    /// An edge passes when its payload passes the edge test and both ends pass.
    fn out_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        let source_passes = self.passes(v);
        self.graph.forward_of(v).filter(move |(target, weight)| {
            source_passes && self.passes(*target) && self.projection.edge(weight)
        })
    }

    fn in_edges<'a>(&'a self, v: usize) -> impl Iterator<Item = (usize, &'a W)>
    where
        W: 'a,
    {
        let target_passes = self.passes(v);
        self.graph.backward_of(v).filter(move |(source, weight)| {
            target_passes && self.passes(*source) && self.projection.edge(weight)
        })
    }
}
