/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{EdgeKey, GraphError, GraphState, GraphTraversal, GraphView, UltraGraphContainer};

impl<N, W> GraphTraversal<N, W> for UltraGraphContainer<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Returns an iterator over the indices of all nodes that have an outbound edge from the given node `a`.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::NodeNotFound` if position `a` holds no node, in either state.
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        match &self.state {
            GraphState::Static(g) => {
                // Delegate to the CsmGraph's hyper-optimized implementation.
                g.outbound_edges(a)
            }
            GraphState::Dynamic(g) => {
                // The position is checked first, so a tombstone names the same error in both
                // states; a live node needs the frozen graph for traversal.
                if g.contains_node(a) {
                    Err(GraphError::GraphNotFrozen)
                } else {
                    Err(GraphError::NodeNotFound(a))
                }
            }
        }
    }
    /// Returns an iterator over the indices of all nodes that have an inbound edge to the given node `a`.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::NodeNotFound` if position `a` holds no node, in either state.
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        match &self.state {
            GraphState::Static(g) => {
                // Delegate to the CsmGraph's hyper-optimized implementation.
                g.inbound_edges(a)
            }
            GraphState::Dynamic(g) => {
                // The position is checked first, so a tombstone names the same error in both
                // states; a live node needs the frozen graph for traversal.
                if g.contains_node(a) {
                    Err(GraphError::GraphNotFrozen)
                } else {
                    Err(GraphError::NodeNotFound(a))
                }
            }
        }
    }
}
