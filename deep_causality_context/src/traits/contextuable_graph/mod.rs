/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextId;
use crate::ContextoidId;
use crate::errors::ContextIndexError;
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::{Contextoid, Datable, RelationKind};

/// Trait for graph containing context-aware nodes.
///
/// D: Datable trait object
/// S: Spatial trait object
/// T: Temporable trait object
/// ST: SpaceTemporal trait object
/// V: Numeric type for dimension values
///
/// Provides methods for:
/// - Adding/removing nodes and edges
/// - Checking if nodes/edges exist
/// - Getting node references
/// - Getting graph size and counts
///
/// Nodes are Contextoid objects implementing required traits.
/// Edges have a relation kind weight.
///
/// Methods return Result or Option types for error handling.
///
#[allow(clippy::type_complexity)]
pub trait ContextuableGraph<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    fn add_node(&mut self, value: Contextoid<D, S, T, ST>) -> Result<usize, ContextIndexError>;
    fn contains_node(&self, index: usize) -> bool;
    fn get_node(&self, index: usize) -> Option<&Contextoid<D, S, T, ST>>;
    fn remove_node(&mut self, node_id: ContextoidId) -> Result<(), ContextIndexError>;
    fn update_node(
        &mut self,
        node_id: ContextoidId,
        new_node: Contextoid<D, S, T, ST>,
    ) -> Result<(), ContextIndexError>;
    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError>;
    fn contains_edge(&self, a: usize, b: usize) -> bool;
    /// Returns the relation the edge from `a` to `b` carries, or `None` when no such edge
    /// exists. This is the read counterpart of `add_edge`: without it the relation a caller
    /// supplies is not recoverable from the context.
    fn get_edge(&self, a: usize, b: usize) -> Option<&RelationKind>;
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError>;
    fn size(&self) -> usize;
    fn is_empty(&self) -> bool;
    // Corrected method names
    fn number_of_nodes(&self) -> usize;
    fn number_of_edges(&self) -> usize;
}

