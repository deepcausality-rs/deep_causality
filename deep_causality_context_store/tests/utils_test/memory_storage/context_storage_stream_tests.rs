/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `subscribe`, `apply` and `apply_batch` on the in-memory backend, and the fixed scope of a
//! subscription. Expected events are the operations performed, in order; expected cursors are
//! log positions, which grow by exactly the events each operation emits.
//!
//! Corner cases (rows A to K): A an empty batch in `test_apply_batch_is_atomic`, a subscription
//! before any event in `test_a_subscription_resumes_from_a_cursor`; B one event after
//! subscribing in `test_no_change_falls_between_snapshot_and_stream`; C a cursor exactly at the
//! end of the log, `test_a_subscription_resumes_from_a_cursor`; D the cursor one past the end,
//! refused, `test_a_cursor_past_the_log_is_refused`; F cursor 0 in
//! `test_a_subscription_resumes_from_a_cursor`; every other row n/a.

use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContainerRef, ContextEvent, ContextEvents, ContextRecord, ContextStorage, ContextStorageStream,
    ContextWrite, ContextoidId, ContextoidRecord, DataRecord, MemoryStorageError, NodeRecord,
    RelationKind, RelationRecord,
};

fn number(id: ContextoidId, value: f64) -> ContextoidRecord {
    ContextoidRecord::new(id, NodeRecord::Data(DataRecord::Number(value)))
}

fn ids(storage: &MemoryStorage, n: usize) -> Vec<ContextoidId> {
    block_on(storage.reserve(n)).unwrap().collect()
}

fn drain<E: ContextEvents>(events: &mut E) -> Vec<(E::Cursor, ContextEvent)> {
    let mut out = Vec::new();
    while let Some(item) = block_on(events.next()) {
        out.push(item.unwrap());
    }
    out
}

#[test]
fn test_no_change_falls_between_snapshot_and_stream() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    let (snapshot, mut events) = block_on(storage.subscribe(&c, None)).unwrap();
    assert!(snapshot.nodes().is_empty());
    block_on(storage.link(c, &[n[0]])).unwrap();
    let delivered = drain(&mut events);
    assert_eq!(delivered.len(), 1);
    assert_eq!(
        delivered[0].1,
        ContextEvent::NodeLinked {
            context: c,
            node: number(n[0], 1.0),
            edges: vec![],
        }
    );
    assert_eq!(drain(&mut events), vec![]);
}

#[test]
fn test_every_operation_emits_its_event() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    let c = block_on(storage.create_context("c")).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&c, None)).unwrap();
    block_on(storage.create_node(&[number(n[0], 1.0), number(n[1], 2.0)])).unwrap();
    let edge = RelationRecord::new(n[0], n[1], RelationKind::Datial);
    block_on(storage.create_edge(&[edge])).unwrap();
    block_on(storage.link(c, &n)).unwrap();
    let d = block_on(storage.create_context("d")).unwrap();
    block_on(storage.attach(c, d)).unwrap();
    block_on(storage.detach(c, d)).unwrap();
    block_on(storage.unlink(c, &[n[1]])).unwrap();
    block_on(storage.retract_edge(n[0], n[1])).unwrap();
    block_on(storage.retract_node(n[0])).unwrap();
    block_on(storage.retract_context(c)).unwrap();
    let delivered: Vec<ContextEvent> = drain(&mut events).into_iter().map(|(_, e)| e).collect();
    assert_eq!(
        delivered,
        vec![
            ContextEvent::NodeCreated(number(n[0], 1.0)),
            ContextEvent::NodeCreated(number(n[1], 2.0)),
            ContextEvent::EdgeCreated(edge),
            ContextEvent::NodeLinked {
                context: c,
                node: number(n[0], 1.0),
                edges: vec![],
            },
            ContextEvent::NodeLinked {
                context: c,
                node: number(n[1], 2.0),
                edges: vec![edge],
            },
            ContextEvent::ContextAttached {
                context: c,
                extra: ContextRecord::new(d, "d".to_string())
            },
            ContextEvent::ContextDetached {
                context: c,
                extra: d
            },
            ContextEvent::NodeUnlinked {
                context: c,
                node: n[1]
            },
            ContextEvent::EdgeRetracted {
                from: n[0],
                to: n[1]
            },
            ContextEvent::NodeRetracted(n[0]),
            ContextEvent::ContextRetracted(c),
        ]
    );
}

#[test]
fn test_idempotent_operations_emit_nothing() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    let d = block_on(storage.create_context("d")).unwrap();
    block_on(storage.link(c, &n)).unwrap();
    block_on(storage.attach(c, d)).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&c, None)).unwrap();
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    block_on(storage.link(c, &n)).unwrap();
    block_on(storage.attach(c, d)).unwrap();
    block_on(storage.unlink(c, &[n[0] + 1_000])).unwrap();
    block_on(storage.detach(c, d + 1_000)).unwrap();
    assert_eq!(drain(&mut events), vec![]);
}

