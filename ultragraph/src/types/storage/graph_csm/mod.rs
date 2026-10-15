/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::EdgeKey;

use alloc::vec;
use alloc::vec::Vec;

pub(crate) mod adjacency;
mod default;
mod graph_csm_algo;
pub(crate) mod graph_csm_algo_articulation_points;
pub(crate) mod graph_csm_algo_biconnected_components;
pub(crate) mod graph_csm_algo_bridges;
pub(crate) mod graph_csm_algo_centrality;
pub(crate) mod graph_csm_algo_pathfinder;
pub(crate) mod graph_csm_algo_structural;
pub(crate) mod graph_csm_algo_topological;
mod graph_csm_biconnectivity_common;
mod graph_csm_unfreeze;
mod graph_csm_view;
mod graph_traversal;

// The "Struct of Arrays" (SoA) representation for adjacencies.
// This is now a first-class, albeit private, component of the CsmGraph.
// It derives Default for convenience in constructors.
#[derive(Default, Debug, Clone)]
pub(crate) struct CsrAdjacency<W>
where
    W: Clone + Default + EdgeKey,
{
    pub(crate) offsets: Vec<usize>,
    pub(crate) targets: Vec<usize>,
    pub(crate) weights: Vec<W>,
}

#[derive(Clone, Debug)]
pub struct CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    // Node payloads by position. A removed node leaves `None`; positions never move.
    nodes: Vec<Option<N>>,

    // The number of `Some` entries in `nodes`, so `number_nodes` stays O(1).
    live_nodes: usize,

    // CsrAdjacency structure for forward traversal (successors).
    // The CsrAdjacency makes the intent of the data layout explicit and clean.
    forward_edges: CsrAdjacency<W>,

    // CSR structure for backward traversal (predecessors).
    backward_edges: CsrAdjacency<W>,

    // Index of the designated root node.
    root_index: Option<usize>,
}

impl<N, W> CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Creates a new, empty `CsmGraph`.
    ///
    /// The graph will have zero nodes and zero edges.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            live_nodes: 0,

            // Initialize with a valid empty CSR state. The offsets vector must
            // contain a single `0` to correctly represent the `V + 1` length rule, where V=0.
            forward_edges: CsrAdjacency {
                offsets: vec![0],
                ..Default::default()
            },
            backward_edges: CsrAdjacency {
                offsets: vec![0],
                ..Default::default()
            },
            root_index: None,
        }
    }

    /// Creates a new, empty `CsmGraph` whose node table has room for `num_nodes` positions.
    ///
    /// The graph has no node and no edge; `freeze` is what fills a `CsmGraph`. Only the
    /// capacity of the node table differs from [`new`](Self::new).
    ///
    /// # Arguments
    /// * `num_nodes`: The number of node positions to reserve.
    pub fn with_capacity(num_nodes: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(num_nodes),
            live_nodes: 0,
            // Initialize with a valid empty CSR state. The offsets vector must
            // contain a single `0` to correctly represent the `V + 1` length rule, where V=0.
            forward_edges: CsrAdjacency {
                offsets: vec![0],
                ..Default::default()
            },
            backward_edges: CsrAdjacency {
                offsets: vec![0],
                ..Default::default()
            },
            root_index: None,
        }
    }

    // Internal helper for freeze. `live_nodes` is the number of `Some` entries in `nodes`.
    pub(crate) fn construct(
        nodes: Vec<Option<N>>,
        forward_edges: CsrAdjacency<W>,
        backward_edges: CsrAdjacency<W>,
        root_index: Option<usize>,
        live_nodes: usize,
    ) -> Self {
        Self {
            nodes,
            live_nodes,
            forward_edges,
            backward_edges,
            root_index,
        }
    }

    /// The number of positions in the node table, tombstones included. Every per-node array an
    /// algorithm indexes by position is sized by this, never by `number_nodes`.
    pub(crate) fn node_slots(&self) -> usize {
        self.nodes.len()
    }

    /// The edges out of `v` as `(target, weight)`, sorted by target then key; nothing when `v`
    /// holds no node.
    pub(crate) fn forward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        let (start, end) = self.range_of(&self.forward_edges, v);
        self.forward_edges.targets[start..end]
            .iter()
            .copied()
            .zip(self.forward_edges.weights[start..end].iter())
    }

    /// The edges into `v` as `(source, weight)`, sorted by source then key; nothing when `v`
    /// holds no node.
    pub(crate) fn backward_of(&self, v: usize) -> impl Iterator<Item = (usize, &W)> {
        let (start, end) = self.range_of(&self.backward_edges, v);
        self.backward_edges.targets[start..end]
            .iter()
            .copied()
            .zip(self.backward_edges.weights[start..end].iter())
    }

    fn range_of(&self, adjacency: &CsrAdjacency<W>, v: usize) -> (usize, usize) {
        if self.nodes.get(v).is_some_and(Option::is_some) {
            (adjacency.offsets[v], adjacency.offsets[v + 1])
        } else {
            (0, 0)
        }
    }
}
