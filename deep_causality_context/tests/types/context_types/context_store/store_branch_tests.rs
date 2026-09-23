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
    ContextId, ContextSnapshot, ContextStorage, ContextStorageStream, ContextWrite, ContextoidId,
    ContextoidRecord, DataRecord, IdReserve, MemoryStorageError, NodeRecord, ProjectionError,
    RelationRecord,
};
use deep_causality_core::Identifiable;
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

/// How `Breaching` breaks the contract.
#[derive(Clone, Copy, PartialEq)]
enum Breach {
    /// Every reserve answers empty.
    ShortReserve,
    /// Every commit applies and reports no created container.
    EmptyCommit,
}

/// The in-memory backend in breach of the contract in one way, so that the store's refusal of the
/// breach is observed.
struct Breaching(MemoryStorage, Breach);

impl ContextStorage for Breaching {
    type Error = MemoryStorageError;
    type Slice = ContextId;

    fn reserve(&self, n: usize) -> impl Future<Output = Result<IdReserve, Self::Error>> + Send {
        let reserve = match self.1 {
            Breach::ShortReserve => Ok(IdReserve::new(Vec::new())),
            Breach::EmptyCommit => block_on(self.0.reserve(n)),
        };
        std::future::ready(reserve)
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
    fn commit(
        &self,
        writes: &[ContextWrite],
    ) -> impl Future<Output = Result<Vec<ContextId>, Self::Error>> + Send {
        let created = block_on(self.0.commit(writes));
        std::future::ready(match self.1 {
            Breach::ShortReserve => created,
            Breach::EmptyCommit => created.map(|_| Vec::new()),
        })
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
    let store = ContextStore::new(Breaching(storage, Breach::ShortReserve));
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

#[test]
fn test_a_shared_conflicting_node_keeps_one_identity() {
    // The same node, changed in place, sits in the base and in an extra. It must be re-created
    // once, under one fresh identifier, and both containers must link that identifier.
    let storage = MemoryStorage::new();
    let (base, extra, n) = stored_world(&storage);
    // Put the base's count into the extra too, so both graphs hold node n[1].
    block_on(storage.link(extra, &n[1..2])).unwrap();
    let store = ContextStore::new(storage.clone());
    let mut branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    branch.update_node(n[1], count(n[1], 99)).unwrap();
    branch.extra_ctx_set_current_id(extra).unwrap();
    let index = (0..8)
        .find(|i| {
            branch
                .extra_ctx_get_node(*i)
                .is_ok_and(|node| node.id() == n[1])
        })
        .unwrap();
    branch.extra_ctx_remove_node(index).unwrap();
    branch.extra_ctx_add_node(count(n[1], 99)).unwrap();

    let stored = block_on(store.store_branch("shared", &branch)).unwrap();
    let snapshot = block_on(storage.hydrate(&stored)).unwrap();
    let in_base: Vec<ContextoidId> = snapshot
        .nodes()
        .iter()
        .filter(|r| r.node() == &NodeRecord::Data(DataRecord::Count(99)))
        .map(|r| r.id())
        .collect();
    let in_extra: Vec<ContextoidId> = snapshot.extras()[0]
        .nodes()
        .iter()
        .filter(|r| r.node() == &NodeRecord::Data(DataRecord::Count(99)))
        .map(|r| r.id())
        .collect();
    assert_eq!(in_base.len(), 1);
    assert_eq!(in_base, in_extra, "one fresh identifier in both containers");
    assert_ne!(in_base[0], n[1]);
}

/// Hydrates the stored world with node n[1] linked into the extra as well as the base.
fn shared_branch(
    storage: &MemoryStorage,
) -> (
    ContextStore<MemoryStorage>,
    UniformContext,
    ContextId,
    Vec<ContextoidId>,
) {
    let (base, extra, n) = stored_world(storage);
    block_on(storage.link(extra, &n[1..2])).unwrap();
    let store = ContextStore::new(storage.clone());
    let branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    (store, branch, extra, n)
}

/// The identifiers a container links for one record.
fn ids_holding(records: &[ContextoidRecord], node: &NodeRecord) -> Vec<ContextoidId> {
    records
        .iter()
        .filter(|r| r.node() == node)
        .map(|r| r.id())
        .collect()
}

#[test]
fn test_a_shared_node_changed_in_one_graph_keeps_the_other_value() {
    // n[1] is shared by the base and the extra; only the base changes it. The base must link a
    // fresh node holding 99 and the extra must keep linking n[1] holding 5.
    let storage = MemoryStorage::new();
    let (store, mut branch, _, n) = shared_branch(&storage);
    branch.update_node(n[1], count(n[1], 99)).unwrap();

    let stored = block_on(store.store_branch("one-sided", &branch)).unwrap();
    let snapshot = block_on(storage.hydrate(&stored)).unwrap();
    let changed = ids_holding(snapshot.nodes(), &NodeRecord::Data(DataRecord::Count(99)));
    assert_eq!(changed.len(), 1);
    assert_ne!(changed[0], n[1]);
    assert_eq!(
        ids_holding(
            snapshot.extras()[0].nodes(),
            &NodeRecord::Data(DataRecord::Count(5))
        ),
        vec![n[1]],
        "the extra keeps the unchanged node"
    );
    assert!(
        ids_holding(
            snapshot.extras()[0].nodes(),
            &NodeRecord::Data(DataRecord::Count(99))
        )
        .is_empty()
    );
}

#[test]
fn test_two_records_under_one_new_identifier_store_two_nodes() {
    // A node the store does not hold, carried by the base and the extra with different values:
    // the base claims the identifier, the extra gets a fresh one, and each keeps its value.
    let storage = MemoryStorage::new();
    let (store, mut branch, extra, _) = shared_branch(&storage);
    let id = next_id(&storage);
    branch.add_node(count(id, 1)).unwrap();
    branch.extra_ctx_set_current_id(extra).unwrap();
    branch.extra_ctx_add_node(count(id, 2)).unwrap();

    let stored = block_on(store.store_branch("two", &branch)).unwrap();
    let snapshot = block_on(storage.hydrate(&stored)).unwrap();
    assert_eq!(
        ids_holding(snapshot.nodes(), &NodeRecord::Data(DataRecord::Count(1))),
        vec![id]
    );
    let in_extra = ids_holding(
        snapshot.extras()[0].nodes(),
        &NodeRecord::Data(DataRecord::Count(2)),
    );
    assert_eq!(in_extra.len(), 1);
    assert_ne!(in_extra[0], id);
}

#[test]
fn test_a_refused_store_writes_nothing() {
    // The branch adds a node and changes the kind of a stored edge. The edge is refused; the
    // node created before it in the same store must not remain, and no event may be emitted.
    let storage = MemoryStorage::new();
    let (base, _, n) = stored_world(&storage);
    let store = ContextStore::new(storage.clone());
    let mut branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    let added = next_id(&storage);
    branch.add_node(count(added, 3)).unwrap();
    let a = branch.get_node_index_by_id(n[0]).unwrap();
    let b = branch.get_node_index_by_id(n[1]).unwrap();
    branch.remove_edge(a, b).unwrap();
    branch.add_edge(a, b, RelationKind::Spatial).unwrap();
    let cursor = block_on(storage.apply_batch(&[])).unwrap();

    let result = block_on(store.store_branch("refused", &branch));
    assert_eq!(
        result.map_err(|e| e.0),
        Err(StoreErrorEnum::Storage(MemoryStorageError::EdgeConflict(
            n[0], n[1]
        )))
    );
    assert_eq!(block_on(storage.lookup(&[added])).unwrap(), vec![None]);
    assert_eq!(
        block_on(storage.apply_batch(&[])).unwrap(),
        cursor,
        "no event emitted"
    );
}

#[test]
fn test_a_commit_that_reports_no_container_is_refused() {
    let storage = MemoryStorage::new();
    let (base, _, _) = stored_world(&storage);
    let store = ContextStore::new(Breaching(storage, Breach::EmptyCommit));
    let branch: UniformContext = block_on(store.hydrate(&base)).unwrap();
    assert_eq!(
        block_on(store.store_branch("empty", &branch)).map_err(|e| e.0),
        Err(StoreErrorEnum::Projection(ProjectionError::Identity(
            0,
            "the backend's commit returned no created container"
        )))
    );
}
