/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ContextId, ContextSnapshot, ContextoidId, ContextoidRecord, IdReserve, RelationRecord,
};

/// What a store of contexts does. Every operation is persistent; the crate's name says so, and no
/// operation name repeats it.
///
/// Every operation returns `impl Future<Output = Result<_, Self::Error>> + Send`, so a backend
/// that awaits a network can implement it without blocking, and a request handler can hold the
/// future across threads. The trait adds no runtime dependency; a synchronous backend returns
/// `core::future::ready`. Under edition 2024 a returned future captures the `&self` borrow and may
/// hold it for the future's life.
///
/// # Invariants every backend keeps
///
/// - A **context is a set of links**: `link` and `unlink` change membership and nothing else, and
///   `retract_context` retracts the container, its links and every reference to or from it, while
///   every contextoid it linked and every container it referenced survives.
/// - A **contextoid is immutable under its identifier**: `create_node` is idempotent for an
///   identifier already created under the same record and refused for one created under a
///   different record. A changed value is a new node under a fresh identifier.
/// - **Identity is the store's**: `create_node` is refused for an identifier the store did not hand
///   out through `reserve`. A container's identifier is never 0 and never reused.
/// - A **relation exists once between two nodes**: `create_edge` is idempotent for an identical
///   `(from, to, kind)`, refused for a different kind between the same pair, and refused when
///   either end is not held. `retract_node` removes every edge incident to the node.
/// - A **reference is a link between containers**: `attach` is idempotent, refused for a container
///   the store does not hold and for a self-reference; `detach` of a reference not held is not an
///   error. References are many to many and may form a cycle.
/// - `hydrate` **materialises one level**: the snapshot holds the container's nodes, the edges among
///   them, and one extra per container it references, carrying that container's identifier, name,
///   nodes and edges. The referenced containers' own references are not followed.
/// - A `Root` record is created like any other node. A backend that cannot hold one declines it
///   with its own error and documents the policy.
pub trait ContextStorage {
    /// The backend's error. It carries the backend's own refusals; a projection failure is the
    /// caller's and never reaches here.
    type Error: core::fmt::Debug + core::fmt::Display;

    /// What names a context to hydrate: a stored identifier, or whatever the backend answers by.
    type Slice;

    /// `n` identifiers the store has made unique, for nodes the caller will create. A reserve is a
    /// lease with no return.
    fn reserve(&self, n: usize) -> impl Future<Output = Result<IdReserve, Self::Error>> + Send;

    /// Creates a container under a name and returns its identifier.
    fn create_context(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<ContextId, Self::Error>> + Send;

    /// Retracts a container, its links and its references. Every contextoid it linked survives.
    fn retract_context(
        &self,
        context: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Creates contextoids under identifiers taken from a reserve.
    fn create_node(
        &self,
        nodes: &[ContextoidRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Retracts a contextoid from every container that linked it, with every edge incident to it.
    fn retract_node(
        &self,
        node: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Creates relations between contextoids the store holds.
    fn create_edge(
        &self,
        edges: &[RelationRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Retracts the relation from one contextoid to another.
    fn retract_edge(
        &self,
        from: ContextoidId,
        to: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Links contextoids into a container. Idempotent.
    fn link(
        &self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Removes contextoids from a container. A node not linked is not an error.
    fn unlink(
        &self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// References a container from another, so hydrating the first materialises the second as an
    /// extra context. Idempotent.
    fn attach(
        &self,
        context: ContextId,
        extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// Removes a reference. A reference not held is not an error.
    fn detach(
        &self,
        context: ContextId,
        extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// The record held under each identifier, in the order given, or `None` where the store holds
    /// none.
    fn lookup(
        &self,
        ids: &[ContextoidId],
    ) -> impl Future<Output = Result<Vec<Option<ContextoidRecord>>, Self::Error>> + Send;

    /// A context as of a consistent point: its nodes, the edges among them, and the contexts it
    /// references, materialised one level deep.
    fn hydrate(
        &self,
        spec: &Self::Slice,
    ) -> impl Future<Output = Result<ContextSnapshot, Self::Error>> + Send;
}
