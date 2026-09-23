/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `ContextStore::store_branch`: a world stored as a new context that links what it shares and
//! creates what it changed. Expected values are the records written through the backend and the
//! nodes added to the branch; whether nodes were shared or created is observed through the
//! backend's identifier counter, which only `reserve` and `create_context` advance.
//!
//! Corner cases (rows A to K): A an empty branch, `test_an_empty_branch_stores_an_empty_container`;
//! C a branch identical to what the store holds, `test_a_shared_node_is_linked_not_copied`; the
//! absent spacetime in a branch, `test_an_unrecordable_branch_is_refused`; a backend whose reserve
//! answers short, `test_a_short_reserve_is_refused`; every other row n/a.

use deep_causality_context::{
    Context, ContextStore, Contextoid, ContextoidType, ContextuableGraph, Data,
    ExtendableContextuableGraph, NoSpaceTime, RelationKind, Root, StoreErrorEnum, TimeKind,
    UniformContext, UniformContextoid,
};
use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContextId, ContextSnapshot, ContextStorage, ContextoidId, ContextoidRecord, DataRecord,
    IdReserve, MemoryStorageError, NodeRecord, ProjectionError, RelationRecord,
};
use std::future::Future;

fn count(id: ContextoidId, value: u64) -> UniformContextoid {
    Contextoid::new(id, ContextoidType::Datoid(Data::new(id, value)))
}

/// A base container with a root and a count, referencing "weather" holding one count.
fn stored_world(storage: &MemoryStorage) -> (ContextId, ContextId, Vec<ContextoidId>) {
    let n: Vec<ContextoidId> = block_on(storage.reserve(3)).unwrap().collect();
    block_on(storage.create_node(&[
        ContextoidRecord::new(n[0], NodeRecord::Root),
        ContextoidRecord::new(n[1], NodeRecord::Data(DataRecord::Count(5))),
        ContextoidRecord::new(n[2], NodeRecord::Data(DataRecord::Count(7))),
    ]))
    .unwrap();
    block_on(storage.create_edge(&[RelationRecord::new(n[0], n[1], RelationKind::Datial)]))
        .unwrap();
    let base = block_on(storage.create_context("base")).unwrap();
    let extra = block_on(storage.create_context("weather")).unwrap();
    block_on(storage.link(base, &n[..2])).unwrap();
    block_on(storage.link(extra, &n[2..])).unwrap();
    block_on(storage.attach(base, extra)).unwrap();
    (base, extra, n)
}

fn next_id(storage: &MemoryStorage) -> ContextoidId {
    block_on(storage.reserve(1)).unwrap().next().unwrap()
}

#[test]
fn test_a_stored_branch_hydrates_equal() {
    let storage = MemoryStorage::new();
    let (base, extra, n) = stored_world(&storage);
    let store = ContextStore::new(storage.clone());
    let mut branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    let fresh: Vec<ContextoidId> = block_on(store.reserve(2)).unwrap().collect();
    let a = branch.add_node(count(fresh[0], 11)).unwrap();
    let b = branch.add_node(count(fresh[1], 12)).unwrap();
    branch.add_edge(a, b, RelationKind::Spatial).unwrap();
    let before = branch.snapshot().unwrap();

    let stored = block_on(store.store_branch("branch", &branch)).unwrap();
    assert_ne!(stored, base);
    assert_eq!(
        branch.snapshot().unwrap(),
        before,
        "the branch is unchanged"
    );

    let again: UniformContext = block_on(store.hydrate(&stored)).unwrap();
    let after = again.snapshot().unwrap();
    assert_eq!(after.context().name(), "branch");
    assert_eq!(after.nodes(), before.nodes());
    assert_eq!(after.edges(), before.edges());
    assert_eq!(after.extras().len(), 1);
    assert_eq!(after.extras()[0].name(), "weather");
    assert_eq!(after.extras()[0].nodes(), before.extras()[0].nodes());
    assert_ne!(after.extras()[0].id(), extra, "a new container, attached");
    // The origin is untouched.
    let origin = block_on(storage.hydrate(&base)).unwrap();
    assert_eq!(origin.nodes().len(), 2);
    assert!(origin.nodes().iter().all(|r| r.id() != fresh[0]));
    let _ = n;
}

