/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the variants; no expectation is computed.
//!
//! Corner cases (rows A to K): C the same number under both variants in
//! `test_held_and_created_are_distinct`; every other row n/a.

use deep_causality_context_store::ContainerRef;

#[test]
fn test_held_and_created_are_distinct() {
    let held = ContainerRef::Held(3);
    let created = ContainerRef::Created(3);
    assert_ne!(held, created);
    let copy = held;
    assert_eq!(copy, ContainerRef::Held(3));
    assert_ne!(created, ContainerRef::Created(4));
    assert!(format!("{created:?}").contains("Created"));
}
