/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Context::apply`: one event applied idempotently, routed by the container it names. The
//! context is restored from a snapshot with base identifier 7 and extras 40 and 41, so every
//! identifier in an event is a literal chosen here and every effect is read back through the
//! canonical snapshot.
//!
//! Corner cases (rows A to K): C an echo of a state already held, `test_an_echo_is_harmless`;
//! a container this context does not hold, `test_an_unknown_container_is_refused`; F an attached
//! container under identifier 0, `test_an_attached_container_under_zero_is_refused`; a node
//! held by two of three graphs, `test_an_edge_lands_wherever_both_ends_are_held`; every other
//! row n/a.

use deep_causality_context::{
    ContextStore, Contextoid, ContextoidType, ContextuableGraph, Data, ExtendableContextuableGraph,
    UniformContext,
};
use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContextEvent, ContextEvents, ContextRecord, ContextSnapshot, ContextStorage, ContextoidId,
    ContextoidRecord, DataRecord, ExtraContextSnapshot, NodeRecord, ProjectionError, RelationKind,
    RelationRecord,
};
use deep_causality_core::Identifiable;

fn count(id: ContextoidId, value: u64) -> ContextoidRecord {
    ContextoidRecord::new(id, NodeRecord::Data(DataRecord::Count(value)))
}

/// Base 7 holds nodes 3 and 4 with an edge 3-4; extra 40 holds 3 and 4 and, as any container
/// holding both ends of a stored edge does, the edge; extra 41 holds 3.
fn world() -> UniformContext {
    let snapshot = ContextSnapshot::new(
        ContextRecord::new(7, "base".to_string()),
        vec![count(3, 30), count(4, 40)],
        vec![RelationRecord::new(3, 4, RelationKind::Datial)],
        vec![
            ExtraContextSnapshot::new(
                40,
                "weather".to_string(),
                vec![count(3, 30), count(4, 40)],
                vec![RelationRecord::new(3, 4, RelationKind::Datial)],
            ),
            ExtraContextSnapshot::new(41, "terrain".to_string(), vec![count(3, 30)], vec![]),
        ],
    );
    UniformContext::restore(snapshot).unwrap()
}

fn extra_nodes(ctx: &UniformContext, id: u64) -> Vec<ContextoidId> {
    ctx.snapshot()
        .unwrap()
        .extras()
        .iter()
        .find(|e| e.id() == id)
        .map(|e| e.nodes().iter().map(|n| n.id()).collect())
        .unwrap_or_default()
}

fn extra_edges(ctx: &UniformContext, id: u64) -> usize {
    ctx.snapshot()
        .unwrap()
        .extras()
        .iter()
        .find(|e| e.id() == id)
        .map(|e| e.edges().len())
        .unwrap_or_default()
}

#[test]
fn test_an_echo_is_harmless() {
    let mut ctx = world();
    let before = ctx.snapshot().unwrap();
    ctx.apply(&ContextEvent::NodeLinked {
        context: 7,
        node: count(3, 30),
        edges: vec![],
    })
    .unwrap();
    ctx.apply(&ContextEvent::EdgeCreated(RelationRecord::new(
        3,
        4,
        RelationKind::Datial,
    )))
    .unwrap();
    ctx.apply(&ContextEvent::NodeCreated(count(9, 9))).unwrap();
    ctx.apply(&ContextEvent::ContextCreated(ContextRecord::new(
        99,
        "x".to_string(),
    )))
    .unwrap();
    assert_eq!(ctx.snapshot().unwrap(), before);
}

#[test]
fn test_a_membership_event_is_self_contained() {
    let mut ctx = world();
    ctx.apply(&ContextEvent::NodeLinked {
        context: 7,
        node: count(5, 50),
        edges: vec![],
    })
    .unwrap();
    ctx.apply(&ContextEvent::NodeEntered {
        context: 41,
        node: count(6, 60),
        edges: vec![],
    })
    .unwrap();
    let snapshot = ctx.snapshot().unwrap();
    assert!(snapshot.nodes().iter().any(|n| n.id() == 5));
    assert_eq!(extra_nodes(&ctx, 41), vec![3, 6]);
    // The base graph's identifier index follows the event, so the node is reachable by id.
    let index = ctx.get_node_index_by_id(5).unwrap();
    assert_eq!(ctx.get_node(index).unwrap().id(), 5);
    ctx.remove_node(5).unwrap();
    assert_eq!(ctx.get_node_index_by_id(5), None);
}

#[test]
fn test_unlink_left_and_retract() {
    let mut ctx = world();
    ctx.apply(&ContextEvent::NodeUnlinked {
        context: 7,
        node: 4,
    })
    .unwrap();
    assert_eq!(
        ctx.get_node_index_by_id(4),
        None,
        "the base index forgets the node at once"
    );
    assert!(ctx.get_node_index_by_id(3).is_some());
    // Leaving an extra touches that extra alone: the base still indexes 3.
    ctx.apply(&ContextEvent::NodeLeft {
        context: 41,
        node: 3,
    })
    .unwrap();
    assert!(ctx.get_node_index_by_id(3).is_some());
    assert!(extra_nodes(&ctx, 41).is_empty());
    ctx.apply(&ContextEvent::NodeLeft {
        context: 40,
        node: 4,
    })
    .unwrap();
    ctx.apply(&ContextEvent::NodeUnlinked {
        context: 7,
        node: 4,
    })
    .unwrap();
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(
        snapshot.nodes().iter().map(|n| n.id()).collect::<Vec<_>>(),
        vec![3]
    );
    assert_eq!(
        ctx.get_node_index_by_id(4),
        None,
        "the index forgets an unlinked node"
    );
    assert!(snapshot.edges().is_empty());
    assert_eq!(extra_nodes(&ctx, 40), vec![3]);
    assert_eq!(extra_edges(&ctx, 40), 0);
    ctx.apply(&ContextEvent::NodeRetracted(3)).unwrap();
    ctx.apply(&ContextEvent::NodeRetracted(3)).unwrap();
    assert!(ctx.snapshot().unwrap().nodes().is_empty());
    assert!(extra_nodes(&ctx, 40).is_empty());
}

#[test]
fn test_an_edge_lands_wherever_both_ends_are_held() {
    let mut ctx = world();
    ctx.apply(&ContextEvent::EdgeCreated(RelationRecord::new(
        4,
        3,
        RelationKind::Spatial,
    )))
    .unwrap();
    let snapshot = ctx.snapshot().unwrap();
    assert_eq!(snapshot.edges().len(), 2);
    assert_eq!(extra_edges(&ctx, 40), 2);
    assert_eq!(extra_edges(&ctx, 41), 0);
    ctx.apply(&ContextEvent::EdgeRetracted { from: 4, to: 3 })
        .unwrap();
    ctx.apply(&ContextEvent::EdgeRetracted { from: 4, to: 3 })
        .unwrap();
    assert_eq!(ctx.snapshot().unwrap().edges().len(), 1);
    assert_eq!(extra_edges(&ctx, 40), 1);
}

#[test]
fn test_attach_and_detach_on_the_held_context() {
    let mut ctx = world();
    let attached = ContextEvent::ContextAttached {
        context: 7,
        extra: ContextRecord::new(42, "sea".to_string()),
    };
    ctx.apply(&attached).unwrap();
    ctx.apply(&attached).unwrap();
    assert_eq!(ctx.extra_ctx_get_name(42), Some("sea"));
    assert!(extra_nodes(&ctx, 42).is_empty());
    ctx.apply(&ContextEvent::ContextAttached {
        context: 40,
        extra: ContextRecord::new(43, "x".to_string()),
    })
    .unwrap();
    assert_eq!(ctx.extra_ctx_get_name(43), None);
    ctx.extra_ctx_set_current_id(42).unwrap();
    ctx.apply(&ContextEvent::ContextDetached {
        context: 7,
        extra: 42,
    })
    .unwrap();
    assert_eq!(ctx.extra_ctx_get_name(42), None);
    assert_eq!(ctx.extra_ctx_get_current_id(), 0);
    ctx.apply(&ContextEvent::ContextDetached {
        context: 7,
        extra: 42,
    })
    .unwrap();
    ctx.apply(&ContextEvent::ContextDetached {
        context: 40,
        extra: 41,
    })
    .unwrap();
    assert_eq!(ctx.extra_ctx_get_name(41), Some("terrain"));
}

#[test]
fn test_context_retracted() {
    let mut ctx = world();
    assert_eq!(
        ctx.apply(&ContextEvent::ContextRetracted(7)),
        Err(ProjectionError::Identity(
            7,
            "the held context cannot retract itself"
        ))
    );
    ctx.apply(&ContextEvent::ContextRetracted(40)).unwrap();
    assert_eq!(ctx.extra_ctx_get_name(40), None);
    ctx.apply(&ContextEvent::ContextRetracted(999)).unwrap();
    assert_eq!(ctx.snapshot().unwrap().extras().len(), 1);
}

#[test]
fn test_an_unknown_container_is_refused() {
    let mut ctx = world();
    assert_eq!(
        ctx.apply(&ContextEvent::NodeLinked {
            context: 999,
            node: count(5, 50),
            edges: vec![],
        }),
        Err(ProjectionError::Identity(
            999,
            "an event names a container this context does not hold"
        ))
    );
    assert_eq!(
        ctx.apply(&ContextEvent::NodeUnlinked {
            context: 999,
            node: 3
        }),
        Err(ProjectionError::Identity(
            999,
            "an event names a container this context does not hold"
        ))
    );
}

#[test]
fn test_an_attached_container_under_zero_is_refused() {
    let mut ctx = world();
    assert_eq!(
        ctx.apply(&ContextEvent::ContextAttached {
            context: 7,
            extra: ContextRecord::new(0, "none".to_string())
        }),
        Err(ProjectionError::Identity(
            0,
            "an extra context identifier is 0"
        ))
    );
}

#[test]
fn test_a_payload_the_type_cannot_hold_is_refused() {
    let mut ctx = world();
    let wrong = ContextoidRecord::new(8, NodeRecord::Data(DataRecord::Number(1.0)));
    assert_eq!(
        ctx.apply(&ContextEvent::NodeLinked {
            context: 7,
            node: wrong,
            edges: vec![],
        }),
        Err(ProjectionError::WrongPayload(8, "Count", "Number"))
    );
}

#[test]
fn test_a_dropped_extra_identifier_is_never_reallocated() {
    // Extras 40 and 41 are held; dropping 41 must not let the allocator hand 41 out again, or a
    // later ContextAttached { 7, 41 } echo would keep a local graph under the store's identifier.
    let mut ctx = world();
    ctx.apply(&ContextEvent::ContextRetracted(41)).unwrap();
    assert_eq!(ctx.extra_ctx_get_name(41), None);
    assert_eq!(ctx.extra_ctx_add_new("local", 1, false), 42);
    // An identifier that arrives through an attachment raises the mark as well.
    ctx.apply(&ContextEvent::ContextAttached {
        context: 7,
        extra: ContextRecord::new(50, "sea".to_string()),
    })
    .unwrap();
    ctx.apply(&ContextEvent::ContextDetached {
        context: 7,
        extra: 50,
    })
    .unwrap();
    assert_eq!(ctx.extra_ctx_add_new("local2", 1, false), 51);
}

#[test]
fn test_a_carried_edge_lands_with_its_node() {
    // Node 5 enters base 7, which holds 3 and 4, carrying an edge to each and one to 9, which no
    // graph holds. Extra 40 also holds 3 and 4 and is not named, so it gains nothing.
    let mut ctx = world();
    ctx.apply(&ContextEvent::NodeLinked {
        context: 7,
        node: count(5, 50),
        edges: vec![
            RelationRecord::new(4, 5, RelationKind::Temporal),
            RelationRecord::new(5, 3, RelationKind::Spatial),
            RelationRecord::new(5, 9, RelationKind::Datial),
        ],
    })
    .unwrap();
    assert_eq!(
        ctx.snapshot().unwrap().edges(),
        &[
            RelationRecord::new(3, 4, RelationKind::Datial),
            RelationRecord::new(4, 5, RelationKind::Temporal),
            RelationRecord::new(5, 3, RelationKind::Spatial),
        ]
    );
    assert_eq!(extra_edges(&ctx, 40), 1);
    // A view's entry carries edges the same way, into the extra it names.
    ctx.apply(&ContextEvent::NodeEntered {
        context: 41,
        node: count(4, 40),
        edges: vec![RelationRecord::new(3, 4, RelationKind::Datial)],
    })
    .unwrap();
    assert_eq!(extra_edges(&ctx, 41), 1);
}

/// Drains the stream into the context, then compares its snapshot with a fresh hydrate.
fn converges<E: ContextEvents>(
    ctx: &mut UniformContext,
    events: &mut E,
    storage: &MemoryStorage,
    base: u64,
) {
    while let Some(item) = block_on(events.next()) {
        ctx.apply(&item.unwrap().1).unwrap();
    }
    assert_eq!(
        ctx.snapshot().unwrap(),
        block_on(storage.hydrate(&base)).unwrap()
    );
}

#[test]
fn test_a_context_driven_by_its_stream_converges_to_the_store() {
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let base = block_on(storage.create_context("base")).unwrap();
    let n: Vec<ContextoidId> = block_on(storage.reserve(3)).unwrap().collect();
    let (a, b, c) = (n[0], n[1], n[2]);
    // c is linked, and an edge from it to b is stored, before the subscription begins.
    block_on(storage.create_node(&[count(c, 3)])).unwrap();
    block_on(storage.link(base, &[c])).unwrap();
    let (mut ctx, mut events) = block_on(store.subscribe::<_, _, _, _>(&base, None)).unwrap();
    let ctx: &mut UniformContext = &mut ctx;

    // An edge created before either end is linked.
    block_on(storage.create_node(&[count(a, 1), count(b, 2)])).unwrap();
    block_on(storage.create_edge(&[
        RelationRecord::new(a, b, RelationKind::Datial),
        RelationRecord::new(c, b, RelationKind::Spatial),
    ]))
    .unwrap();
    block_on(storage.link(base, &[a])).unwrap();
    block_on(storage.link(base, &[b])).unwrap();
    converges(ctx, &mut events, &storage, base);
    assert_eq!(ctx.snapshot().unwrap().edges().len(), 2);

    // Unlinked, a loses its edge; relinked, it has it again.
    block_on(storage.unlink(base, &[a])).unwrap();
    converges(ctx, &mut events, &storage, base);
    assert_eq!(ctx.snapshot().unwrap().edges().len(), 1);
    block_on(storage.link(base, &[a])).unwrap();
    converges(ctx, &mut events, &storage, base);
    assert_eq!(ctx.snapshot().unwrap().edges().len(), 2);
}

#[test]
fn test_a_store_container_event_never_reaches_a_local_extra() {
    // Extras 40 and 41 come from the store; 42 is allocated locally and holds node 8. The store
    // assigns container identifiers, so its container 42 is a different container.
    let mut ctx = world();
    let local = ctx.extra_ctx_add_new("local", 1, true);
    assert_eq!(local, 42);
    ctx.extra_ctx_add_node(Contextoid::new(8, ContextoidType::Datoid(Data::new(8, 80))))
        .unwrap();
    let held = |ctx: &UniformContext| {
        (
            ctx.extra_ctx_get_name(local).map(str::to_string),
            extra_nodes(ctx, local),
        )
    };
    let before = held(&ctx);
    assert_eq!(before, (Some("local".to_string()), vec![8]));

    // The store's container 42 is attached: refused, not merged into the local graph.
    assert_eq!(
        ctx.apply(&ContextEvent::ContextAttached {
            context: 7,
            extra: ContextRecord::new(local, "sea".to_string()),
        }),
        Err(ProjectionError::Identity(
            local,
            "an attached container's identifier is held by a local extra context"
        ))
    );
    // Membership naming 42 is a container this context does not hold.
    let not_held = Err(ProjectionError::Identity(
        local,
        "an event names a container this context does not hold",
    ));
    assert_eq!(
        ctx.apply(&ContextEvent::NodeLinked {
            context: local,
            node: count(5, 50),
            edges: vec![],
        }),
        not_held
    );
    assert_eq!(
        ctx.apply(&ContextEvent::NodeUnlinked {
            context: local,
            node: 8,
        }),
        not_held
    );
    // A detach or retraction of the store's 42 leaves the local extra in place.
    ctx.apply(&ContextEvent::ContextDetached {
        context: 7,
        extra: local,
    })
    .unwrap();
    ctx.apply(&ContextEvent::ContextRetracted(local)).unwrap();
    assert_eq!(held(&ctx), before);
    assert_eq!(ctx.extra_ctx_get_current_id(), local);

    // A stored extra still takes every store event.
    ctx.apply(&ContextEvent::ContextAttached {
        context: 7,
        extra: ContextRecord::new(40, "weather".to_string()),
    })
    .unwrap();
    assert_eq!(extra_nodes(&ctx, 40), vec![3, 4]);
    ctx.apply(&ContextEvent::ContextRetracted(40)).unwrap();
    assert_eq!(ctx.extra_ctx_get_name(40), None);
}

#[test]
fn test_a_node_or_edge_event_reaches_a_local_extra_holding_the_node() {
    // Node identifiers are the store's: a local extra holding nodes 3 and 4 holds the store's
    // nodes 3 and 4, so events naming them apply to it as to every other graph.
    let mut ctx = world();
    let local = ctx.extra_ctx_add_new("local", 2, true);
    for (id, value) in [(3, 30), (4, 40)] {
        ctx.extra_ctx_add_node(Contextoid::new(
            id,
            ContextoidType::Datoid(Data::new(id, value)),
        ))
        .unwrap();
    }
    assert_eq!(extra_edges(&ctx, local), 0);
    ctx.apply(&ContextEvent::EdgeCreated(RelationRecord::new(
        4,
        3,
        RelationKind::Spatial,
    )))
    .unwrap();
    assert_eq!(extra_edges(&ctx, local), 1);
    ctx.apply(&ContextEvent::EdgeRetracted { from: 4, to: 3 })
        .unwrap();
    assert_eq!(extra_edges(&ctx, local), 0);
    ctx.apply(&ContextEvent::NodeRetracted(3)).unwrap();
    assert_eq!(extra_nodes(&ctx, local), vec![4]);
    assert_eq!(extra_nodes(&ctx, 40), vec![4]);
}
