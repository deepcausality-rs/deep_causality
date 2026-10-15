/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::types::ultra_graph::traversal_iter::TraversalIter;
use crate::{EdgeKey, GraphError, GraphState, GraphTraversal, UltraGraphContainer};

impl<N, W> GraphTraversal<N, W> for UltraGraphContainer<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Returns an iterator over the targets of the edges out of node `a`, one entry per edge, in
    /// either state. The frozen state reads its CSR slice; the dynamic state reads the node's
    /// forward list.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node, in either state.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.outbound_edges(a).map(TraversalIter::Frozen),
            GraphState::Dynamic(g) => g.outbound_edges(a).map(TraversalIter::Dynamic),
        }
    }

    /// Returns an iterator over the sources of the edges into node `a`, one entry per edge, in
    /// either state.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node, in either state.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.inbound_edges(a).map(TraversalIter::Frozen),
            GraphState::Dynamic(g) => g.inbound_edges(a).map(TraversalIter::Dynamic),
        }
    }
}