#[test]
fn test_a_changed_value_is_a_new_node() {
    let storage = MemoryStorage::new();
    let (base, _, n) = stored_world(&storage);
    let store = ContextStore::new(storage.clone());
    let mut branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    branch.update_node(n[1], count(n[1], 99)).unwrap();
    let stored = block_on(store.store_branch("changed", &branch)).unwrap();

    let held = block_on(storage.lookup(&[n[1]])).unwrap();
    assert_eq!(
        held,
        vec![Some(ContextoidRecord::new(
            n[1],
            NodeRecord::Data(DataRecord::Count(5))
        ))]
    );
    let snapshot = block_on(storage.hydrate(&stored)).unwrap();
    let changed: Vec<&ContextoidRecord> = snapshot
        .nodes()
        .iter()
        .filter(|r| r.node() == &NodeRecord::Data(DataRecord::Count(99)))
        .collect();
    assert_eq!(changed.len(), 1);
    let fresh = changed[0].id();
    assert_ne!(fresh, n[1]);
    assert!(snapshot.nodes().iter().all(|r| r.id() != n[1]));
    assert_eq!(
        snapshot.edges(),
        &[RelationRecord::new(n[0], fresh, RelationKind::Datial)]
    );
    let origin = block_on(storage.hydrate(&base)).unwrap();
    assert_eq!(
        origin.edges(),
        &[RelationRecord::new(n[0], n[1], RelationKind::Datial)]
    );
}

#[test]
fn test_a_shared_node_is_linked_not_copied() {
    let storage = MemoryStorage::new();
    let (base, _, n) = stored_world(&storage);
    let store = ContextStore::new(storage.clone());
    let branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    let before = next_id(&storage);
    let stored = block_on(store.store_branch("same", &branch)).unwrap();
    let after = next_id(&storage);
    // Two containers were created (the base and the attached extra) and no node was reserved.
    assert_eq!(after, before + 3);
    let snapshot = block_on(storage.hydrate(&stored)).unwrap();
    let ids: Vec<ContextoidId> = snapshot.nodes().iter().map(|r| r.id()).collect();
    assert_eq!(ids, vec![n[0], n[1]]);
    assert_eq!(snapshot.extras()[0].nodes()[0].id(), n[2]);
}

#[test]
fn test_extras_become_attached_containers() {
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let n: Vec<ContextoidId> = block_on(store.reserve(3)).unwrap().collect();
    let mut branch: UniformContext = Context::with_capacity(1, "built", 4);
    branch
        .add_node(Contextoid::new(n[0], ContextoidType::Root(Root::new(n[0]))))
        .unwrap();
    branch.extra_ctx_add_new("weather", 4, true);
    branch.extra_ctx_add_node(count(n[1], 1)).unwrap();
    branch.extra_ctx_add_new("terrain", 4, true);
    branch.extra_ctx_add_node(count(n[2], 2)).unwrap();
    let stored = block_on(store.store_branch("built", &branch)).unwrap();
    let snapshot = block_on(storage.hydrate(&stored)).unwrap();
    let names: Vec<&str> = snapshot.extras().iter().map(|e| e.name()).collect();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"weather") && names.contains(&"terrain"));
    for extra in snapshot.extras() {
        let own = block_on(storage.hydrate(&extra.id())).unwrap();
        assert_eq!(own.nodes(), extra.nodes());
        assert_eq!(own.context().name(), extra.name());
    }
    let again: UniformContext = block_on(store.hydrate(&stored)).unwrap();
    assert_eq!(again.snapshot().unwrap().extras().len(), 2);
}

