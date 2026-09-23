/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The handle itself: construction, `Default`, and that clones share one store. Expected values
//! are what an empty store must answer.
//!
//! Corner cases (rows A to K): A an empty store in `test_a_new_store_holds_nothing`; every other
//! row belongs to the operations and is in the sibling files.

use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{ContextStorage, MemoryStorageError};

#[test]
fn test_a_new_store_holds_nothing() {
    let storage = MemoryStorage::new();
    assert_eq!(block_on(storage.lookup(&[1, 2])), Ok(vec![None, None]));
    assert_eq!(
        block_on(storage.hydrate(&1)),
        Err(MemoryStorageError::UnknownContext(1))
    );
}

#[test]
fn test_default_is_new() {
    let storage = MemoryStorage::default();
    assert_eq!(block_on(storage.lookup(&[])), Ok(vec![]));
}

#[test]
fn test_clones_share_one_store() {
    let storage = MemoryStorage::new();
    let twin = storage.clone();
    let id = block_on(storage.create_context("shared")).unwrap();
    let snapshot = block_on(twin.hydrate(&id)).unwrap();
    assert_eq!(snapshot.context().name(), "shared");
}
