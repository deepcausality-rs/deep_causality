/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context_store::{ContextStorage, IdReserve};

use crate::StoreError;

mod hydrate;
mod store_branch;
mod subscribe;
mod substrate;

/// Projection, then storage, in one place.
///
/// A caller may equally call `Context::snapshot` and the storage directly; the store exists so
/// that the common paths are one call and the branch rule is written once. It holds the backend
/// by value and adds no cache, so the backend seen through [`ContextStore::storage`] is the one
/// the store writes to.
pub struct ContextStore<S: ContextStorage> {
    storage: S,
}

impl<S: ContextStorage> ContextStore<S> {
    pub const fn new(storage: S) -> Self {
        Self { storage }
    }

    pub const fn storage(&self) -> &S {
        &self.storage
    }

    /// Identifiers for nodes a context will create in memory: the backend's reserve, wrapped.
    pub async fn reserve(&self, n: usize) -> Result<IdReserve, StoreError<S::Error>> {
        self.storage.reserve(n).await.map_err(StoreError::Storage)
    }
}
