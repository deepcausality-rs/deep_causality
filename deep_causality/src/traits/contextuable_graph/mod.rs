/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::ContextIndexError;
use crate::prelude::{Contextoid, ContextoidId, Datable, RelationKind, Symbolic};
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;

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
pub trait ContextuableGraph<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn add_node(&mut self, value: Contextoid<D, S, T, ST, SYM, VS, VT>) -> usize;
    fn contains_node(&self, index: usize) -> bool;
    fn get_node(&self, index: usize) -> Option<&Contextoid<D, S, T, ST, SYM, VS, VT>>;
    fn remove_node(&mut self, node_id: ContextoidId) -> Result<(), ContextIndexError>;
    fn update_node(
        &mut self,
        node_id: ContextoidId,
        new_node: Contextoid<D, S, T, ST, SYM, VS, VT>,
    ) -> Result<(), ContextIndexError>;
    fn add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError>;
    fn contains_edge(&self, a: usize, b: usize) -> bool;
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
pub trait ExtendableContextuableGraph<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    // Creates a new context and returns the index of the new context.
    fn extra_ctx_add_new(&mut self, capacity: usize, default: bool) -> u64;

    /// Creates a new extra context with a specific ID.
    fn extra_ctx_add_new_with_id(
        &mut self,
        id: u64,
        capacity: usize,
        default: bool,
    ) -> Result<(), ContextIndexError>;

    fn extra_ctx_check_exists(&self, idx: u64) -> bool;
    fn extra_ctx_get_current_id(&self) -> u64;
    fn extra_ctx_set_current_id(&mut self, idx: u64) -> Result<(), ContextIndexError>;
    fn extra_ctx_unset_current_id(&mut self) -> Result<(), ContextIndexError>;

    fn extra_ctx_add_node(
        &mut self,
        value: Contextoid<D, S, T, ST, SYM, VS, VT>,
    ) -> Result<usize, ContextIndexError>;
    fn extra_ctx_contains_node(&self, index: usize) -> bool;
    fn extra_ctx_get_node(
        &self,
        index: usize,
    ) -> Result<&Contextoid<D, S, T, ST, SYM, VS, VT>, ContextIndexError>;
    fn extra_ctx_remove_node(&mut self, index: usize) -> Result<(), ContextIndexError>;
    fn extra_ctx_add_edge(
        &mut self,
        a: usize,
        b: usize,
        weight: RelationKind,
    ) -> Result<(), ContextIndexError>;
    fn extra_ctx_contains_edge(&self, a: usize, b: usize) -> bool;
    fn extra_ctx_remove_edge(&mut self, a: usize, b: usize) -> Result<(), ContextIndexError>;
    fn extra_ctx_size(&self) -> Result<usize, ContextIndexError>;
    fn extra_ctx_is_empty(&self) -> Result<bool, ContextIndexError>;
    fn extra_ctx_node_count(&self) -> Result<usize, ContextIndexError>;
    fn extra_ctx_edge_count(&self) -> Result<usize, ContextIndexError>;
}
