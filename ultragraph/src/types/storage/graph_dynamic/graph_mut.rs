/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::vec::Vec;

use crate::{DynamicGraph, EdgeKey, GraphError, GraphMut, GraphView};

impl<N, W> GraphMut<N, W> for DynamicGraph<N, W>
where
    W: Clone + Default + EdgeKey,
{
    /// Adds a new node with the given node to the graph.
    ///
    /// This operation is amortized O(1). It adds a new entry for the node
    /// and its corresponding edge lists. It returns the stable `usize` index
    /// that will identify this node for its lifetime.
    fn add_node(&mut self, node: N) -> Result<usize, GraphError> {
        let index = self.nodes.len();
        self.nodes.push(Some(node));

        match self.num_edges_per_node {
            Some(edge_capacity) => {
                self.edges.push(Vec::with_capacity(edge_capacity));
                self.backward.push(Vec::with_capacity(edge_capacity));
            }
            None => {
                self.edges.push(Vec::default());
                self.backward.push(Vec::default());
            }
        }

        Ok(index)
    }

    /// Updates the node of an existing, non-tombstoned node.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if the index is out of bounds or if the
    /// node at that index has already been removed.
    fn update_node(&mut self, index: usize, node: N) -> Result<(), GraphError> {
        // `get_mut` performs the bounds check.
        match self.nodes.get_mut(index) {
            // Check if the node is not a tombstone.
            Some(node_slot) if node_slot.is_some() => {
                *node_slot = Some(node); // Update the node
                Ok(())
            }
            // Either out of bounds or a tombstone, it's considered not found.
            _ => Err(GraphError::NodeNotFound(index)),
        }
    }

    /// Removes a node from the graph, leaving a tombstone at its position.
    ///
    /// The position is never issued again. Every edge out of the node and every edge into it is
    /// removed, from the other end's list as well, so no edge in any state points at a tombstone.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if the index is out of bounds or if the
    /// node at that index has already been removed.
    fn remove_node(&mut self, index: usize) -> Result<(), GraphError> {
        if !self.contains_node(index) {
            return Err(GraphError::NodeNotFound(index));
        }
        self.nodes[index] = None;

        // The edges out of the node leave their targets' backward lists; the edges into it leave
        // their sources' forward lists. A self-loop sits in both lists of the node and is one edge,
        // counted once with the outgoing edges. Each neighbour's list is scanned once, however
        // many keyed edges join it to the node.
        let outgoing = core::mem::take(&mut self.edges[index]);
        let incoming = core::mem::take(&mut self.backward[index]);
        let removed = outgoing.len()
            + incoming
                .iter()
                .filter(|(source, _)| *source != index)
                .count();

        let mut targets: Vec<usize> = outgoing.into_iter().map(|(target, _)| target).collect();
        targets.sort_unstable();
        targets.dedup();
        for target in targets {
            self.backward[target].retain(|(source, _)| *source != index);
        }

        let mut sources: Vec<usize> = incoming.into_iter().map(|(source, _)| source).collect();
        sources.sort_unstable();
        sources.dedup();
        for source in sources {
            self.edges[source].retain(|(target, _)| *target != index);
        }
        self.live_edges -= removed;

        if self.root_index == Some(index) {
            self.root_index = None;
        }
        Ok(())
    }

    /// Adds the edge `(a, b, weight.edge_key())`, or replaces the payload of that edge when it
    /// exists. An edge between `a` and `b` with another key is left in place.
    ///
    /// # Errors
    /// Returns `GraphError::EdgeCreationError` if either end holds no live node.
    fn add_edge(&mut self, a: usize, b: usize, weight: W) -> Result<(), GraphError> {
        if !self.contains_node(a) || !self.contains_node(b) {
            return Err(GraphError::EdgeCreationError {
                source: a,
                target: b,
            });
        }

        let key = weight.edge_key();
        let existing = self.edges[a]
            .iter()
            .position(|(target, w)| *target == b && w.edge_key() == key);
        match existing {
            Some(position) => {
                // The backward entry holds the source and the key, and the key is unchanged.
                self.edges[a][position].1 = weight;
            }
            None => {
                self.edges[a].push((b, weight));
                self.backward[b].push((a, key));
                self.live_edges += 1;
            }
        }
        Ok(())
    }

    /// Removes every edge from `a` to `b`, whatever its key. O(degree(a) + degree(b)).
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if `a` holds no live node.
    /// Returns `GraphError::EdgeNotFoundError` if no edge from `a` to `b` exists.
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), GraphError> {
        if !self.contains_node(a) {
            return Err(GraphError::NodeNotFound(a));
        }

        let before = self.edges[a].len();
        self.edges[a].retain(|(target, _)| *target != b);
        let removed = before - self.edges[a].len();
        if removed == 0 {
            return Err(GraphError::EdgeNotFoundError {
                source: a,
                target: b,
            });
        }
        self.backward[b].retain(|(source, _)| *source != a);
        self.live_edges -= removed;
        Ok(())
    }

    /// Removes the edge from `a` to `b` with key `key` alone. O(degree(a) + degree(b)).
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if `a` holds no live node.
    /// Returns `GraphError::EdgeNotFoundError` if no edge from `a` to `b` carries `key`.
    fn remove_edge_keyed(&mut self, a: usize, b: usize, key: W::Key) -> Result<(), GraphError> {
        if !self.contains_node(a) {
            return Err(GraphError::NodeNotFound(a));
        }

        let position = self.edges[a]
            .iter()
            .position(|(target, w)| *target == b && w.edge_key() == key);
        let Some(position) = position else {
            return Err(GraphError::EdgeNotFoundError {
                source: a,
                target: b,
            });
        };
        self.edges[a].remove(position);
        self.backward[b].retain(|(source, k)| !(*source == a && *k == key));
        self.live_edges -= 1;
        Ok(())
    }

    /// Adds a new node and designates it as the graph's root node.
    ///
    /// # Errors
    /// Returns `GraphError::RootNodeAlreadyExists` when a root is already designated.
    fn add_root_node(&mut self, node: N) -> Result<usize, GraphError> {
        if self.root_index.is_none() {
            let index = self.add_node(node)?;
            self.root_index = Some(index);
            Ok(index)
        } else {
            Err(GraphError::RootNodeAlreadyExists)
        }
    }

    /// Clears all nodes and edges from the graph, resetting it to an empty state. The next
    /// position issued is 0.
    fn clear(&mut self) -> Result<(), GraphError> {
        self.nodes.clear();
        self.edges.clear();
        self.backward.clear();
        self.live_edges = 0;
        self.root_index = None;
        Ok(())
    }
}
