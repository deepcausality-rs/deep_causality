/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::EdgeKey;

use crate::DynamicGraph;
use crate::types::storage::graph_dynamic::DynamicGraphParts;

impl<N, W> DynamicGraph<N, W>
where
    W: Clone + Default + EdgeKey,
{
    /// Builds a graph from its node table, its forward adjacency lists and its root.
    ///
    /// The backward lists are derived from the forward lists. Every edge must join two live
    /// positions, and no two edges from one source to one target may share a key: that is the
    /// state `to_parts` returns and the state every mutation keeps.
    ///
    /// # Panics
    /// When the node table and the adjacency lists differ in length, and when an edge names a
    /// target past the node table: the backward rebuild indexes it.
    pub fn from_parts(parts: DynamicGraphParts<N, W>) -> Self {
        let (nodes, edges, root_index) = parts;

        // A non-negotiable sanity check. This prevents gross structural mismatches.
        assert_eq!(
            nodes.len(),
            edges.len(),
            "The number of node payloads must equal the number of adjacency lists."
        );

        Self::construct(nodes, edges, root_index)
    }

    /// The node table, the forward adjacency lists and the root; the backward lists are not
    /// part of it, `from_parts` derives them.
    pub fn to_parts(self) -> DynamicGraphParts<N, W> {
        (self.nodes, self.edges, self.root_index)
    }
}