#[test]
fn test_cursors_are_log_positions() {
    let storage = MemoryStorage::new();
    let c = block_on(storage.create_context("c")).unwrap();
    let n = ids(&storage, 2);
    let (_, mut events) = block_on(storage.subscribe(&c, None)).unwrap();
    let cursor = block_on(storage.apply(&ContextEvent::NodeCreated(number(n[0], 1.0)))).unwrap();
    let after = block_on(storage.apply(&ContextEvent::NodeCreated(number(n[1], 2.0)))).unwrap();
    assert_eq!(after, cursor + 1);
    let delivered = drain(&mut events);
    assert_eq!(delivered[0].0, cursor);
    assert_eq!(delivered[1].0, after);
}

#[test]
fn test_apply_performs_the_operation_the_event_names() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    let c = block_on(storage.create_context("c")).unwrap();
    let d = block_on(storage.create_context("d")).unwrap();
    let apply = |e: ContextEvent| block_on(storage.apply(&e));
    apply(ContextEvent::NodeCreated(number(n[0], 1.0))).unwrap();
    apply(ContextEvent::NodeCreated(number(n[1], 2.0))).unwrap();
    let edge = RelationRecord::new(n[0], n[1], RelationKind::Spatial);
    apply(ContextEvent::EdgeCreated(edge)).unwrap();
    apply(ContextEvent::NodeLinked {
        context: c,
        node: number(n[0], 1.0),
        edges: vec![],
    })
    .unwrap();
    apply(ContextEvent::NodeLinked {
        context: c,
        node: number(n[1], 2.0),
        edges: vec![RelationRecord::new(n[1], n[0], RelationKind::Temporal)],
    })
    .unwrap();
    apply(ContextEvent::ContextAttached {
        context: c,
        extra: ContextRecord::new(d, "d".to_string()),
    })
    .unwrap();
    let snapshot = block_on(storage.hydrate(&c)).unwrap();
    assert_eq!(snapshot.nodes().len(), 2);
    assert_eq!(snapshot.edges(), &[edge]);
    assert_eq!(snapshot.extras()[0].id(), d);

    apply(ContextEvent::ContextDetached {
        context: c,
        extra: d,
    })
    .unwrap();
    apply(ContextEvent::NodeUnlinked {
        context: c,
        node: n[1],
    })
    .unwrap();
    apply(ContextEvent::EdgeRetracted {
        from: n[0],
        to: n[1],
    })
    .unwrap();
    apply(ContextEvent::NodeRetracted(n[1])).unwrap();
    let snapshot = block_on(storage.hydrate(&c)).unwrap();
    assert_eq!(snapshot.nodes(), &[number(n[0], 1.0)]);
    assert!(snapshot.edges().is_empty() && snapshot.extras().is_empty());
    apply(ContextEvent::ContextRetracted(c)).unwrap();
    assert_eq!(
        block_on(storage.hydrate(&c)),
        Err(MemoryStorageError::UnknownContext(c))
    );
}

#[test]
fn test_apply_keeps_every_refusal() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    let c = block_on(storage.create_context("c")).unwrap();
    let d = block_on(storage.create_context("d")).unwrap();
    let apply = |e: ContextEvent| block_on(storage.apply(&e));
    assert_eq!(
        apply(ContextEvent::NodeCreated(number(n[0] + 1_000, 1.0))),
        Err(MemoryStorageError::IdentityNotReserved(n[0] + 1_000))
    );
    assert_eq!(
        apply(ContextEvent::NodeLinked {
            context: c,
            node: number(n[0], 1.0),
            edges: vec![],
        }),
        Err(MemoryStorageError::UnknownNode(n[0]))
    );
    apply(ContextEvent::NodeCreated(number(n[0], 1.0))).unwrap();
    assert_eq!(
        apply(ContextEvent::NodeLinked {
            context: c,
            node: number(n[0], 9.0),
            edges: vec![],
        }),
        Err(MemoryStorageError::NodeConflict(n[0]))
    );
    assert_eq!(
        apply(ContextEvent::ContextCreated(ContextRecord::new(
            99,
            "x".to_string()
        ))),
        Err(MemoryStorageError::EventNotApplicable("ContextCreated"))
    );
    assert_eq!(
        apply(ContextEvent::NodeEntered {
            context: c,
            node: number(n[0], 1.0),
            edges: vec![],
        }),
        Err(MemoryStorageError::EventNotApplicable("NodeEntered"))
    );
    assert_eq!(
        apply(ContextEvent::NodeLeft {
            context: c,
            node: n[0]
        }),
        Err(MemoryStorageError::EventNotApplicable("NodeLeft"))
    );
    assert_eq!(
        apply(ContextEvent::ContextAttached {
            context: c,
            extra: ContextRecord::new(d, "not d".to_string())
        }),
        Err(MemoryStorageError::ContextConflict(d))
    );
    assert_eq!(
        apply(ContextEvent::ContextAttached {
            context: c,
            extra: ContextRecord::new(c, "c".to_string())
        }),
        Err(MemoryStorageError::SelfReference(c))
    );
}

