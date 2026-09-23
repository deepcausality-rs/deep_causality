/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The state is a fold over the log: replaying the log to any cursor reproduces the state as of
//! that cursor. Pinned through the public API by comparing `subscribe(Some(cursor))` snapshots
//! with `hydrate` taken at the same points. Expected values are the records handed in.
//!
//! Corner cases (rows A to K): A the empty log, cursor 0, in `test_replay_at_zero_is_empty`; C
//! a replay to the end equals the live state, `test_replay_reproduces_every_intermediate_state`;
//! every other row n/a.

use deep_causality_context_store::utils_test::{MemoryStorage, block_on};
use deep_causality_context_store::{
    ContextSnapshot, ContextStorage, ContextStorageStream, ContextoidId, ContextoidRecord,
    DataRecord, MemoryStorageError, NodeRecord, RelationKind, RelationRecord,
};

fn number(id: ContextoidId, value: f64) -> ContextoidRecord {
    ContextoidRecord::new(id, NodeRecord::Data(DataRecord::Number(value)))
}

#[test]
fn test_replay_at_zero_is_empty() {
    let storage = MemoryStorage::new();
    let c = block_on(storage.create_context("c")).unwrap();
    assert_eq!(
        block_on(storage.subscribe(&c, Some(0))).map(|_| ()),
        Err(MemoryStorageError::UnknownContext(c))
    );
}

#[test]
fn test_replay_reproduces_every_intermediate_state() {
    let storage = MemoryStorage::new();
    let n: Vec<ContextoidId> = block_on(storage.reserve(2)).unwrap().collect();
    let mut checkpoints: Vec<(usize, ContextSnapshot)> = Vec::new();
    let c = block_on(storage.create_context("c")).unwrap();
    let d = block_on(storage.create_context("d")).unwrap();
    let mut record = |storage: &MemoryStorage| {
        let cursor = block_on(storage.apply_batch(&[])).unwrap();
        checkpoints.push((cursor, block_on(storage.hydrate(&c)).unwrap()));
    };
    record(&storage);
    block_on(storage.create_node(&[number(n[0], 1.0), number(n[1], 2.0)])).unwrap();
    block_on(storage.link(c, &n)).unwrap();
    record(&storage);
    block_on(storage.create_edge(&[RelationRecord::new(n[0], n[1], RelationKind::Datial)]))
        .unwrap();
    block_on(storage.attach(c, d)).unwrap();
    block_on(storage.link(d, &[n[1]])).unwrap();
    record(&storage);
    block_on(storage.retract_node(n[1])).unwrap();
    block_on(storage.detach(c, d)).unwrap();
    record(&storage);
    for (cursor, expected) in checkpoints {
        let (replayed, _) = block_on(storage.subscribe(&c, Some(cursor))).unwrap();
        assert_eq!(replayed, expected, "at cursor {cursor}");
    }
}
