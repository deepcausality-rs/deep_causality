/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The store handle: construction, the backend it exposes, and `reserve` as a pass-through.
//! Expected values are what the in-memory backend answers.
//!
//! Corner cases (rows A to K): A `reserve(0)`, `test_reserve_is_a_pass_through`; every other row
//! belongs to the operations and is in the sibling files.

use deep_causality_context::ContextStore;
use deep_causality_context::{
    Context, Contextoid, ContextoidType, ContextuableGraph, Data, ExtendableContextuableGraph,
    UniformContext,
};
use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{ContextStorage, ContextoidId};

#[test]
fn test_the_store_is_thin() {
    let storage = MemoryStorage::new();
    let store = ContextStore::new(storage.clone());
    let id = block_on(store.storage().create_context("through the store")).unwrap();
    assert_eq!(
        block_on(storage.hydrate(&id)).unwrap().context().name(),
        "through the store"
    );
    let other = block_on(storage.create_context("through the backend")).unwrap();
    assert_eq!(
        block_on(store.storage().hydrate(&other))
            .unwrap()
            .context()
            .name(),
        "through the backend"
    );
}

#[test]
fn test_reserve_is_a_pass_through() {
    let store = ContextStore::new(MemoryStorage::new());
    let mut none = block_on(store.reserve(0)).unwrap();
    assert_eq!(none.remaining(), 0);
    assert_eq!(none.next(), None);
    let mut three = block_on(store.reserve(3)).unwrap();
    assert_eq!(three.remaining(), 3);
    let first = three.next().unwrap();
    assert!(first != 0);
}

#[test]
fn test_a_reserve_outlives_a_branch() {
    let store = ContextStore::new(MemoryStorage::new());
    let mut reserve = block_on(store.reserve(10)).unwrap();
    let scrapped: Vec<ContextoidId> = (0..3).map(|_| reserve.next().unwrap()).collect();
    let kept: Vec<ContextoidId> = (0..3).map(|_| reserve.next().unwrap()).collect();
    let mut branch: UniformContext = Context::with_capacity(1, "kept", 4);
    for (i, id) in kept.iter().enumerate() {
        branch
            .add_node(Contextoid::new(
                *id,
                ContextoidType::Datoid(Data::new(*id, i as u64)),
            ))
            .unwrap();
    }
    let _ = branch.extra_ctx_get_current_id();
    let container = block_on(store.store_branch("kept", &branch)).unwrap();
    let held = block_on(store.storage().lookup(&kept)).unwrap();
    assert!(held.iter().all(Option::is_some));
    let scrapped_held = block_on(store.storage().lookup(&scrapped)).unwrap();
    assert!(scrapped_held.iter().all(Option::is_none));
    assert_eq!(
        block_on(store.storage().hydrate(&container))
            .unwrap()
            .nodes()
            .len(),
        3
    );
}
