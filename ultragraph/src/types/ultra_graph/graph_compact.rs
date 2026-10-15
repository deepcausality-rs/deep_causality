/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{EdgeKey, GraphError, GraphState, UltraGraphContainer};
use alloc::vec::Vec;

impl<N, W> UltraGraphContainer<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Removes every tombstone and renumbers the live nodes in their order, returning for each
    /// old position its new position, or `None` for a tombstone. See
    /// [`DynamicGraph::compact_ids`](crate::DynamicGraph::compact_ids).
    ///
    /// # Errors
    /// Returns `GraphError::GraphIsFrozen` in the frozen state; the graph is unchanged.
    pub fn compact_ids(&mut self) -> Result<Vec<Option<usize>>, GraphError> {
        match &mut self.state {
            GraphState::Dynamic(g) => Ok(g.compact_ids()),
            GraphState::Static(_) => Err(GraphError::GraphIsFrozen),
        }
    }
}
