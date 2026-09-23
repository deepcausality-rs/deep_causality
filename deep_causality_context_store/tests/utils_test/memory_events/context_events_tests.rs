/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! What the stream delivers and what it withholds: every node and edge event, the membership and
//! reference events of the containers in scope, none of another container's, and `None` for
//! good at the end. Expected events are the operations performed.
//!
//! Corner cases (rows A to K): A a stream over an empty tail, `test_the_stream_ends`; C two
//! containers with the same node, one in scope and one not,
//! `test_membership_events_of_other_containers_are_withheld`; every other row n/a.

use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContextEvent, ContextEvents, ContextStorage, ContextStorageStream, ContextoidId,
    ContextoidRecord, DataRecord, NodeRecord, RelationKind, RelationRecord,
};

fn number(id: ContextoidId, value: f64) -> ContextoidRecord {
    ContextoidRecord::new(id, NodeRecord::Data(DataRecord::Number(value)))
}

fn drain<E: ContextEvents>(events: &mut E) -> Vec<ContextEvent> {
    let mut out = Vec::new();
    while let Some(item) = block_on(events.next()) {
        out.push(item.unwrap().1);
    }
    out
}

#[test]
fn test_the_stream_ends() {
    let storage = MemoryStorage::new();
    let a = block_on(storage.create_context("a")).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&a, None)).unwrap();
    assert_eq!(block_on(events.next()), None);
    assert_eq!(block_on(events.next()), None);
}

#[test]
fn test_node_and_edge_events_are_always_delivered() {
    let storage = MemoryStorage::new();
    let n: Vec<ContextoidId> = block_on(storage.reserve(2)).unwrap().collect();
    let a = block_on(storage.create_context("a")).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&a, None)).unwrap();
    block_on(storage.create_node(&[number(n[0], 1.0), number(n[1], 2.0)])).unwrap();
    let edge = RelationRecord::new(n[0], n[1], RelationKind::Datial);
    block_on(storage.create_edge(&[edge])).unwrap();
    block_on(storage.retract_edge(n[0], n[1])).unwrap();
    block_on(storage.retract_node(n[1])).unwrap();
    assert_eq!(
        drain(&mut events),
        vec![
            ContextEvent::NodeCreated(number(n[0], 1.0)),
            ContextEvent::NodeCreated(number(n[1], 2.0)),
            ContextEvent::EdgeCreated(edge),
            ContextEvent::EdgeRetracted {
                from: n[0],
                to: n[1]
            },
            ContextEvent::NodeRetracted(n[1]),
        ]
    );
}

#[test]
fn test_membership_events_of_other_containers_are_withheld() {
    let storage = MemoryStorage::new();
    let n: Vec<ContextoidId> = block_on(storage.reserve(1)).unwrap().collect();
    block_on(storage.create_node(&[number(n[0], 1.0)])).unwrap();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    let r = block_on(storage.create_context("r")).unwrap();
    block_on(storage.attach(a, r)).unwrap();
    let (_, mut events) = block_on(storage.subscribe(&a, None)).unwrap();
    block_on(storage.link(b, &n)).unwrap();
    block_on(storage.link(r, &n)).unwrap();
    block_on(storage.attach(b, r)).unwrap();
    block_on(storage.retract_context(b)).unwrap();
    block_on(storage.retract_context(r)).unwrap();
    assert_eq!(
        drain(&mut events),
        vec![
            ContextEvent::NodeLinked {
                context: r,
                node: number(n[0], 1.0),
                edges: vec![],
            },
            ContextEvent::ContextRetracted(r),
        ]
    );
}
