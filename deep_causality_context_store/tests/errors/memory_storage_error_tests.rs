/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every variant of `MemoryStorageErrorEnum` is constructed through its constructor and asserted
//! by variant. Expected values are the literals passed in; the display text is checked for each
//! literal it must carry. The tests that provoke each variant through the public API are in
//! `tests/utils_test/memory_storage/`.
//!
//! Corner cases (rows A to K): F identifier 0 and H `u64::MAX` in `test_boundary_identifiers`;
//! every other row n/a.

use deep_causality_context_store::{MemoryStorageError, MemoryStorageErrorEnum};
use std::error::Error;

#[test]
fn test_every_variant_by_constructor() {
    let cases: Vec<(MemoryStorageError, MemoryStorageErrorEnum, &str)> = vec![
        (
            MemoryStorageError::UnknownContext(7),
            MemoryStorageErrorEnum::UnknownContext(7),
            "container 7",
        ),
        (
            MemoryStorageError::UnknownNode(8),
            MemoryStorageErrorEnum::UnknownNode(8),
            "contextoid 8",
        ),
        (
            MemoryStorageError::UnknownEdge(1, 2),
            MemoryStorageErrorEnum::UnknownEdge { from: 1, to: 2 },
            "from 1 to 2",
        ),
        (
            MemoryStorageError::IdentityNotReserved(9),
            MemoryStorageErrorEnum::IdentityNotReserved(9),
            "identifier 9",
        ),
        (
            MemoryStorageError::NodeConflict(3),
            MemoryStorageErrorEnum::NodeConflict(3),
            "contextoid 3",
        ),
        (
            MemoryStorageError::ContextConflict(10),
            MemoryStorageErrorEnum::ContextConflict(10),
            "container 10",
        ),
        (
            MemoryStorageError::EdgeConflict(4, 5),
            MemoryStorageErrorEnum::EdgeConflict { from: 4, to: 5 },
            "from 4 to 5",
        ),
        (
            MemoryStorageError::SelfReference(6),
            MemoryStorageErrorEnum::SelfReference(6),
            "container 6",
        ),
        (
            MemoryStorageError::EventNotApplicable("ContextCreated"),
            MemoryStorageErrorEnum::EventNotApplicable("ContextCreated"),
            "ContextCreated",
        ),
        (
            MemoryStorageError::UnknownCursor(11),
            MemoryStorageErrorEnum::UnknownCursor(11),
            "cursor 11",
        ),
        (
            MemoryStorageError::UnknownCreated(12),
            MemoryStorageErrorEnum::UnknownCreated(12),
            "container 12",
        ),
    ];
    assert_eq!(cases.len(), 11);
    for (err, kind, needle) in cases {
        assert_eq!(err.kind(), &kind);
        assert_eq!(err, MemoryStorageError::new(kind));
        assert!(err.to_string().contains(needle), "{err}");
        assert!(err.to_string().starts_with("MemoryStorageError"));
    }
}

#[test]
fn test_boundary_identifiers() {
    assert!(
        MemoryStorageError::UnknownContext(0)
            .to_string()
            .contains("container 0")
    );
    assert!(
        MemoryStorageError::UnknownNode(u64::MAX)
            .to_string()
            .contains(&u64::MAX.to_string())
    );
}

#[test]
fn test_clone_equality_debug_and_source() {
    let err = MemoryStorageError::SelfReference(1);
    assert_eq!(err.clone(), err);
    assert_ne!(err, MemoryStorageError::SelfReference(2));
    assert!(format!("{err:?}").contains("SelfReference"));
    let dyn_err: &dyn Error = &err;
    assert!(dyn_err.source().is_none());
}
