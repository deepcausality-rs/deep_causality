/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{EdgeKey, GraphError, GraphTraversal, Projection, ProjectionView};

/// One entry per passing edge, under either mode.
impl<N, W, P> GraphTraversal<N, W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    /// # Errors
    /// Returns `GraphError::NodeNotFound` when `a` does not pass: removed, never issued or
    /// failing the node test.
    fn outbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.passes(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        Ok(self.out_edges(a).map(|(target, _)| target))
    }

    /// # Errors
    /// Returns `GraphError::NodeNotFound` when `a` does not pass.
    fn inbound_edges(&self, a: usize) -> Result<impl Iterator<Item = usize> + '_, GraphError> {
        if !self.passes(a) {
            return Err(GraphError::NodeNotFound(a));
        }
        Ok(self.in_edges(a).map(|(source, _)| source))
    }
}
