/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructor; no expectation is computed.
//!
//! Corner cases (rows A to K): A empty nodes and edges in `test_empty_extra`; B one node in
//! `test_empty_extra`; C same identifier with a different name in `test_clone_equality_and_debug`;
//! D to K n/a.

use deep_causality_context_store::{
    ContextoidRecord, ExtraContextSnapshot, NodeRecord, RelationKind, RelationRecord,
};

fn sample() -> ExtraContextSnapshot {
    ExtraContextSnapshot::new(
        40,
        "weather".to_string(),
        vec![
            ContextoidRecord::new(1, NodeRecord::Root),
            ContextoidRecord::new(2, NodeRecord::Root),
        ],
        vec![RelationRecord::new(1, 2, RelationKind::Datial)],
    )
}

#[test]
fn test_new_and_getters() {
    let extra = sample();
    assert_eq!(extra.id(), 40);
    assert_eq!(extra.name(), "weather");
    assert_eq!(extra.nodes().len(), 2);
    assert_eq!(extra.nodes()[1].id(), 2);
    assert_eq!(extra.edges().len(), 1);
    assert_eq!(extra.edges()[0].to(), 2);
}

#[test]
fn test_empty_extra() {
    let empty = ExtraContextSnapshot::new(41, "e".to_string(), vec![], vec![]);
    assert!(empty.nodes().is_empty());
    assert!(empty.edges().is_empty());
    let one = ExtraContextSnapshot::new(
        42,
        "o".to_string(),
        vec![ContextoidRecord::new(5, NodeRecord::Root)],
        vec![],
    );
    assert_eq!(one.nodes().len(), 1);
    assert_eq!(one.nodes()[0].id(), 5);
}

#[test]
fn test_into_parts() {
    let (id, name, nodes, edges) = sample().into_parts();
    assert_eq!(id, 40);
    assert_eq!(name, "weather");
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges.len(), 1);
}

#[test]
fn test_clone_equality_and_debug() {
    let extra = sample();
    assert_eq!(extra.clone(), extra);
    assert_ne!(
        extra,
        ExtraContextSnapshot::new(40, "climate".to_string(), vec![], vec![])
    );
    assert!(format!("{extra:?}").contains("weather"));
}
