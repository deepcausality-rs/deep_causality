/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The thirteen operations and every invariant of `context-storage-contract`, against the
//! in-memory backend. Expected values are the records handed to the operations, read back through
//! `hydrate` and `lookup`; identifiers come from `reserve` and `create_context` and are compared
//! by relation, never assumed.
//!
//! Corner cases (rows A to K): A an empty slice to `create_node`, `create_edge`, `link`,
//! `lookup`, and `reserve(0)`, in `test_empty_inputs`; B a container with one node in
//! `test_a_self_loop_edge`; C the same node linked from two containers,
//! `test_a_node_survives_its_container`, and a self-loop edge in `test_a_self_loop_edge`; D the
//! reference chain A-B-C where materialisation stops at one level,
//! `test_a_reference_materialises_one_level`, an edge with only one end linked,
//! `test_hydrate_holds_only_edges_among_members`, and edges observed through `retract_edge`
//! rather than through membership, `test_a_retracted_node_takes_only_its_own_edges`; E n/a; F identifier 0 refused as a node in
//! `test_an_identifier_not_from_a_reserve_is_refused`; G n/a; H n/a; I/J/K n/a.

use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContextStorage, ContextoidId, ContextoidRecord, DataRecord, MemoryStorageError, NodeRecord,
    RelationKind, RelationRecord, SpaceRecord, TimeRecord, TimeScale,
};

fn number(id: ContextoidId, value: f64) -> ContextoidRecord {
    ContextoidRecord::new(id, NodeRecord::Data(DataRecord::Number(value)))
}

fn ids(storage: &MemoryStorage, n: usize) -> Vec<ContextoidId> {
    block_on(storage.reserve(n)).unwrap().collect()
}

#[test]
fn test_reserve_hands_out_fresh_identifiers() {
    let storage = MemoryStorage::new();
    let first = ids(&storage, 3);
    let second = ids(&storage, 2);
    assert_eq!(first.len(), 3);
    assert_eq!(second.len(), 2);
    assert!(first.iter().all(|id| *id != 0));
    assert!(first.iter().all(|id| !second.contains(id)));
    let container = block_on(storage.create_context("c")).unwrap();
    assert!(container != 0);
    assert!(!first.contains(&container) && !second.contains(&container));
    let third = ids(&storage, 1);
    assert_ne!(third[0], container);
}

#[test]
fn test_the_thirteen_operations_hydrate_what_they_built() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 3);
    let root = ContextoidRecord::new(n[0], NodeRecord::Root);
    let data = number(n[1], 2.5);
    let time = ContextoidRecord::new(
        n[2],
        NodeRecord::Time(TimeRecord::Discrete {
            scale: TimeScale::Steps,
            tick: 7,
        }),
    );
    block_on(storage.create_node(&[time.clone(), data.clone(), root.clone()])).unwrap();
    let e1 = RelationRecord::new(n[0], n[1], RelationKind::Datial);
    let e2 = RelationRecord::new(n[0], n[2], RelationKind::Temporal);
    block_on(storage.create_edge(&[e2, e1])).unwrap();
    let base = block_on(storage.create_context("base")).unwrap();
    let extra = block_on(storage.create_context("weather")).unwrap();
    block_on(storage.link(base, &[n[2], n[0], n[1]])).unwrap();
    block_on(storage.link(extra, &[n[1]])).unwrap();
    block_on(storage.attach(base, extra)).unwrap();

    let snapshot = block_on(storage.hydrate(&base)).unwrap();
    assert_eq!(snapshot.context().name(), "base");
    assert_eq!(snapshot.context().id(), base);
    assert_eq!(snapshot.nodes(), &[root, data.clone(), time]);
    assert_eq!(snapshot.edges(), &[e1, e2]);
    assert_eq!(snapshot.extras().len(), 1);
    assert_eq!(snapshot.extras()[0].id(), extra);
    assert_eq!(snapshot.extras()[0].name(), "weather");
    assert_eq!(snapshot.extras()[0].nodes(), &[data]);
    assert!(snapshot.extras()[0].edges().is_empty());

    block_on(storage.detach(base, extra)).unwrap();
    block_on(storage.unlink(base, &[n[2]])).unwrap();
    block_on(storage.retract_edge(n[0], n[1])).unwrap();
    let snapshot = block_on(storage.hydrate(&base)).unwrap();
    assert_eq!(snapshot.nodes().len(), 2);
    assert!(snapshot.edges().is_empty());
    assert!(snapshot.extras().is_empty());
    block_on(storage.retract_node(n[1])).unwrap();
    block_on(storage.retract_context(extra)).unwrap();
    assert_eq!(
        block_on(storage.lookup(&[n[1], n[0]])).unwrap(),
        vec![None, Some(ContextoidRecord::new(n[0], NodeRecord::Root))]
    );
    assert_eq!(
        block_on(storage.hydrate(&extra)),
        Err(MemoryStorageError::UnknownContext(extra))
    );
}

