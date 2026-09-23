/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Context, ContextStore, Datable, SpaceTemporal, Spatial, StoreError, Temporal};
use deep_causality_context_store::{
    ContextId, ContextStorage, ContextoidId, ContextoidRecord, DataRecord, IdReserve,
    ProjectionError, Recordable, RelationRecord, SpaceRecord, SpaceTimeRecord, TimeRecord,
};
use std::collections::HashMap;

impl<S: ContextStorage> ContextStore<S> {
    /// Stores a branch as a new context that links what it shares and creates what it changed.
    ///
    /// Every node the store holds under the same record is linked; every node it does not hold
    /// is created; every node it holds under a different record is created under a fresh
    /// reserved identifier and the branch's edges are remapped to it. Each extra of the branch
    /// becomes a container of its own under the extra's name, attached to the new one. The
    /// in-memory branch is unchanged; a stored branch is a record of a world, not a continuation
    /// of one. To keep exploring it, hydrate it.
    #[allow(clippy::type_complexity)]
    pub async fn store_branch<D, S_, T, ST>(
        &self,
        name: &str,
        branch: &Context<D, S_, T, ST>,
    ) -> Result<ContextId, StoreError<S::Error>>
    where
        D: Datable + Clone + Recordable<DataRecord>,
        S_: Spatial + Clone + Recordable<SpaceRecord>,
        T: Temporal + Clone + Recordable<TimeRecord>,
        ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
    {
        let (_, _, nodes, edges, extras) = branch.snapshot()?.into_parts();
        let base = self.store_graph(name, nodes, edges).await?;
        for extra in extras {
            let (_, extra_name, nodes, edges) = extra.into_parts();
            let container = self.store_graph(&extra_name, nodes, edges).await?;
            self.storage
                .attach(base, container)
                .await
                .map_err(StoreError::Storage)?;
        }
        Ok(base)
    }

    /// One graph into one new container: the node rule, the edges, the container, the links.
    async fn store_graph(
        &self,
        name: &str,
        nodes: Vec<ContextoidRecord>,
        edges: Vec<RelationRecord>,
    ) -> Result<ContextId, StoreError<S::Error>> {
        let ids: Vec<ContextoidId> = nodes.iter().map(ContextoidRecord::id).collect();
        let held = self
            .storage
            .lookup(&ids)
            .await
            .map_err(StoreError::Storage)?;
        let conflicts = held
            .iter()
            .zip(&nodes)
            .filter(|(held, record)| held.as_ref().is_some_and(|held| held != *record))
            .count();
        let mut fresh = if conflicts == 0 {
            IdReserve::new(Vec::new())
        } else {
            self.storage
                .reserve(conflicts)
                .await
                .map_err(StoreError::Storage)?
        };
        let mut remap: HashMap<ContextoidId, ContextoidId> = HashMap::with_capacity(conflicts);
        let mut to_create = Vec::with_capacity(nodes.len());
        let mut to_link = Vec::with_capacity(nodes.len());
        for (record, held) in nodes.into_iter().zip(held) {
            match held {
                Some(held) if held == record => to_link.push(record.id()),
                Some(_) => {
                    let id = fresh.next().ok_or(ProjectionError::Identity(
                        record.id(),
                        "the backend's reserve held fewer identifiers than asked",
                    ))?;
                    remap.insert(record.id(), id);
                    to_link.push(id);
                    to_create.push(ContextoidRecord::new(id, record.node().clone()));
                }
                None => {
                    to_link.push(record.id());
                    to_create.push(record);
                }
            }
        }
        if !to_create.is_empty() {
            self.storage
                .create_node(&to_create)
                .await
                .map_err(StoreError::Storage)?;
        }
        let edges: Vec<RelationRecord> = edges
            .iter()
            .map(|edge| {
                let at = |id| remap.get(&id).copied().unwrap_or(id);
                RelationRecord::new(at(edge.from()), at(edge.to()), edge.kind())
            })
            .collect();
        if !edges.is_empty() {
            self.storage
                .create_edge(&edges)
                .await
                .map_err(StoreError::Storage)?;
        }
        let container = self
            .storage
            .create_context(name)
            .await
            .map_err(StoreError::Storage)?;
        if !to_link.is_empty() {
            self.storage
                .link(container, &to_link)
                .await
                .map_err(StoreError::Storage)?;
        }
        Ok(container)
    }
}
