/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils_test::memory_storage::MemoryState;
use crate::utils_test::{MemoryEvents, MemoryStorage};
use crate::{ContextEvent, ContextSnapshot, ContextStorageStream, MemoryStorageError};
use core::future::ready;

impl ContextStorageStream for MemoryStorage {
    type Cursor = usize;
    type Events = MemoryEvents;

    fn subscribe(
        &self,
        spec: &Self::Slice,
        from: Option<Self::Cursor>,
    ) -> impl Future<Output = Result<(ContextSnapshot, Self::Events), Self::Error>> + Send {
        let shared = self.lock();
        let position = from.unwrap_or(shared.log.len());
        if position > shared.log.len() {
            return ready(Err(MemoryStorageError::UnknownCursor(position)));
        }
        // A cursor names a past state, which only a replay of the log up to it reproduces.
        let replayed = from.map(|cursor| {
            let mut state = MemoryState::new();
            shared.log[..cursor]
                .iter()
                .for_each(|event| state.fold(event));
            state
        });
        let state = replayed.as_ref().unwrap_or(&shared.state);
        let result = state
            .hydrate(*spec)
            .and_then(|snapshot| Ok((snapshot, state.scope(*spec)?)))
            .map(|(snapshot, scope)| (snapshot, MemoryEvents::new(self.shared(), position, scope)));
        ready(result)
    }

    fn apply(
        &self,
        event: &ContextEvent,
    ) -> impl Future<Output = Result<Self::Cursor, Self::Error>> + Send {
        let mut shared = self.lock();
        let result = shared.state.apply(event).map(|events| {
            shared.log.extend(events);
            shared.log.len()
        });
        ready(result)
    }

    fn apply_batch(
        &self,
        events: &[ContextEvent],
    ) -> impl Future<Output = Result<Self::Cursor, Self::Error>> + Send {
        let mut shared = self.lock();
        let mut working = shared.state.clone();
        let mut emitted = Vec::new();
        for event in events {
            match working.apply(event) {
                Ok(new) => emitted.extend(new),
                Err(error) => return ready(Err(error)),
            }
        }
        shared.state = working;
        shared.log.extend(emitted);
        ready(Ok(shared.log.len()))
    }
}