#[test]
fn test_an_empty_branch_stores_an_empty_container() {
    let store = ContextStore::new(MemoryStorage::new());
    let branch: UniformContext = Context::with_capacity(1, "nothing", 1);
    let stored = block_on(store.store_branch("nothing", &branch)).unwrap();
    let snapshot = block_on(store.storage().hydrate(&stored)).unwrap();
    assert!(
        snapshot.nodes().is_empty() && snapshot.edges().is_empty() && snapshot.extras().is_empty()
    );
    assert_eq!(snapshot.context().name(), "nothing");
}

#[test]
fn test_an_unreserved_identifier_is_the_backends_refusal() {
    let store = ContextStore::new(MemoryStorage::new());
    let mut branch: UniformContext = Context::with_capacity(1, "stranger", 1);
    branch.add_node(count(1_000, 1)).unwrap();
    let result = block_on(store.store_branch("stranger", &branch));
    assert!(matches!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Storage(_))
    ));
}

#[test]
fn test_an_unrecordable_branch_is_refused() {
    type Clockwork = Context<Data<u64>, NoSpaceTime<f64>, TimeKind<f64>, NoSpaceTime<f64>>;
    let store = ContextStore::new(MemoryStorage::new());
    let mut branch: Clockwork = Context::with_capacity(1, "clock", 2);
    branch
        .add_node(Contextoid::new(
            4,
            ContextoidType::Spaceoid(NoSpaceTime::new()),
        ))
        .unwrap();
    let result = block_on(store.store_branch("clock", &branch));
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Projection(ProjectionError::Unrecordable(
            0,
            "NoSpaceTime"
        )))
    );
}

/// The in-memory backend with a reserve that always answers empty: a backend in breach of the
/// contract, so that the store's refusal of a short reserve is observed.
struct ShortReserve(MemoryStorage);

impl ContextStorage for ShortReserve {
    type Error = MemoryStorageError;
    type Slice = ContextId;

    fn reserve(&self, _: usize) -> impl Future<Output = Result<IdReserve, Self::Error>> + Send {
        std::future::ready(Ok(IdReserve::new(Vec::new())))
    }
    fn create_context(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<ContextId, Self::Error>> + Send {
        self.0.create_context(name)
    }
    fn retract_context(
        &self,
        id: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.retract_context(id)
    }
    fn create_node(
        &self,
        nodes: &[ContextoidRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.create_node(nodes)
    }
    fn retract_node(
        &self,
        id: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.retract_node(id)
    }
    fn create_edge(
        &self,
        edges: &[RelationRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.create_edge(edges)
    }
    fn retract_edge(
        &self,
        from: ContextoidId,
        to: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.retract_edge(from, to)
    }
    fn link(
        &self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.link(context, nodes)
    }
    fn unlink(
        &self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.unlink(context, nodes)
    }
    fn attach(
        &self,
        context: ContextId,
        extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.attach(context, extra)
    }
    fn detach(
        &self,
        context: ContextId,
        extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.0.detach(context, extra)
    }
    fn lookup(
        &self,
        nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<Vec<Option<ContextoidRecord>>, Self::Error>> + Send {
        self.0.lookup(nodes)
    }
    fn hydrate(
        &self,
        spec: &Self::Slice,
    ) -> impl Future<Output = Result<ContextSnapshot, Self::Error>> + Send {
        self.0.hydrate(spec)
    }
}

#[test]
fn test_a_short_reserve_is_refused() {
    let storage = MemoryStorage::new();
    let (base, _, n) = stored_world(&storage);
    let store = ContextStore::new(ShortReserve(storage));
    let mut branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    branch.update_node(n[1], count(n[1], 99)).unwrap();
    let result = block_on(store.store_branch("short", &branch));
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Projection(ProjectionError::Identity(
            n[1],
            "the backend's reserve held fewer identifiers than asked"
        )))
    );
}
