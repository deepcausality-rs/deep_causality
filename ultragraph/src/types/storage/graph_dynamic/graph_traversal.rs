/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{DynamicGraph, EdgeKey, GraphError, GraphTraversal, GraphView};

impl<N, W> GraphTraversal<N, W> for DynamicGraph<N, W>
where
    W: Clone + Default + EdgeKey,
{
    /// The targets of the node's outgoing edges, one per edge. The order is not specified:
    /// insertion order until a freeze or a compaction reorders the list.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node: past the table or removed.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.contains_node(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        Ok(self.edges[a].iter().map(|(target, _)| *target))
    }

    /// The sources of the node's incoming edges, one per edge. The order is not specified:
    /// insertion order until a freeze or a compaction reorders the list.
    ///
    /// # Errors
    /// Returns `GraphError::NodeNotFound` if position `a` holds no node: past the table or removed.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.contains_node(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        Ok(self.backward[a].iter().map(|(source, _)| *source))
    }
}
