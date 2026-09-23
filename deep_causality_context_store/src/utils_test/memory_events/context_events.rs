/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils_test::MemoryEvents;
use crate::{ContextEventItem, ContextEvents, MemoryStorageError};
use core::future::ready;
use std::sync::PoisonError;

impl ContextEvents for MemoryEvents {
    type Error = MemoryStorageError;
    type Cursor = usize;

    /// Advances past every event outside the scope and yields the first one inside it, with the
    /// log position after it as the cursor; `None` at the end of the log.
    fn next(&mut self) -> impl Future<Output = ContextEventItem<Self::Cursor, Self::Error>> + Send {
        let shared = self.shared();
        let log = shared.lock().unwrap_or_else(PoisonError::into_inner);
        let mut item = None;
        while self.position < log.log.len() && item.is_none() {
            let event = &log.log[self.position];
            self.position += 1;
            if self.in_scope(event) {
                item = Some(Ok((self.position, event.clone())));
            }
        }
        drop(log);
        ready(item)
    }
}