#[test]
fn test_empty_inputs() {
    let storage = MemoryStorage::new();
    let mut none = block_on(storage.reserve(0)).unwrap();
    assert_eq!(none.remaining(), 0);
    assert_eq!(none.next(), None);
    assert_eq!(block_on(storage.create_node(&[])), Ok(()));
    assert_eq!(block_on(storage.create_edge(&[])), Ok(()));
    assert_eq!(block_on(storage.lookup(&[])), Ok(vec![]));
    let c = block_on(storage.create_context("c")).unwrap();
    assert_eq!(block_on(storage.link(c, &[])), Ok(()));
    assert_eq!(block_on(storage.unlink(c, &[])), Ok(()));
    let snapshot = block_on(storage.hydrate(&c)).unwrap();
    assert!(snapshot.nodes().is_empty() && snapshot.edges().is_empty());
}

#[test]
fn test_an_identifier_not_from_a_reserve_is_refused() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    let stranger = n[0] + 1_000;
    assert_eq!(
        block_on(storage.create_node(&[number(stranger, 1.0)])),
        Err(MemoryStorageError::IdentityNotReserved(stranger))
    );
    assert_eq!(
        block_on(storage.create_node(&[number(0, 1.0)])),
        Err(MemoryStorageError::IdentityNotReserved(0))
    );
    assert_eq!(
        block_on(storage.lookup(&[stranger, 0])),
        Ok(vec![None, None])
    );
    // A refused slice creates nothing, even the part that was reserved.
    assert_eq!(
        block_on(storage.create_node(&[number(n[0], 1.0), number(stranger, 1.0)])),
        Err(MemoryStorageError::IdentityNotReserved(stranger))
    );
    assert_eq!(block_on(storage.lookup(&[n[0]])), Ok(vec![None]));
}

#[test]
fn test_a_node_is_immutable_under_its_name() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    assert_eq!(block_on(storage.create_node(&[number(n[0], 1.0)])), Ok(()));
    assert_eq!(
        block_on(storage.create_node(&[number(n[0], 2.0)])),
        Err(MemoryStorageError::NodeConflict(n[0]))
    );
    assert_eq!(
        block_on(storage.lookup(&[n[0]])),
        Ok(vec![Some(number(n[0], 1.0))])
    );
}

#[test]
fn test_a_node_survives_its_container() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    block_on(storage.link(a, &[n[0]])).unwrap();
    block_on(storage.link(b, &[n[0]])).unwrap();
    block_on(storage.retract_context(a)).unwrap();
    assert_eq!(
        block_on(storage.lookup(&[n[0]])),
        Ok(vec![Some(number(n[0], 1.0))])
    );
    let snapshot = block_on(storage.hydrate(&b)).unwrap();
    assert_eq!(snapshot.nodes().len(), 1);
    assert_eq!(
        block_on(storage.retract_context(a)),
        Err(MemoryStorageError::UnknownContext(a))
    );
}

#[test]
fn test_a_retracted_node_leaves_every_container() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    block_on(storage.create_node(&[number(n[0], 1.0), number(n[1], 2.0)])).unwrap();
    block_on(storage.create_edge(&[RelationRecord::new(n[0], n[1], RelationKind::Spatial)]))
        .unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    block_on(storage.link(a, &n)).unwrap();
    block_on(storage.link(b, &n)).unwrap();
    block_on(storage.retract_node(n[0])).unwrap();
    for c in [a, b] {
        let snapshot = block_on(storage.hydrate(&c)).unwrap();
        assert_eq!(snapshot.nodes(), &[number(n[1], 2.0)]);
        assert!(snapshot.edges().is_empty());
    }
    assert_eq!(
        block_on(storage.retract_node(n[0])),
        Err(MemoryStorageError::UnknownNode(n[0]))
    );
    assert_eq!(
        block_on(storage.retract_edge(n[0], n[1])),
        Err(MemoryStorageError::UnknownEdge(n[0], n[1]))
    );
}

