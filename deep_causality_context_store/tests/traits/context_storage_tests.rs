/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A backend that holds nothing pins the trait's shape: thirteen operations, each a `Send` future
//! that `block_on` completes. Expected values are the literals the null backend returns.
//!
//! Corner cases (rows A to K): A a reserve of zero and a lookup of no identifiers in
//! `test_empty_reserve_and_lookup`; every other row is the in-memory backend's (group 5) and is
//! n/a for a shape test.
use core::future::{Future, ready};
use deep_causality_context_store::utils_test::block_on;
use deep_causality_context_store::{
    ContextId, ContextRecord, ContextSnapshot, ContextStorage, ContextoidId, ContextoidRecord,
    IdReserve, NodeRecord, RelationKind, RelationRecord,
};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq)]
struct NullError;

impl Display for NullError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NullError")
    }
}

/// A backend that holds nothing and refuses nothing: every operation is a ready future.
struct NullStorage;

impl ContextStorage for NullStorage {
    type Error = NullError;
    type Slice = ContextId;

    fn reserve(&self, n: usize) -> impl Future<Output = Result<IdReserve, Self::Error>> + Send {
        ready(Ok(IdReserve::new((1..=n as ContextoidId).collect())))
    }

    fn create_context(
        &self,
        _name: &str,
    ) -> impl Future<Output = Result<ContextId, Self::Error>> + Send {
        ready(Ok(1))
    }

    fn retract_context(
        &self,
        _context: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn create_node(
        &self,
        _nodes: &[ContextoidRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn retract_node(
        &self,
        _node: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn create_edge(
        &self,
        _edges: &[RelationRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn retract_edge(
        &self,
        _from: ContextoidId,
        _to: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn link(
        &self,
        _context: ContextId,
        _nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn unlink(
        &self,
        _context: ContextId,
        _nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn attach(
        &self,
        _context: ContextId,
        _extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn detach(
        &self,
        _context: ContextId,
        _extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(Ok(()))
    }

    fn lookup(
        &self,
        ids: &[ContextoidId],
    ) -> impl Future<Output = Result<Vec<Option<ContextoidRecord>>, Self::Error>> + Send {
        ready(Ok(vec![None; ids.len()]))
    }

    fn hydrate(
        &self,
        spec: &Self::Slice,
    ) -> impl Future<Output = Result<ContextSnapshot, Self::Error>> + Send {
        ready(Ok(ContextSnapshot::new(
            ContextRecord::new(*spec, "null".to_string()),
            vec![],
            vec![],
            vec![],
        )))
    }
}

fn assert_send<F: Send>(_: &F) {}

/// Compiles only because every operation's future carries the `Send` bound the trait states.
fn every_future_is_send<S>(storage: &S, slice: &S::Slice)
where
    S: ContextStorage + Sync,
    S::Slice: Sync,
{
    let node = ContextoidRecord::new(1, NodeRecord::Root);
    let edge = RelationRecord::new(1, 2, RelationKind::Datial);
    assert_send(&storage.reserve(1));
    assert_send(&storage.create_context("c"));
    assert_send(&storage.retract_context(1));
    assert_send(&storage.create_node(std::slice::from_ref(&node)));
    assert_send(&storage.retract_node(1));
    assert_send(&storage.create_edge(std::slice::from_ref(&edge)));
    assert_send(&storage.retract_edge(1, 2));
    assert_send(&storage.link(1, &[1]));
    assert_send(&storage.unlink(1, &[1]));
    assert_send(&storage.attach(1, 2));
    assert_send(&storage.detach(1, 2));
    assert_send(&storage.lookup(&[1]));
    assert_send(&storage.hydrate(slice));
}

#[test]
fn test_the_futures_are_send() {
    every_future_is_send(&NullStorage, &1);
}

#[test]
fn test_every_operation_completes_with_block_on() {
    let storage = NullStorage;
    let mut reserve = block_on(storage.reserve(2)).unwrap();
    assert_eq!(reserve.next(), Some(1));
    assert_eq!(reserve.next(), Some(2));
    assert_eq!(block_on(storage.create_context("c")), Ok(1));
    assert_eq!(block_on(storage.retract_context(1)), Ok(()));
    let node = ContextoidRecord::new(1, NodeRecord::Root);
    assert_eq!(block_on(storage.create_node(&[node])), Ok(()));
    assert_eq!(block_on(storage.retract_node(1)), Ok(()));
    let edge = RelationRecord::new(1, 2, RelationKind::Spatial);
    assert_eq!(block_on(storage.create_edge(&[edge])), Ok(()));
    assert_eq!(block_on(storage.retract_edge(1, 2)), Ok(()));
    assert_eq!(block_on(storage.link(1, &[1])), Ok(()));
    assert_eq!(block_on(storage.unlink(1, &[1])), Ok(()));
    assert_eq!(block_on(storage.attach(1, 2)), Ok(()));
    assert_eq!(block_on(storage.detach(1, 2)), Ok(()));
    assert_eq!(block_on(storage.lookup(&[1, 2])), Ok(vec![None, None]));
    let snapshot = block_on(storage.hydrate(&7)).unwrap();
    assert_eq!(snapshot.context().id(), 7);
    assert_eq!(snapshot.context().name(), "null");
    assert!(snapshot.nodes().is_empty());
}

#[test]
fn test_null_error_displays() {
    assert_eq!(NullError.to_string(), "NullError");
}

#[test]
fn test_empty_reserve_and_lookup() {
    let storage = NullStorage;
    let mut reserve = block_on(storage.reserve(0)).unwrap();
    assert_eq!(reserve.remaining(), 0);
    assert_eq!(reserve.next(), None);
    assert_eq!(block_on(storage.lookup(&[])), Ok(vec![]));
}
