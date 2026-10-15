/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod adjacency;
mod graph_algo;
mod graph_traversal;
mod graph_view;

use crate::{CsmGraph, EdgeKey, ParallelEdges, Projection};

/// A view of a frozen graph restricted to the nodes and edges a [`Projection`] passes, with the
/// parallel-edge mode its algorithms count under. Positions are those of the graph; a failing
/// node reads as absent. The view borrows the graph, so the graph cannot change while it lives.
#[derive(Clone, Debug)]
pub struct ProjectionView<'a, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    graph: &'a CsmGraph<N, W>,
    projection: P,
    mode: ParallelEdges,
}

impl<'a, N, W, P> ProjectionView<'a, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    pub(crate) fn new(graph: &'a CsmGraph<N, W>, projection: P, mode: ParallelEdges) -> Self {
        Self {
            graph,
            projection,
            mode,
        }
    }

    /// The parallel-edge mode the view's algorithms count under.
    pub fn mode(&self) -> ParallelEdges {
        self.mode
    }
}