#[test]
fn test_apply_batch_is_atomic() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    let c = block_on(storage.create_context("c")).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&c, None)).unwrap();
    let before = block_on(storage.apply_batch(&[])).unwrap();
    let batch = [
        ContextEvent::NodeCreated(number(n[0], 1.0)),
        ContextEvent::NodeLinked {
            context: c,
            node: number(n[0], 1.0),
            edges: vec![],
        },
        ContextEvent::NodeCreated(number(n[0] + 1_000, 2.0)),
    ];
    assert_eq!(
        block_on(storage.apply_batch(&batch)),
        Err(MemoryStorageError::IdentityNotReserved(n[0] + 1_000))
    );
    assert_eq!(block_on(storage.lookup(&[n[0]])), Ok(vec![None]));
    assert!(block_on(storage.hydrate(&c)).unwrap().nodes().is_empty());
    assert_eq!(drain(&mut events), vec![]);
    let after = block_on(storage.apply_batch(&batch[..2])).unwrap();
    assert_eq!(after, before + 2);
    assert_eq!(block_on(storage.hydrate(&c)).unwrap().nodes().len(), 1);
    assert_eq!(drain(&mut events).len(), 2);
}

#[test]
fn test_a_subscription_resumes_from_a_cursor() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    let c = block_on(storage.create_context("c")).unwrap();
    assert_eq!(
        block_on(storage.subscribe(&c, Some(0))).map(|_| ()),
        Err(MemoryStorageError::UnknownContext(c))
    );
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    block_on(storage.link(c, &[n[0]])).unwrap();
    let (snapshot, mut events) = block_on(storage.subscribe(&c, None)).unwrap();
    assert_eq!(snapshot.nodes().len(), 1);
    let cursor = block_on(storage.apply(&ContextEvent::NodeCreated(number(n[1], 2.0)))).unwrap();
    let (delivered_cursor, _) = drain(&mut events).swap_remove(0);
    assert_eq!(delivered_cursor, cursor);
    drop(events);
    block_on(storage.link(c, &[n[1]])).unwrap();
    block_on(storage.unlink(c, &[n[0]])).unwrap();
    let (as_of, mut resumed) = block_on(storage.subscribe(&c, Some(cursor))).unwrap();
    assert_eq!(as_of.nodes(), &[number(n[0], 1.0)]);
    let delivered: Vec<ContextEvent> = drain(&mut resumed).into_iter().map(|(_, e)| e).collect();
    assert_eq!(
        delivered,
        vec![
            ContextEvent::NodeLinked {
                context: c,
                node: number(n[1], 2.0),
                edges: vec![],
            },
            ContextEvent::NodeUnlinked {
                context: c,
                node: n[0]
            },
        ]
    );
    let (now, mut at_end) = block_on(storage.subscribe(&c, Some(cursor + 2))).unwrap();
    assert_eq!(now, block_on(storage.hydrate(&c)).unwrap());
    assert_eq!(drain(&mut at_end), vec![]);
}

#[test]
fn test_a_cursor_past_the_log_is_refused() {
    let storage = MemoryStorage::new();
    let c = block_on(storage.create_context("c")).unwrap();
    let end = block_on(storage.apply_batch(&[])).unwrap();
    assert_eq!(
        block_on(storage.subscribe(&c, Some(end + 1))).map(|_| ()),
        Err(MemoryStorageError::UnknownCursor(end + 1))
    );
    assert_eq!(
        block_on(storage.subscribe(&(c + 1_000), None)).map(|_| ()),
        Err(MemoryStorageError::UnknownContext(c + 1_000))
    );
}

#[test]
fn test_a_new_container_is_outside_an_existing_subscription() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&a, None)).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    block_on(storage.link(b, &n)).unwrap();
    let delivered = drain(&mut events);
    assert!(
        delivered
            .iter()
            .all(|(_, e)| !matches!(e, ContextEvent::NodeLinked { context, .. } if *context == b))
    );
    assert!(
        delivered
            .iter()
            .all(|(_, e)| !matches!(e, ContextEvent::ContextCreated(_)))
    );
    let (snapshot, mut on_b) = block_on(storage.subscribe(&b, None)).unwrap();
    assert_eq!(snapshot.nodes(), &[number(n[0], 1.0)]);
    block_on(storage.unlink(b, &n)).unwrap();
    assert_eq!(drain(&mut on_b).len(), 1);
}

