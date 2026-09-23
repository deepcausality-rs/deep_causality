/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `ContextStore::hydrate`: the backend's snapshot restored, with referenced containers as
//! extras. Expected values are the records written through the backend.
//!
//! Corner cases (rows A to K): A an empty container, `test_an_empty_container_hydrates`; C the
//! same container hydrated twice agrees, `test_two_hydrations_agree`; a record the type cannot
//! hold, `test_a_projection_failure_names_the_node`; every other row n/a.

use deep_causality_context::{
    ContextStore, ContextuableGraph, ExtendableContextuableGraph, StoreErrorEnum, UniformContext,
};
use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContextId, ContextStorage, ContextoidId, ContextoidRecord, DataRecord, MemoryStorageError,
    NodeRecord, ProjectionError, RelationKind, RelationRecord, TimeRecord, TimeScale,
};

/// A base container with a root, a count and a tick joined by two edges, referencing a second
/// container holding one count. Returns the two container identifiers and the four node
/// identifiers.
fn stored_world(storage: &MemoryStorage) -> (ContextId, ContextId, Vec<ContextoidId>) {
    let n: Vec<ContextoidId> = block_on(storage.reserve(4)).unwrap().collect();
    block_on(storage.create_node(&[
        ContextoidRecord::new(n[0], NodeRecord::Root),
        ContextoidRecord::new(n[1], NodeRecord::Data(DataRecord::Count(5))),
        ContextoidRecord::new(
            n[2],
            NodeRecord::Time(TimeRecord::Discrete {
                scale: TimeScale::Steps,
                tick: 9,
            }),
        ),
        ContextoidRecord::new(n[3], NodeRecord::Data(DataRecord::Count(7))),
    ]))
    .unwrap();
    block_on(storage.create_edge(&[
        RelationRecord::new(n[0], n[1], RelationKind::Datial),
        RelationRecord::new(n[0], n[2], RelationKind::Temporal),
    ]))
    .unwrap();
    let base = block_on(storage.create_context("base")).unwrap();
    let extra = block_on(storage.create_context("weather")).unwrap();
    block_on(storage.link(base, &n[..3])).unwrap();
    block_on(storage.link(extra, &n[3..])).unwrap();
    block_on(storage.attach(base, extra)).unwrap();
    (base, extra, n)
}

#[test]
fn test_a_stored_context_with_references_hydrates_whole() {
    let storage = MemoryStorage::new();
    let (base, extra, n) = stored_world(&storage);
    let store = ContextStore::new(storage.clone());
    let mut ctx: UniformContext = block_on(store.hydrate(&base)).unwrap();
    assert_eq!(ctx.name(), "base");
    assert_eq!(ctx.number_of_nodes(), 3);
    assert_eq!(ctx.number_of_edges(), 2);
    let root = ctx.get_node_index_by_id(n[0]).unwrap();
    let count = ctx.get_node_index_by_id(n[1]).unwrap();
    assert_eq!(ctx.get_edge(root, count), Some(&RelationKind::Datial));
    assert_eq!(ctx.extra_ctx_get_name(extra), Some("weather"));
    ctx.extra_ctx_set_current_id(extra).unwrap();
    assert_eq!(ctx.extra_ctx_node_count().unwrap(), 1);
    assert_eq!(
        ctx.snapshot().unwrap(),
        block_on(storage.hydrate(&base)).unwrap()
    );
}

#[test]
fn test_two_hydrations_agree() {
    let storage = MemoryStorage::new();
    let (base, extra, _) = stored_world(&storage);
    let store = ContextStore::new(storage);
    let a: UniformContext = block_on(store.hydrate(&base)).unwrap();
    let b: UniformContext = block_on(store.hydrate(&base)).unwrap();
    assert_eq!(a.snapshot().unwrap(), b.snapshot().unwrap());
    assert_eq!(a.extra_ctx_get_name(extra), b.extra_ctx_get_name(extra));
}

#[test]
fn test_an_empty_container_hydrates() {
    let storage = MemoryStorage::new();
    let id = block_on(storage.create_context("empty")).unwrap();
    let store = ContextStore::new(storage);
    let ctx: UniformContext = block_on(store.hydrate(&id)).unwrap();
    assert!(ctx.is_empty());
    assert_eq!(ctx.name(), "empty");
}

#[test]
fn test_an_unknown_slice_is_the_backends_refusal() {
    let store = ContextStore::new(MemoryStorage::new());
    let result = block_on(store.hydrate::<_, _, _, _>(&99)).map(|_: UniformContext| ());
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Storage(MemoryStorageError::UnknownContext(
            99
        )))
    );
}

#[test]
fn test_a_projection_failure_names_the_node() {
    let storage = MemoryStorage::new();
    let n: Vec<ContextoidId> = block_on(storage.reserve(1)).unwrap().collect();
    block_on(storage.create_node(&[ContextoidRecord::new(
        n[0],
        NodeRecord::Data(DataRecord::Number(1.5)),
    )]))
    .unwrap();
    let id = block_on(storage.create_context("numbers")).unwrap();
    block_on(storage.link(id, &n)).unwrap();
    let store = ContextStore::new(storage);
    let result = block_on(store.hydrate::<_, _, _, _>(&id)).map(|_: UniformContext| ());
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Projection(ProjectionError::WrongPayload(
            n[0], "Count", "Number"
        )))
    );
}
