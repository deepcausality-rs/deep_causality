/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils_test::MemoryStorage;
use crate::utils_test::memory_storage::MemoryState;
use crate::{
    ContextEvent, ContextId, ContextSnapshot, ContextStorage, ContextoidId, ContextoidRecord,
    IdReserve, MemoryStorageError, RelationRecord,
};
use core::future::ready;

impl MemoryStorage {
    /// Runs one operation on the state and appends the events it emitted to the log. An
    /// operation validates before it folds, so a refusal leaves both untouched.
    fn commit(
        &self,
        operation: impl FnOnce(&mut MemoryState) -> Result<Vec<ContextEvent>, MemoryStorageError>,
    ) -> Result<(), MemoryStorageError> {
        let mut shared = self.lock();
        let events = operation(&mut shared.state)?;
        shared.log.extend(events);
        Ok(())
    }
}

impl ContextStorage for MemoryStorage {
    type Error = MemoryStorageError;
    type Slice = ContextId;

    fn reserve(&self, n: usize) -> impl Future<Output = Result<IdReserve, Self::Error>> + Send {
        let ids = self.lock().state.reserve(n);
        ready(Ok(IdReserve::new(ids)))
    }

    fn create_context(
        &self,
        name: &str,
    ) -> impl Future<Output = Result<ContextId, Self::Error>> + Send {
        let mut shared = self.lock();
        let (id, events) = shared.state.create_context(name);
        shared.log.extend(events);
        ready(Ok(id))
    }

    fn retract_context(
        &self,
        context: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.retract_context(context)))
    }

    fn create_node(
        &self,
        nodes: &[ContextoidRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.create_node(nodes)))
    }

    fn retract_node(
        &self,
        node: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.retract_node(node)))
    }

    fn create_edge(
        &self,
        edges: &[RelationRecord],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.create_edge(edges)))
    }

    fn retract_edge(
        &self,
        from: ContextoidId,
        to: ContextoidId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.retract_edge(from, to)))
    }

    fn link(
        &self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.link(context, nodes)))
    }

    fn unlink(
        &self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.unlink(context, nodes)))
    }

    fn attach(
        &self,
        context: ContextId,
        extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.attach(context, extra)))
    }

    fn detach(
        &self,
        context: ContextId,
        extra: ContextId,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.commit(|state| state.detach(context, extra)))
    }

    fn lookup(
        &self,
        ids: &[ContextoidId],
    ) -> impl Future<Output = Result<Vec<Option<ContextoidRecord>>, Self::Error>> + Send {
        ready(Ok(self.lock().state.lookup(ids)))
    }

    fn hydrate(
        &self,
        spec: &Self::Slice,
    ) -> impl Future<Output = Result<ContextSnapshot, Self::Error>> + Send {
        ready(self.lock().state.hydrate(*spec))
    }
}
