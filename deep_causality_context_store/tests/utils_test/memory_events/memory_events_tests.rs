/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The stream handle's scope: the subscribed container and the containers it referenced when
//! the subscription began, and nothing added later. Expected values are the identifiers
//! `create_context` returned.
//!
//! Corner cases (rows A to K): A a container with no references, `test_scope_without_references`;
//! C the scope taken as of a cursor rather than now, `test_scope_is_as_of_the_cursor`, and a
//! reference to a retracted container, which leaves the scope with the container,
//! `test_scope_drops_a_retracted_reference`; every other row n/a.

use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{ContextStorage, ContextStorageStream};
use std::collections::BTreeSet;

#[test]
fn test_scope_without_references() {
    let storage = MemoryStorage::new();
    let a = block_on(storage.create_context("a")).unwrap();
    let (_, events) = block_on(storage.subscribe(&a, None)).unwrap();
    assert_eq!(events.scope(), &BTreeSet::from([a]));
}

#[test]
fn test_scope_holds_the_references_at_subscription() {
    let storage = MemoryStorage::new();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    let c = block_on(storage.create_context("c")).unwrap();
    block_on(storage.attach(a, b)).unwrap();
    block_on(storage.attach(b, c)).unwrap();
    let (_, events) = block_on(storage.subscribe(&a, None)).unwrap();
    assert_eq!(events.scope(), &BTreeSet::from([a, b]));
    let d = block_on(storage.create_context("d")).unwrap();
    block_on(storage.attach(a, d)).unwrap();
    assert_eq!(events.scope(), &BTreeSet::from([a, b]));
}

#[test]
fn test_scope_is_as_of_the_cursor() {
    let storage = MemoryStorage::new();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    let before = block_on(storage.apply_batch(&[])).unwrap();
    block_on(storage.attach(a, b)).unwrap();
    let (_, events) = block_on(storage.subscribe(&a, Some(before))).unwrap();
    assert_eq!(events.scope(), &BTreeSet::from([a]));
}

#[test]
fn test_scope_drops_a_retracted_reference() {
    let storage = MemoryStorage::new();
    let a = block_on(storage.create_context("a")).unwrap();
    let b = block_on(storage.create_context("b")).unwrap();
    block_on(storage.attach(a, b)).unwrap();
    block_on(storage.retract_context(b)).unwrap();
    let (snapshot, events) = block_on(storage.subscribe(&a, None)).unwrap();
    assert_eq!(events.scope(), &BTreeSet::from([a]));
    assert!(snapshot.extras().is_empty());
}
