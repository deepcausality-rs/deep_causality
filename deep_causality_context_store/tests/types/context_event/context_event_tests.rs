/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructors; no expectation is computed.
//!
//! Corner cases (rows A to K): C coinciding payloads under different variants in
//! `test_twelve_variants_all_distinct` (the same container and node identifiers appear in every
//! membership variant, and no two variants are equal); every other row n/a.

use deep_causality_context_store::{
    ContextEvent, ContextRecord, ContextoidRecord, NodeRecord, RelationKind, RelationRecord,
};

fn one_of_each() -> Vec<ContextEvent> {
    let node = ContextoidRecord::new(1, NodeRecord::Root);
    let context = ContextRecord::new(9, "base".to_string());
    vec![
        ContextEvent::ContextCreated(context.clone()),
        ContextEvent::ContextRetracted(9),
        ContextEvent::NodeCreated(node.clone()),
        ContextEvent::NodeRetracted(1),
        ContextEvent::EdgeCreated(RelationRecord::new(1, 2, RelationKind::Datial)),
        ContextEvent::EdgeRetracted { from: 1, to: 2 },
        ContextEvent::NodeLinked {
            context: 9,
            node: node.clone(),
            edges: vec![RelationRecord::new(1, 2, RelationKind::Datial)],
        },
        ContextEvent::NodeUnlinked {
            context: 9,
            node: 1,
        },
        ContextEvent::ContextAttached {
            context: 9,
            extra: context,
        },
        ContextEvent::ContextDetached {
            context: 9,
            extra: 40,
        },
        ContextEvent::NodeEntered {
            context: 9,
            node,
            edges: vec![],
        },
        ContextEvent::NodeLeft {
            context: 9,
            node: 1,
        },
    ]
}

#[test]
fn test_twelve_variants_all_distinct() {
    let all = one_of_each();
    assert_eq!(all.len(), 12);
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            assert_eq!(a == b, i == j);
        }
    }
}

#[test]
fn test_a_membership_event_carries_the_record() {
    let ContextEvent::NodeLinked {
        context,
        node,
        edges,
    } = one_of_each().swap_remove(6)
    else {
        panic!("a NodeLinked event");
    };
    assert_eq!(context, 9);
    assert_eq!(node.id(), 1);
    assert_eq!(node.node(), &NodeRecord::Root);
    assert_eq!(edges, vec![RelationRecord::new(1, 2, RelationKind::Datial)]);
}

#[test]
fn test_clone_and_debug() {
    for event in one_of_each() {
        assert_eq!(event.clone(), event);
        assert!(!format!("{event:?}").is_empty());
    }
}