#[test]
fn test_an_attachment_is_delivered_and_materialised_by_resubscribing() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 1);
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.link(c, &n)).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&a, None)).unwrap();
    block_on(storage.attach(a, c)).unwrap();
    let delivered = drain(&mut events);
    assert_eq!(delivered.len(), 1);
    assert_eq!(
        delivered[0].1,
        ContextEvent::ContextAttached {
            context: a,
            extra: ContextRecord::new(c, "c".to_string())
        }
    );
    // Membership of c was outside the first subscription's scope; a new subscription holds it.
    block_on(storage.unlink(c, &n)).unwrap();
    assert_eq!(drain(&mut events), vec![]);
    let (snapshot, mut wider) = block_on(storage.subscribe(&a, Some(delivered[0].0))).unwrap();
    assert_eq!(snapshot.extras()[0].id(), c);
    assert_eq!(snapshot.extras()[0].nodes(), &[number(n[0], 1.0)]);
    let later: Vec<ContextEvent> = drain(&mut wider).into_iter().map(|(_, e)| e).collect();
    assert_eq!(
        later,
        vec![ContextEvent::NodeUnlinked {
            context: c,
            node: n[0]
        }]
    );
}

#[test]
fn test_a_link_carries_the_relations_to_members() {
    // Nodes a < b < c < d. Edges in both directions, a self-loop, and one to d, which is never
    // linked. b is linked first; c and a arrive in one call, c before a.
    let storage = MemoryStorage::new();
    let n = ids(&storage, 4);
    let (a, b, c, d) = (n[0], n[1], n[2], n[3]);
    block_on(storage.create_node(&[
        number(a, 1.0),
        number(b, 2.0),
        number(c, 3.0),
        number(d, 4.0),
    ]))
    .unwrap();
    let a_b = RelationRecord::new(a, b, RelationKind::Datial);
    let c_a = RelationRecord::new(c, a, RelationKind::Spatial);
    let a_a = RelationRecord::new(a, a, RelationKind::Temporal);
    let a_d = RelationRecord::new(a, d, RelationKind::SpaceTemporal);
    block_on(storage.create_edge(&[c_a, a_d, a_b, a_a])).unwrap();
    let ctx = block_on(storage.create_context("c")).unwrap();
    block_on(storage.link(ctx, &[b])).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&ctx, None)).unwrap();
    block_on(storage.link(ctx, &[c, a])).unwrap();
    let linked_a = ContextEvent::NodeLinked {
        context: ctx,
        node: number(a, 1.0),
        edges: vec![a_a, a_b, c_a],
    };
    let delivered: Vec<ContextEvent> = drain(&mut events).into_iter().map(|(_, e)| e).collect();
    assert_eq!(
        delivered,
        vec![
            ContextEvent::NodeLinked {
                context: ctx,
                node: number(c, 3.0),
                edges: vec![],
            },
            linked_a.clone(),
        ]
    );
    // Relinked after an unlink, a carries the same relations again.
    block_on(storage.unlink(ctx, &[a])).unwrap();
    block_on(storage.link(ctx, &[a])).unwrap();
    let delivered: Vec<ContextEvent> = drain(&mut events).into_iter().map(|(_, e)| e).collect();
    assert_eq!(
        delivered,
        vec![
            ContextEvent::NodeUnlinked {
                context: ctx,
                node: a
            },
            linked_a,
        ]
    );
}

#[test]
fn test_a_committed_link_carries_the_relations_to_members() {
    let storage = MemoryStorage::new();
    let n = ids(&storage, 2);
    let c = block_on(storage.create_context("c")).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&c, None)).unwrap();
    let edge = RelationRecord::new(n[1], n[0], RelationKind::Datial);
    let writes = [
        ContextWrite::CreateNode(vec![number(n[0], 1.0), number(n[1], 2.0)]),
        ContextWrite::CreateEdge(vec![edge]),
        ContextWrite::Link {
            context: ContainerRef::Held(c),
            nodes: n.clone(),
        },
    ];
    block_on(storage.commit(&writes)).unwrap();
    let delivered: Vec<ContextEvent> = drain(&mut events).into_iter().map(|(_, e)| e).collect();
    assert_eq!(
        delivered[3..],
        [
            ContextEvent::NodeLinked {
                context: c,
                node: number(n[0], 1.0),
                edges: vec![],
            },
            ContextEvent::NodeLinked {
                context: c,
                node: number(n[1], 2.0),
                edges: vec![edge],
            },
        ]
    );
}
