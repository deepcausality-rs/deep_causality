/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Context, ContextStore, Datable, SpaceTemporal, Spatial, StoreError, Temporal};
use deep_causality_context_store::{
    ContextStorage, DataRecord, Recordable, SpaceRecord, SpaceTimeRecord, TimeRecord,
};

impl<S: ContextStorage> ContextStore<S> {
    /// The context a slice names, as a context the engine can reason over: the backend's
    /// snapshot restored, with each referenced container as an extra under its own identifier
    /// and name.
    #[allow(clippy::type_complexity)]
    pub async fn hydrate<D, S_, T, ST>(
        &self,
        spec: &S::Slice,
    ) -> Result<Context<D, S_, T, ST>, StoreError<S::Error>>
    where
        D: Datable + Clone + Recordable<DataRecord>,
        S_: Spatial + Clone + Recordable<SpaceRecord>,
        T: Temporal + Clone + Recordable<TimeRecord>,
        ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
    {
        let snapshot = self
            .storage
            .hydrate(spec)
            .await
            .map_err(StoreError::Storage)?;
        Ok(Context::restore(snapshot)?)
    }
}
