/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::{DynamicGraph, EdgeKey, GraphView};

impl<N, W> GraphView<N, W> for DynamicGraph<N, W>
where
    W: Clone + Default + EdgeKey,
{
    /// Checks if the graph is in a frozen, high-performance state.
    /// For `DynamicGraph`, this is by definition always false.
    fn is_frozen(&self) -> bool {
        false
    }

    fn is_empty(&self) -> bool {
        self.nodes.iter().all(Option::is_none) && self.root_index.is_none()
    }

    /// Checks if a valid, non-tombstoned node exists at the given index.
    fn contains_node(&self, index: usize) -> bool {
        // `is_some_and` first checks if the index is in bounds (`get` returns `Some`),
        // and then checks if the inner Option is also `Some`.
        self.nodes
            .get(index)
            .is_some_and(|node_opt| node_opt.is_some())
    }

    /// Retrieves a reference to the payload of a node, if it exists and has not been removed.
    fn get_node(&self, index: usize) -> Option<&N> {
        // We first get the `Option<&Option<N>>` which handles the bounds check.
        // Then, `and_then` unwraps the outer Option and `as_ref` converts `&Option<N>` to `Option<&N>`.
        self.nodes.get(index).and_then(|node_opt| node_opt.as_ref())
    }

    /// Returns the total number of valid, non-tombstoned nodes in the graph.
    ///
    /// Note: This is an O(V) operation for `DynamicGraph` as it must iterate
    /// to ignore any removed ("tombstoned") nodes.
    fn number_nodes(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_some()).count()
    }

    /// Checks if an edge exists from node `a` to node `b` with any key. Every stored edge joins
    /// two live nodes, so no further check is needed.
    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.edges
            .get(a)
            .is_some_and(|edge_list| edge_list.iter().any(|(target, _)| *target == b))
    }

    /// Checks if the edge from `a` to `b` with key `key` exists.
    fn contains_edge_keyed(&self, a: usize, b: usize, key: W::Key) -> bool {
        self.edges.get(a).is_some_and(|edge_list| {
            edge_list
                .iter()
                .any(|(target, weight)| *target == b && weight.edge_key() == key)
        })
    }

    /// Returns the number of edges between live nodes, kept as a count so this is O(1).
    fn number_edges(&self) -> usize {
        self.live_edges
    }

    fn get_all_nodes(&self) -> Vec<&N> {
        self.nodes.iter().filter_map(Option::as_ref).collect()
    }

    /// Retrieves a list of all outgoing edges from a given source node, in insertion order.
    /// Returns `None` if the source node does not exist.
    /// The returned vector contains tuples of `(target_node_index, edge_weight_reference)`.
    fn get_edges(&self, source: usize) -> Option<Vec<(usize, &W)>> {
        if !self.contains_node(source) {
            return None;
        }
        Some(
            self.edges[source]
                .iter()
                .map(|(target, weight)| (*target, weight))
                .collect(),
        )
    }

    fn get_last_index(&self) -> Option<usize> {
        self.nodes.iter().rposition(|node| node.is_some())
    }

    /// Checks if a valid, non-tombstoned root node has been designated.
    fn contains_root_node(&self) -> bool {
        // Use `is_some_and` to check both that a root_index exists AND
        // that the node at that index is still valid.
        self.root_index
            .is_some_and(|index| self.contains_node(index))
    }

    /// Retrieves a reference to the payload of the designated root node, if it exists and is valid.
    fn get_root_node(&self) -> Option<&N> {
        // We chain options: get the root index, then use that to get the node data.
        self.root_index.and_then(|index| self.get_node(index))
    }

    /// Retrieves the index of the designated root node, if one exists and is valid.
    fn get_root_index(&self) -> Option<usize> {
        // We use `filter` to return the index only if the node at that index is still valid.
        self.root_index.filter(|&index| self.contains_node(index))
    }
}
