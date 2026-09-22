/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructors and `RECORD_VERSION`, which the
//! constants test pins to 1 independently.
//!
//! Corner cases (rows A to K): A empty snapshot in `test_empty_snapshot`; B n/a; C n/a; D n/a;
//! E version at, below and above the threshold in `test_with_version_carries_an_explicit_version`
//! (`RECORD_VERSION`, 0, `RECORD_VERSION + 1`; refusal is `Context::restore`'s and is tested
//! there); F version 0 in the same test; G n/a; H `u16::MAX` in the same test; I/J/K n/a.

use deep_causality_context_store::{
    ContextRecord, ContextSnapshot, ContextoidRecord, ExtraContextSnapshot, NodeRecord,
    RECORD_VERSION, RelationKind, RelationRecord,
};

fn sample() -> ContextSnapshot {
    ContextSnapshot::new(
        ContextRecord::new(9, "base".to_string()),
        vec![
            ContextoidRecord::new(1, NodeRecord::Root),
            ContextoidRecord::new(2, NodeRecord::Root),
        ],
        vec![RelationRecord::new(1, 2, RelationKind::SpaceTemporal)],
        vec![ExtraContextSnapshot::new(
            40,
            "weather".to_string(),
            vec![],
            vec![],
        )],
    )
}

#[test]
fn test_new_is_at_the_current_version() {
    let snapshot = sample();
    assert_eq!(snapshot.version(), RECORD_VERSION);
    assert_eq!(snapshot.context().id(), 9);
    assert_eq!(snapshot.context().name(), "base");
    assert_eq!(snapshot.nodes().len(), 2);
    assert_eq!(snapshot.nodes()[1].id(), 2);
    assert_eq!(snapshot.edges().len(), 1);
    assert_eq!(snapshot.extras().len(), 1);
    assert_eq!(snapshot.extras()[0].id(), 40);
}

#[test]
fn test_empty_snapshot() {
    let empty = ContextSnapshot::new(
        ContextRecord::new(1, "e".to_string()),
        vec![],
        vec![],
        vec![],
    );
    assert!(empty.nodes().is_empty());
    assert!(empty.edges().is_empty());
    assert!(empty.extras().is_empty());
}

#[test]
fn test_with_version_carries_an_explicit_version() {
    let context = || ContextRecord::new(9, "base".to_string());
    for version in [0, RECORD_VERSION, RECORD_VERSION + 1, u16::MAX] {
        let snapshot = ContextSnapshot::with_version(version, context(), vec![], vec![], vec![]);
        assert_eq!(snapshot.version(), version);
    }
}

#[test]
fn test_into_parts() {
    let (version, context, nodes, edges, extras) = sample().into_parts();
    assert_eq!(version, RECORD_VERSION);
    assert_eq!(context.name(), "base");
    assert_eq!(nodes.len(), 2);
    assert_eq!(edges[0].kind(), RelationKind::SpaceTemporal);
    assert_eq!(extras[0].name(), "weather");
}

#[test]
fn test_clone_equality_and_debug() {
    let snapshot = sample();
    assert_eq!(snapshot.clone(), snapshot);
    let other = ContextSnapshot::new(
        ContextRecord::new(10, "base".to_string()),
        vec![],
        vec![],
        vec![],
    );
    assert_ne!(snapshot, other);
    assert!(format!("{snapshot:?}").contains("base"));
}
