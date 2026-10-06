/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::{format, string::ToString, vec, vec::Vec};
use ultragraph::*;

use crate::ContextoidId;
use crate::{
    Context, ContextIndexError, Contextoid, ContextuableGraph, Datable, RelationKind,
    SpaceTemporal, Spatial, Temporal,
};
use deep_causality_core::Identifiable;

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> ContextuableGraph<D, S, T, ST> for Context<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn add_node(&mut self, value: Contextoid<D, S, T, ST>) -> Result<usize, ContextIndexError> {
        let contextoid_id = value.id();
        if self.id_to_index_map.contains_key(&contextoid_id) {
            return Err(ContextIndexError(format!(
                "Cannot add node. A contextoid with ID {contextoid_id} is already in the context"
            )));
        }
        let index = match self.base_context.add_node(value) {
            Ok(index) => index,
            Err(e) => return Err(ContextIndexError(e.to_string())),
        };
        self.id_to_index_map.insert(contextoid_id, index);
        Ok(index)
    }

    /// Returns only true if the context contains the contextoid with the given index.
    fn contains_node(&self, index: usize) -> bool {
        self.base_context.contains_node(index)
    }

    /// Returns a reference to the contextoid with the given index.
    /// If the context does not contain the contextoid, it will return None.
    fn get_node(&self, index: usize) -> Option<&Contextoid<D, S, T, ST>> {
        self.base_context.get_node(index)
    }

    fn remove_node(&mut self, node_id: ContextoidId) -> Result<(), ContextIndexError> {
        if let Some(&index_to_remove) = self.id_to_index_map.get(&node_id) {
            // Try to remove from the underlying graph first.
            self.base_context
                .remove_node(index_to_remove)
                .map_err(|e| ContextIndexError(e.to_string()))?;

            // If successful, then remove the entry from our map to stay in sync.
            self.id_to_index_map.remove(&node_id);

            Ok(())
        } else {
            Err(ContextIndexError(format!(
                "Cannot remove node. Contextoid with ID {node_id} not found in context"
            )))
        }
    }

    fn update_node(
        &mut self,
        node_id: ContextoidId,
        new_node: Contextoid<D, S, T, ST>,
    ) -> Result<(), ContextIndexError> {
        if let Some(&index_to_update) = self.id_to_index_map.get(&node_id) {
            let new_node_id = new_node.id(); // Extract the new node's ID
            if new_node_id != node_id && self.id_to_index_map.contains_key(&new_node_id) {
                return Err(ContextIndexError(format!(
                    "Cannot update node. A contextoid with ID {new_node_id} is already in the context"
                )));
            }

            self.base_context
                .update_node(index_to_update, new_node)
                .map_err(|e| ContextIndexError(e.to_string()))?;

            // Update id_to_index_map if the ID changed to maintain consistency
            if new_node_id != node_id {
                self.id_to_index_map.remove(&node_id);
                self.id_to_index_map.insert(new_node_id, index_to_update);
            }

            Ok(())
        } else {
            Err(ContextIndexError(format!(
                "Cannot update node. Contextoid with ID {node_id} not found in context"
            )))
        }
    }

    /// Adds a new weighted edge between two nodes.
    /// Returns either Ok after success, or ContextIndexError if
    /// any of the nodes are not in the context.
    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError> {
        if !self.contains_node(a) {
            return Err(ContextIndexError(format!("index a {a} not found")));
        };

        if !self.contains_node(b) {
            return Err(ContextIndexError(format!("index b {b} not found")));
        };

        self.base_context.add_edge(a, b, weight).map_err(|e| {
            ContextIndexError(format!("Failed to add edge for index a {a} and b {b}: {e}"))
        })
    }

    /// Returns only true if the context contains the edge between the two nodes.
    /// If the context does not contain the edge or any of the nodes it will return false.
    /// You may want to call contains_node first to ascertain that the nodes are in the context.
    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.base_context.contains_edge(a, b)
    }

    /// Returns the relation the edge from `a` to `b` carries, or `None` when no such edge exists.
    fn get_edge(&self, a: usize, b: usize) -> Option<&RelationKind> {
        self.base_context
            .get_edges(a)?
            .into_iter()
            .find(|(target, _)| *target == b)
            .map(|(_, weight)| weight)
    }

    /// Removes an edge between two nodes.
    /// Returns either Ok after success, or ContextIndexError if
    /// any of the nodes are not in the context.
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError> {
        if !self.contains_node(a) {
            return Err(ContextIndexError("index a not found".into()));
        };

        if !self.contains_node(b) {
            return Err(ContextIndexError("index b not found".into()));
        };
        self.base_context
            .remove_edge(a, b)
            .map_err(|e| ContextIndexError(e.to_string()))
    }

    /// Returns the number of nodes in the context. Alias for node_count().
    fn size(&self) -> usize {
        self.base_context.number_nodes()
    }

    /// Returns true if the context contains no nodes.
    fn is_empty(&self) -> bool {
        self.base_context.is_empty()
    }

    /// Returns the number of nodes in the context.
    fn number_of_nodes(&self) -> usize {
        self.base_context.number_nodes()
    }

    /// Returns the number of edges in the context.
    fn number_of_edges(&self) -> usize {
        self.base_context.number_edges()
    }

    fn freeze(&mut self) {
        if self.base_context.is_frozen() {
            return;
        }
        // Freezing keeps the live nodes in order and drops the removed ones, so a live node's
        // index in the frozen graph is its rank among the live nodes.
        let end = self
            .base_context
            .get_last_index()
            .map_or(0, |last| last + 1);
        let mut frozen_index: Vec<Option<usize>> = vec![None; end];
        let mut rank = 0;
        for (index, slot) in frozen_index.iter_mut().enumerate() {
            if self.base_context.contains_node(index) {
                *slot = Some(rank);
                rank += 1;
            }
        }
        self.base_context.freeze();

        // Every stored index follows its node. An entry naming a removed node named nothing
        // before the freeze and would name another node after it, so it is dropped.
        let remap = |index: &mut usize| match frozen_index.get(*index).copied().flatten() {
            Some(frozen) => {
                *index = frozen;
                true
            }
            None => false,
        };
        self.id_to_index_map.retain(|_, index| remap(index));
        self.current_data_map.retain(|_, index| remap(index));
        self.previous_data_map.retain(|_, index| remap(index));
        self.current_index_map.retain(|_, index| remap(index));
        self.previous_index_map.retain(|_, index| remap(index));
    }

    fn unfreeze(&mut self) {
        self.base_context.unfreeze();
    }

    fn is_frozen(&self) -> bool {
        self.base_context.is_frozen()
    }

    fn outbound_edges(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = usize> + '_, ContextIndexError> {
        self.base_context
            .outbound_edges(index)
            .map_err(|e| traversal_error(e, index))
    }

    fn inbound_edges(
        &self,
        index: usize,
    ) -> Result<impl Iterator<Item = usize> + '_, ContextIndexError> {
        self.base_context
            .inbound_edges(index)
            .map_err(|e| traversal_error(e, index))
    }
}

/// The context's account of a failed neighbour listing.
fn traversal_error(error: GraphError, index: usize) -> ContextIndexError {
    ContextIndexError(match error {
        GraphError::GraphNotFrozen => {
            "Cannot list neighbours. The context is not frozen; call freeze first".to_string()
        }
        GraphError::NodeNotFound(_) => {
            format!("Cannot list neighbours. No node at index {index}")
        }
        other => other.to_string(),
    })
}
