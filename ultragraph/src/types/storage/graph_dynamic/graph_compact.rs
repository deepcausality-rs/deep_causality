/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{DynamicGraph, EdgeKey};
use alloc::vec::Vec;

impl<N, W> DynamicGraph<N, W>
where
    W: Clone + Default + EdgeKey,
{
    /// Removes every tombstone, renumbers the live nodes in their order, remaps every edge and
    /// the root, and returns for each old position its new position, or `None` for a tombstone.
    ///
    /// The only operation besides `clear` that changes a live node's position. A caller that
    /// kept positions remaps them through the returned table.
    pub fn compact_ids(&mut self) -> Vec<Option<usize>> {
        let mut table = Vec::with_capacity(self.nodes.len());
        let mut next = 0;
        for node in &self.nodes {
            table.push(node.as_ref().map(|_| {
                next += 1;
                next - 1
            }));
        }

        let nodes = core::mem::take(&mut self.nodes);
        let edges = core::mem::take(&mut self.edges);
        let mut compacted_nodes = Vec::with_capacity(next);
        let mut compacted_edges = Vec::with_capacity(next);
        for (node, edge_list) in nodes.into_iter().zip(edges) {
            if node.is_none() {
                continue;
            }
            compacted_nodes.push(node);
            // Every stored edge joins two live nodes, so every target has a new position.
            compacted_edges.push(
                edge_list
                    .into_iter()
                    .filter_map(|(target, weight)| table[target].map(|new| (new, weight)))
                    .collect(),
            );
        }
        self.nodes = compacted_nodes;
        self.edges = compacted_edges;
        self.rebuild_backward();
        // A root past the table, as `from_parts` may have been given, has no row and is cleared.
        self.root_index = self
            .root_index
            .and_then(|root| table.get(root).copied().flatten());
        table
    }
}
