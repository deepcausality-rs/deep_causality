/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Context, ContextStore, Datable, SpaceTemporal, Spatial, StoreError, Temporal};
use deep_causality_context_store::{
    ContainerRef, ContextId, ContextStorage, ContextWrite, ContextoidId, ContextoidRecord,
    DataRecord, IdReserve, ProjectionError, Recordable, RelationRecord, SpaceRecord,
    SpaceTimeRecord, TimeRecord,
};
use std::collections::{HashMap, HashSet};

/// A graph's map from a branch identifier to the fresh one it links instead.
type Remap = HashMap<ContextoidId, ContextoidId>;

impl<S: ContextStorage> ContextStore<S> {
    /// Stores a branch as a new context that links what it shares and creates what it changed.
    ///
    /// Every node the store holds under the same record is linked; every node it does not hold
    /// is created; every node it holds under a different record is created under a fresh
    /// reserved identifier and the branch's edges are remapped to it. The rule is applied once
    /// over the base graph and every extra together, so a node two graphs share keeps one
    /// identifier in the store, and two graphs carrying different records under one identifier
    /// store two nodes. Each extra of the branch becomes a container of its own under the
    /// extra's name, attached to the new one. The in-memory branch is unchanged; a stored branch
    /// is a record of a world, not a continuation of one. To keep exploring it, hydrate it.
    ///
    /// The lookup and the reserve of fresh identifiers are reads and a lease; every write goes to
    /// the store as one `commit`, so a refusal leaves the store without any of the branch.
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
        let extras: Vec<(String, Vec<ContextoidRecord>, Vec<RelationRecord>)> = extras
            .into_iter()
            .map(|extra| {
                let (_, name, nodes, edges) = extra.into_parts();
                (name, nodes, edges)
            })
            .collect();

        // One remap per graph: the base first, then each extra in order.
        let node_sets: Vec<&[ContextoidRecord]> = std::iter::once(nodes.as_slice())
            .chain(extras.iter().map(|(_, nodes, _)| nodes.as_slice()))
            .collect();
        let (to_create, remaps) = self.plan_nodes(&node_sets).await?;

        // Container i of the commit is graph i: the base is 0, extra i is i + 1.
        let mut writes = Vec::new();
        if !to_create.is_empty() {
            writes.push(ContextWrite::CreateNode(to_create));
        }
        container_writes(&mut writes, 0, name, &nodes, &edges, &remaps[0]);
        for (i, ((extra_name, nodes, edges), remap)) in extras.iter().zip(&remaps[1..]).enumerate()
        {
            container_writes(&mut writes, i + 1, extra_name, nodes, edges, remap);
            writes.push(ContextWrite::Attach {
                context: ContainerRef::Created(0),
                extra: ContainerRef::Created(i + 1),
            });
        }
        let created = self
            .storage
            .commit(&writes)
            .await
            .map_err(StoreError::Storage)?;
        created
            .first()
            .copied()
            .ok_or(StoreError::Projection(ProjectionError::Identity(
                0,
                "the backend's commit returned no created container",
            )))
    }

    /// Applies the node rule once over every graph of a branch and returns the records to create
    /// with one remap per graph, from a branch identifier to the fresh one that graph links.
    ///
    /// Each distinct record is handled once, however many graphs carry it, so a record two graphs
    /// share keeps one identifier in the store. A record the store holds under its identifier is
    /// linked; the first record under an identifier the store does not hold is created under it;
    /// every other record is created under a fresh reserved identifier.
    async fn plan_nodes(
        &self,
        graphs: &[&[ContextoidRecord]],
    ) -> Result<(Vec<ContextoidRecord>, Vec<Remap>), StoreError<S::Error>> {
        // The distinct records in first-seen order, each with the graphs that carry it.
        let mut records: Vec<(&ContextoidRecord, Vec<usize>)> = Vec::new();
        let mut by_id: HashMap<ContextoidId, Vec<usize>> = HashMap::new();
        for (graph, nodes) in graphs.iter().enumerate() {
            for record in nodes.iter() {
                let seen = by_id.entry(record.id()).or_default();
                match seen.iter().find(|&&i| records[i].0 == record) {
                    Some(&i) => records[i].1.push(graph),
                    None => {
                        seen.push(records.len());
                        records.push((record, vec![graph]));
                    }
                }
            }
        }

        let ids: Vec<ContextoidId> = by_id.keys().copied().collect();
        let held: HashMap<ContextoidId, Option<ContextoidRecord>> = ids
            .iter()
            .copied()
            .zip(
                self.storage
                    .lookup(&ids)
                    .await
                    .map_err(StoreError::Storage)?,
            )
            .collect();

        // Whether each record needs a fresh identifier.
        let mut claimed: HashSet<ContextoidId> = HashSet::new();
        let mut to_create = Vec::with_capacity(records.len());
        let mut needs_fresh = Vec::with_capacity(records.len());
        for (record, _) in &records {
            let fresh = match held.get(&record.id()) {
                Some(Some(held)) if held == *record => false,
                Some(None) if claimed.insert(record.id()) => {
                    to_create.push((*record).clone());
                    false
                }
                _ => true,
            };
            needs_fresh.push(fresh);
        }
        let conflicts = needs_fresh.iter().filter(|&&fresh| fresh).count();
        let mut fresh = if conflicts == 0 {
            IdReserve::new(Vec::new())
        } else {
            self.storage
                .reserve(conflicts)
                .await
                .map_err(StoreError::Storage)?
        };
        let mut remaps: Vec<Remap> = vec![HashMap::new(); graphs.len()];
        for ((record, carriers), _) in records
            .iter()
            .zip(&needs_fresh)
            .filter(|(_, needs)| **needs)
        {
            let id = fresh.next().ok_or(ProjectionError::Identity(
                record.id(),
                "the backend's reserve held fewer identifiers than asked",
            ))?;
            for &graph in carriers {
                remaps[graph].insert(record.id(), id);
            }
            to_create.push(ContextoidRecord::new(id, record.node().clone()));
        }
        Ok((to_create, remaps))
    }
}

/// The writes that store one graph as the commit's `index`-th container: the edges under the
/// remap, the container, the links.
fn container_writes(
    writes: &mut Vec<ContextWrite>,
    index: usize,
    name: &str,
    nodes: &[ContextoidRecord],
    edges: &[RelationRecord],
    remap: &Remap,
) {
    let at = |id| remap.get(&id).copied().unwrap_or(id);
    let edges: Vec<RelationRecord> = edges
        .iter()
        .map(|edge| RelationRecord::new(at(edge.from()), at(edge.to()), edge.kind()))
        .collect();
    if !edges.is_empty() {
        writes.push(ContextWrite::CreateEdge(edges));
    }
    writes.push(ContextWrite::CreateContext(name.to_string()));
    let to_link: Vec<ContextoidId> = nodes.iter().map(|record| at(record.id())).collect();
    if !to_link.is_empty() {
        writes.push(ContextWrite::Link {
            context: ContainerRef::Created(index),
            nodes: to_link,
        });
    }
}
