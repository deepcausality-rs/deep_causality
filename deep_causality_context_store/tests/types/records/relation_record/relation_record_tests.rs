/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructor; `from` and `to` are different
//! numbers so a swapped getter is caught.
//!
//! Corner cases (rows A to K): A/B n/a; C both ends equal (a self-loop) in `test_self_loop`, and
//! the reversed pair in `test_copy_equality_and_debug`; D/E n/a; F identifier 0 in
//! `test_self_loop`; G n/a; H n/a; I/J/K n/a.

use deep_causality_context_store::{RelationKind, RelationRecord};

#[test]
fn test_new_and_getters() {
    let record = RelationRecord::new(1, 2, RelationKind::Temporal);
    assert_eq!(record.from(), 1);
    assert_eq!(record.to(), 2);
    assert_eq!(record.kind(), RelationKind::Temporal);
}

#[test]
fn test_self_loop() {
    let record = RelationRecord::new(0, 0, RelationKind::Datial);
    assert_eq!(record.from(), record.to());
    assert_eq!(record.from(), 0);
}

#[test]
fn test_copy_equality_and_debug() {
    let record = RelationRecord::new(1, 2, RelationKind::Spatial);
    let copy = record;
    assert_eq!(record, copy);
    assert_ne!(record, RelationRecord::new(2, 1, RelationKind::Spatial));
    assert_ne!(record, RelationRecord::new(1, 2, RelationKind::Datial));
    assert!(format!("{record:?}").contains("Spatial"));
}