#[test]
fn test_a_relation_exists_once_between_two_nodes() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    block_on(storage.create_node(&[number(n[0], 1.0), number(n[1], 2.0)])).unwrap();
    let edge = RelationRecord::new(n[0], n[1], RelationKind::Datial);
    block_on(storage.create_edge(&[edge])).unwrap();
    assert_eq!(block_on(storage.create_edge(&[edge])), Ok(()));
    assert_eq!(
        block_on(storage.create_edge(&[RelationRecord::new(n[0], n[1], RelationKind::Spatial)])),
        Err(MemoryStorageError::EdgeConflict(n[0], n[1]))
    );
    let reverse = RelationRecord::new(n[1], n[0], RelationKind::Spatial);
    assert_eq!(block_on(storage.create_edge(&[reverse])), Ok(()));
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.link(c, &n)).unwrap();
    assert_eq!(
        block_on(storage.hydrate(&c)).unwrap().edges(),
        &[edge, reverse]
    );
    let stranger = n[1] + 1_000;
    assert_eq!(
        block_on(storage.create_edge(&[RelationRecord::new(n[0], stranger, RelationKind::Datial)])),
        Err(MemoryStorageError::UnknownNode(stranger))
    );
    assert_eq!(
        block_on(storage.create_edge(&[RelationRecord::new(stranger, n[0], RelationKind::Datial)])),
        Err(MemoryStorageError::UnknownNode(stranger))
    );
}

#[test]
fn test_a_self_loop_edge() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let edge = RelationRecord::new(n[0], n[0], RelationKind::Temporal);
    assert_eq!(block_on(storage.create_edge(&[edge])), Ok(()));
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.link(c, &n)).unwrap();
    let snapshot = block_on(storage.hydrate(&c)).unwrap();
    assert_eq!(snapshot.nodes().len(), 1);
    assert_eq!(snapshot.edges(), &[edge]);
}

#[test]
fn test_link_and_unlink() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    assert_eq!(
        block_on(storage.link(c + 1_000, &[n[0]])),
        Err(MemoryStorageError::UnknownContext(c + 1_000))
    );
    assert_eq!(
        block_on(storage.link(c, &[n[1]])),
        Err(MemoryStorageError::UnknownNode(n[1]))
    );
    block_on(storage.link(c, &[n[0]])).unwrap();
    block_on(storage.link(c, &[n[0]])).unwrap();
    assert_eq!(block_on(storage.hydrate(&c)).unwrap().nodes().len(), 1);
    assert_eq!(block_on(storage.unlink(c, &[n[1]])), Ok(()));
    assert_eq!(block_on(storage.unlink(c, &[n[0]])), Ok(()));
    assert_eq!(block_on(storage.unlink(c, &[n[0]])), Ok(()));
    assert!(block_on(storage.hydrate(&c)).unwrap().nodes().is_empty());
    assert_eq!(
        block_on(storage.unlink(c + 1_000, &[n[0]])),
        Err(MemoryStorageError::UnknownContext(c + 1_000))
    );
}

#[test]
fn test_lookup_answers_in_order() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    let space = ContextoidRecord::new(
        n[1],
        NodeRecord::Space(SpaceRecord::Ned {
            north: 1.0,
            east: 2.0,
            down: 3.0,
        }),
    );
    block_on(storage.create_node(std::slice::from_ref(&space))).unwrap();
    assert_eq!(
        block_on(storage.lookup(&[n[1], n[0], n[1]])),
        Ok(vec![Some(space.clone()), None, Some(space)])
    );
}

#[test]
fn test_a_reference_materialises_one_level() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 3);
    block_on(storage.create_node(&[number(n[0], 1.0), number(n[1], 2.0), number(n[2], 3.0)]))
        .unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.link(a, &[n[0]])).unwrap();
    block_on(storage.link(b, &[n[1], n[2]])).unwrap();
    block_on(storage.link(c, &[n[2]])).unwrap();
    block_on(storage.create_edge(&[RelationRecord::new(n[1], n[2], RelationKind::Datial)]))
        .unwrap();
    block_on(storage.attach(a, b)).unwrap();
    block_on(storage.attach(b, c)).unwrap();
    let snapshot = block_on(storage.hydrate(&a)).unwrap();
    assert_eq!(snapshot.nodes(), &[number(n[0], 1.0)]);
    assert_eq!(snapshot.extras().len(), 1);
    let extra = &snapshot.extras()[0];
    assert_eq!((extra.id(), extra.name()), (b, "b"));
    assert_eq!(extra.nodes(), &[number(n[1], 2.0), number(n[2], 3.0)]);
    assert_eq!(
        extra.edges(),
        &[RelationRecord::new(n[1], n[2], RelationKind::Datial)]
    );
    assert!(snapshot.extras().iter().all(|e| e.id() != c));
}

