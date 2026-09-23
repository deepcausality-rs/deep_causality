/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils_test::memory_storage::Shared;
use crate::{ContextEvent, ContextId};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

mod context_events;

/// The stream a `MemoryStorage` subscription yields.
///
/// It reads the shared log from a position and delivers every node and edge event, and the
/// membership, attachment and retraction events of the containers in its scope: the subscribed
/// container and the containers it referenced when the subscription began. The scope is fixed for
/// the life of the stream. The stream ends at the end of the log.
pub struct MemoryEvents {
    shared: Arc<Mutex<Shared>>,
    position: usize,
    scope: BTreeSet<ContextId>,
}

impl MemoryEvents {
    pub(crate) fn new(
        shared: Arc<Mutex<Shared>>,
        position: usize,
        scope: BTreeSet<ContextId>,
    ) -> Self {
        Self {
            shared,
            position,
            scope,
        }
    }

    /// The containers this stream reports on.
    pub fn scope(&self) -> &BTreeSet<ContextId> {
        &self.scope
    }

    pub(crate) fn shared(&self) -> Arc<Mutex<Shared>> {
        Arc::clone(&self.shared)
    }

    /// Whether an event concerns this stream: every node and edge event does, a container event
    /// does when the container is in scope, and a container's creation never does, because a
    /// subscription's scope is fixed at its start.
    pub(crate) fn in_scope(&self, event: &ContextEvent) -> bool {
        match event {
            ContextEvent::NodeCreated(_)
            | ContextEvent::NodeRetracted(_)
            | ContextEvent::EdgeCreated(_)
            | ContextEvent::EdgeRetracted { .. } => true,
            ContextEvent::ContextCreated(_) => false,
            ContextEvent::ContextRetracted(id) => self.scope.contains(id),
            ContextEvent::NodeLinked { context, .. }
            | ContextEvent::NodeUnlinked { context, .. }
            | ContextEvent::ContextAttached { context, .. }
            | ContextEvent::ContextDetached { context, .. }
            | ContextEvent::NodeEntered { context, .. }
            | ContextEvent::NodeLeft { context, .. } => self.scope.contains(context),
        }
    }
}