/// Trait for poly-contextuable causal graphs.
/// By default, the context graph is assumed to be a single-context graph.
///
/// This trait supports multiple contexts by extending the ContextuableGraph trait.
///
/// Extends ContextuableGraph trait with methods for:
///
/// - Creating and managing additional "contexts"
/// - Setting a current context ID
/// - Context-specific node/edge methods
///
/// Provides methods for:
///
/// - Creating new contexts
/// - Checking if a context ID exists
/// - Getting/setting current context ID
/// - Context-specific node and edge methods
///
/// Nodes are Contextoid objects implementing required traits.
/// Edges have a relation kind weight.
///
/// Methods return Result or Option types for error handling.
///
#[allow(clippy::type_complexity)]
pub trait ExtendableContextuableGraph<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// Creates a new, empty, named "extra" context and adds it to the collection.
    ///
    /// The identifier is one more than the highest extra-context identifier held, and 1 when none
    /// is held, so an identifier added through `extra_ctx_add_new_with_id` is never allocated
    /// again.
    ///
    /// # Parameters
    /// - `name`: The name the extra context is referenced by.
    /// - `capacity`: The initial storage capacity to pre-allocate for the new context's graph.
    /// - `default`: If `true`, the newly created context is immediately set as the
    ///   currently active context for subsequent `extra_ctx_*` operations.
    ///
    /// # Returns
    /// The `ContextId` assigned to the newly created context.
    fn extra_ctx_add_new(&mut self, name: &str, capacity: usize, default: bool) -> ContextId;

    /// Creates a new, named extra context with a specific, caller-provided ID.
    ///
    /// This is what a context hydrated from a store uses: the extra lands under the identifier
    /// the store holds it by.
    ///
    /// # Parameters
    /// - `id`: The `ContextId` for the new context. 0 is refused, because 0 means no extra
    ///   context is current.
    /// - `name`: The name the extra context is referenced by.
    /// - `capacity`: The initial storage capacity for the new context's graph.
    /// - `default`: If `true`, this new context is set as the currently active one.
    ///
    /// # Returns
    /// - `Ok(())` if the context was created successfully.
    ///
    /// # Errors
    /// - `ContextIndexError` if a context with the provided `id` already exists, or if `id` is 0.
    fn extra_ctx_add_new_with_id(
        &mut self,
        id: ContextId,
        name: &str,
        capacity: usize,
        default: bool,
    ) -> Result<(), ContextIndexError>;

    /// The name of the extra context with the given ID, or `None` when no extra holds it.
    fn extra_ctx_get_name(&self, id: ContextId) -> Option<&str>;

    /// Checks if an extra context with the given ID exists.
    ///
    /// # Parameters
    /// - `idx`: The `u64` ID of the context to check for.
    ///
    /// # Returns
    /// `true` if a context with the specified ID exists, `false` otherwise.
    fn extra_ctx_check_exists(&self, idx: ContextId) -> bool;

    /// Gets the ID of the currently active extra context.
    ///
    /// By convention, an ID of `0` indicates that no extra context is currently active,
    /// and operations will target the primary context.
    ///
    /// # Returns
    /// The `u64` ID of the active context.
    fn extra_ctx_get_current_id(&self) -> ContextId;

    /// Sets the active extra context to the one identified by the given ID.
    ///
    /// All subsequent `extra_ctx_*` operations will be directed to this context
    /// until it is changed or unset.
    ///
    /// # Parameters
    /// - `idx`: The `u64` ID of the context to set as active.
    ///
    /// # Returns
    /// - `Ok(())` on success.
    ///
    /// # Errors
    /// - `ContextIndexError` if no context with the specified `idx` exists.
    fn extra_ctx_set_current_id(&mut self, idx: ContextId) -> Result<(), ContextIndexError>;

    /// Unsets the currently active extra context.
    ///
    /// After this operation, `extra_ctx_get_current_id` will return `0`,
    /// indicating that no extra context is active.
    ///
    /// # Returns
    /// - `Ok(())` if a context was active and has now been unset.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context was active to begin with.
    fn extra_ctx_unset_current_id(&mut self) -> Result<(), ContextIndexError>;

    /// Adds a `Contextoid` node to the currently active extra context.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Parameters
    /// - `value`: The `Contextoid` instance to add to the graph.
    ///
    /// # Returns
    /// - `Ok(usize)` containing the unique index of the newly added node within the active context's graph.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is currently active.
    fn extra_ctx_add_node(
        &mut self,
        value: Contextoid<D, S, T, ST>,
    ) -> Result<usize, ContextIndexError>;

    /// Checks if a node with the given index exists in the currently active extra context.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Parameters
    /// - `index`: The index of the node to check for.
    ///
    /// # Returns
    /// - `true` if a node with the specified index exists in the active context.
    /// - `false` if the node does not exist or if no extra context is currently active.
    fn extra_ctx_contains_node(&self, index: usize) -> bool;

    /// Retrieves an immutable reference to a `Contextoid` from the currently active extra context.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Parameters
    /// - `index`: The index of the node to retrieve.
    ///
    /// # Returns
    /// - `Ok(&Contextoid<...>)` containing a reference to the node if found.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is active or if the `index` is out of bounds for the active context's graph.
    fn extra_ctx_get_node(
        &self,
        index: usize,
    ) -> Result<&Contextoid<D, S, T, ST>, ContextIndexError>;

    /// Removes a node by its index from the currently active extra context.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    /// Note that this will also remove all edges connected to the specified node.
    ///
    /// # Parameters
    /// - `index`: The index of the node to remove.
    ///
    /// # Returns
    /// - `Ok(())` if the node was successfully removed.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is active or if the `index` is out of bounds.
    fn extra_ctx_remove_node(&mut self, index: usize) -> Result<(), ContextIndexError>;

    /// Adds a directed edge between two nodes in the currently active extra context.
    ///
    /// The edge is created from node `a` to node `b`. This operation is directed
    /// to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Parameters
    /// - `a`: The index of the source node.
    /// - `b`: The index of the target node.
    /// - `weight`: The `RelationKind` that describes the relationship between the nodes.
    ///
    /// # Returns
    /// - `Ok(())` if the edge was successfully added.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is active or if either `a` or `b` are invalid node indices.
    fn extra_ctx_add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError>;

    /// Checks if a directed edge exists between two nodes in the currently active extra context.
    ///
    /// The check is for an edge from node `a` to node `b`. This operation is directed
    /// to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Parameters
    /// - `a`: The index of the source node.
    /// - `b`: The index of the target node.
    ///
    /// # Returns
    /// - `true` if a directed edge from `a` to `b` exists in the active context.
    /// - `false` if the edge does not exist, if either index is invalid, or if no
    ///   extra context is currently active.
    fn extra_ctx_contains_edge(&self, a: usize, b: usize) -> bool;

    /// Returns the relation carried by the edge from node `a` to node `b` in the currently
    /// active extra context.
    ///
    /// This is the read counterpart of `extra_ctx_add_edge`, and is directed to the context set
    /// by `extra_ctx_set_current_id`.
    ///
    /// # Parameters
    /// - `a`: The index of the source node.
    /// - `b`: The index of the target node.
    ///
    /// # Returns
    /// - `Some(&RelationKind)` when a directed edge from `a` to `b` exists in the active context.
    /// - `None` if the edge does not exist, if either index is invalid, or if no extra context is
    ///   currently active.
    fn extra_ctx_get_edge(&self, a: usize, b: usize) -> Option<&RelationKind>;

    /// Removes a directed edge between two nodes in the currently active extra context.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Parameters
    /// - `a`: The index of the source node of the edge to remove.
    /// - `b`: The index of the target node of the edge to remove.
    ///
    /// # Returns
    /// - `Ok(())` if the edge was successfully removed.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is active or if the edge does not exist.
    fn extra_ctx_remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError>;

    /// Returns the number of nodes in the currently active extra context's graph.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Returns
    /// - `Ok(usize)` containing the total number of nodes in the active context's graph.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is currently active.
    fn extra_ctx_size(&self) -> Result<usize, ContextIndexError>;

    /// Checks if the currently active extra context's graph is empty (contains no nodes).
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Returns
    /// - `Ok(true)` if the active context's graph has zero nodes.
    /// - `Ok(false)` if it contains one or more nodes.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is currently active.
    fn extra_ctx_is_empty(&self) -> Result<bool, ContextIndexError>;

    /// Returns the total number of nodes in the currently active extra context's graph.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Returns
    /// - `Ok(usize)` containing the count of nodes in the active context's graph.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is currently active.
    fn extra_ctx_node_count(&self) -> Result<usize, ContextIndexError>;

    /// Returns the total number of edges in the currently active extra context's graph.
    ///
    /// This operation is directed to the context set by `extra_ctx_set_current_id`.
    ///
    /// # Returns
    /// - `Ok(usize)` containing the count of edges in the active context's graph.
    ///
    /// # Errors
    /// - `ContextIndexError` if no extra context is currently active.
    fn extra_ctx_edge_count(&self) -> Result<usize, ContextIndexError>;
}
