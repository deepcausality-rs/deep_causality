/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextEvent;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

mod context_storage;
mod context_storage_stream;
mod memory_state;

pub(crate) use memory_state::MemoryState;

/// The state and the event log behind one `MemoryStorage`, shared with every stream it hands out.
pub(crate) struct Shared {
    pub(crate) state: MemoryState,
    pub(crate) log: Vec<ContextEvent>,
}

/// An in-memory backend implementing `ContextStorage` and `ContextStorageStream`.
///
/// Its state is a fold over its own event log: every mutating operation appends the events it
/// emits, a subscription from a cursor replays the log to that position into a fresh state, and a
/// batch applies to a clone of the state and commits only on success. Identifiers come from one
/// counter that starts at 1 and is never reused, so no container is 0 and no reserve repeats.
///
/// A `Mutex` gives interior mutability behind `&self`; a poisoned lock is recovered rather than
/// reported, because nothing this backend does panics while holding it. Clones share one store.
#[derive(Clone)]
pub struct MemoryStorage {
    shared: Arc<Mutex<Shared>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Shared {
                state: MemoryState::new(),
                log: Vec::new(),
            })),
        }
    }

    pub(crate) fn lock(&self) -> MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn shared(&self) -> Arc<Mutex<Shared>> {
        Arc::clone(&self.shared)
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}