#[test]
fn test_references_are_many_to_many_and_may_cycle() {
    let storage = MemoryStorage::new();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.attach(a, c)).unwrap();
    block_on(storage.attach(b, c)).unwrap();
    block_on(storage.attach(c, a)).unwrap();
    assert_eq!(block_on(storage.attach(c, a)), Ok(()));
    let extras_of = |x| {
        block_on(storage.hydrate(&x))
            .unwrap()
            .extras()
            .iter()
            .map(|e| e.id())
            .collect::<Vec<_>>()
    };
    assert_eq!(extras_of(a), vec![c]);
    assert_eq!(extras_of(b), vec![c]);
    assert_eq!(extras_of(c), vec![a]);
    assert_eq!(
        block_on(storage.attach(a, a)),
        Err(MemoryStorageError::SelfReference(a))
    );
    assert_eq!(
        block_on(storage.attach(a, c + 1_000)),
        Err(MemoryStorageError::UnknownContext(c + 1_000))
    );
    assert_eq!(
        block_on(storage.attach(c + 1_000, a)),
        Err(MemoryStorageError::UnknownContext(c + 1_000))
    );
}

#[test]
fn test_a_retracted_container_leaves_no_dangling_reference() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    block_on(storage.link(a, &n)).unwrap();
    block_on(storage.attach(a, b)).unwrap();
    block_on(storage.retract_context(b)).unwrap();
    let snapshot = block_on(storage.hydrate(&a)).unwrap();
    assert!(snapshot.extras().is_empty());
    assert_eq!(snapshot.nodes(), &[number(n[0], 1.0)]);
    assert_eq!(block_on(storage.detach(a, b)), Ok(()));
    assert_eq!(
        block_on(storage.detach(b, a)),
        Err(MemoryStorageError::UnknownContext(b))
    );
}

#[test]
fn test_hydrate_orders_canonically_and_refuses_unknown() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 3);
    block_on(storage.create_node(&[number(n[2], 3.0), number(n[0], 1.0), number(n[1], 2.0)]))
        .unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.link(c, &[n[2], n[0], n[1]])).unwrap();
    block_on(storage.create_edge(&[
        RelationRecord::new(n[2], n[0], RelationKind::Datial),
        RelationRecord::new(n[0], n[2], RelationKind::Datial),
        RelationRecord::new(n[0], n[1], RelationKind::Datial),
    ]))
    .unwrap();
    let snapshot = block_on(storage.hydrate(&c)).unwrap();
    let node_ids: Vec<_> = snapshot.nodes().iter().map(|r| r.id()).collect();
    assert_eq!(node_ids, vec![n[0], n[1], n[2]]);
    let edge_ends: Vec<_> = snapshot
        .edges()
        .iter()
        .map(|e| (e.from(), e.to()))
        .collect();
    assert_eq!(edge_ends, vec![(n[0], n[1]), (n[0], n[2]), (n[2], n[0])]);
    assert_eq!(
        block_on(storage.hydrate(&(c + 1_000))),
        Err(MemoryStorageError::UnknownContext(c + 1_000))
    );
}

#[test]
fn test_a_container_identifier_is_never_reused() {
    let storage = MemoryStorage::new();
    let a = block_on(storage.create_context("a")).unwrap();
    block_on(storage.retract_context(a)).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    assert_ne!(a, b);
    assert!(a != 0 && b != 0);
}

#[test]
fn test_hydrate_holds_only_edges_among_members() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    block_on(storage.create_node(&[number(n[0], 1.0), number(n[1], 2.0)])).unwrap();
    block_on(storage.create_edge(&[
        RelationRecord::new(n[0], n[1], RelationKind::Datial),
        RelationRecord::new(n[1], n[0], RelationKind::Spatial),
    ]))
    .unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.link(c, &[n[0]])).unwrap();
    let snapshot = block_on(storage.hydrate(&c)).unwrap();
    assert_eq!(snapshot.nodes(), &[number(n[0], 1.0)]);
    assert!(snapshot.edges().is_empty());
    block_on(storage.link(c, &[n[1]])).unwrap();
    assert_eq!(block_on(storage.hydrate(&c)).unwrap().edges().len(), 2);
}

#[test]
fn test_a_retracted_node_takes_only_its_own_edges() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 4);
    block_on(storage.create_node(&[
        number(n[0], 1.0),
        number(n[1], 2.0),
        number(n[2], 3.0),
        number(n[3], 4.0),
    ]))
    .unwrap();
    block_on(storage.create_edge(&[
        RelationRecord::new(n[0], n[1], RelationKind::Datial),
        RelationRecord::new(n[1], n[0], RelationKind::Datial),
        RelationRecord::new(n[2], n[3], RelationKind::Datial),
    ]))
    .unwrap();
    // Retracting the target of one edge and the source of another takes both, and only both.
    block_on(storage.retract_node(n[1])).unwrap();
    assert_eq!(
        block_on(storage.retract_edge(n[0], n[1])),
        Err(MemoryStorageError::UnknownEdge(n[0], n[1]))
    );
    assert_eq!(
        block_on(storage.retract_edge(n[1], n[0])),
        Err(MemoryStorageError::UnknownEdge(n[1], n[0]))
    );
    assert_eq!(block_on(storage.retract_edge(n[2], n[3])), Ok(()));
}
