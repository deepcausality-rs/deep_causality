/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Context, ContextStore, Datable, SpaceTemporal, Spatial, StoreError, Temporal};
use deep_causality_context_store::{
    ContextSnapshot, ContextStorage, ContextoidRecord, DataRecord, ExtraContextSnapshot,
    NodeRecord, Recordable, SpaceRecord, SpaceTimeRecord, Substrate, TimeRecord,
};

impl<S: ContextStorage> ContextStore<S> {
    /// Creates nodes through a substrate: every data payload that is not already a `Reference`
    /// is deposited and replaced by where it now lives before the store sees it, so a
    /// value-typed context reaches a store that holds references alone.
    pub async fn create_node_via<B: Substrate>(
        &self,
        substrate: &B,
        nodes: &[ContextoidRecord],
    ) -> Result<(), StoreError<S::Error, B::Error>> {
        let mut deposited = Vec::with_capacity(nodes.len());
        for record in nodes {
            let node = match record.node() {
                NodeRecord::Data(payload) if !matches!(payload, DataRecord::Reference(_)) => {
                    let reference = substrate
                        .deposit(record.id(), payload)
                        .await
                        .map_err(StoreError::Substrate)?;
                    NodeRecord::Data(DataRecord::Reference(reference))
                }
                other => other.clone(),
            };
            deposited.push(ContextoidRecord::new(record.id(), node));
        }
        self.storage
            .create_node(&deposited)
            .await
            .map_err(StoreError::Storage)
    }

    /// Hydrates through a substrate: every `Reference` payload is resolved into the record the
    /// substrate returns before the context is restored, so the value type the caller asked for
    /// is what the data nodes hold.
    #[allow(clippy::type_complexity)]
    pub async fn hydrate_via<B, D, S_, T, ST>(
        &self,
        substrate: &B,
        spec: &S::Slice,
    ) -> Result<Context<D, S_, T, ST>, StoreError<S::Error, B::Error>>
    where
        B: Substrate,
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
        let (version, context, nodes, edges, extras) = snapshot.into_parts();
        let nodes = Self::resolve_all(substrate, nodes).await?;
        let mut resolved_extras = Vec::with_capacity(extras.len());
        for extra in extras {
            let (id, name, nodes, edges) = extra.into_parts();
            let nodes = Self::resolve_all(substrate, nodes).await?;
            resolved_extras.push(ExtraContextSnapshot::new(id, name, nodes, edges));
        }
        let snapshot =
            ContextSnapshot::with_version(version, context, nodes, edges, resolved_extras);
        Ok(Context::restore(snapshot)?)
    }

    /// Every `Reference` payload replaced by the record the substrate holds for it.
    async fn resolve_all<B: Substrate>(
        substrate: &B,
        nodes: Vec<ContextoidRecord>,
    ) -> Result<Vec<ContextoidRecord>, StoreError<S::Error, B::Error>> {
        let mut resolved = Vec::with_capacity(nodes.len());
        for record in nodes {
            let node = match record.node() {
                NodeRecord::Data(DataRecord::Reference(reference)) => NodeRecord::Data(
                    substrate
                        .resolve(reference)
                        .await
                        .map_err(StoreError::Substrate)?,
                ),
                other => other.clone(),
            };
            resolved.push(ContextoidRecord::new(record.id(), node));
        }
        Ok(resolved)
    }
}
