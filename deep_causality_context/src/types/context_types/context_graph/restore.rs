/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::extra_context::ExtraContext;
use crate::{
    Context, Contextoid, ContextuableGraph, Datable, ExtendableContextuableGraph, SpaceTemporal,
    Spatial, Temporal,
};
use deep_causality_context_store::{
    ContextSnapshot, ContextoidId, ContextoidRecord, DataRecord, ProjectionError, RECORD_VERSION,
    Recordable, RelationRecord, SpaceRecord, SpaceTimeRecord, TimeRecord,
};
use std::collections::{HashMap, HashSet};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> Context<D, S, T, ST>
where
    D: Datable + Clone + Recordable<DataRecord>,
    S: Spatial + Clone + Recordable<SpaceRecord>,
    T: Temporal + Clone + Recordable<TimeRecord>,
    ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
{
    /// A context rebuilt from a snapshot: the base graph and every extra under the identifier
    /// and name the snapshot gives it, with no extra context current and every index empty.
    /// Every extra's identifier is taken as the store's, so store events reach it.
    ///
    /// Refused with `ProjectionError::Version` for a snapshot newer than `RECORD_VERSION`, and
    /// with `ProjectionError::Identity` for a node carried twice, an edge naming no node, an edge
    /// carried twice (a relation exists once between two nodes), or an extra under identifier 0
    /// or carried twice.
    pub fn restore(snapshot: ContextSnapshot) -> Result<Self, ProjectionError> {
        if snapshot.version() > RECORD_VERSION {
            return Err(ProjectionError::Version(snapshot.version(), RECORD_VERSION));
        }
        let (_, context, nodes, edges, extras) = snapshot.into_parts();
        let mut restored = Self::with_capacity(context.id(), context.name(), nodes.len());
        restored.fill(&nodes, &edges, false)?;
        for extra in extras {
            let (id, name, nodes, edges) = extra.into_parts();
            if id == 0 {
                return Err(ProjectionError::Identity(
                    id,
                    "an extra context identifier is 0",
                ));
            }
            restored
                .insert_extra(id, ExtraContext::stored(&name, nodes.len()), true)
                .map_err(|_| {
                    ProjectionError::Identity(id, "an extra context identifier is carried twice")
                })?;
            restored.fill(&nodes, &edges, true)?;
        }
        restored.extra_context_id = 0;
        Ok(restored)
    }

    /// Adds the nodes, then the edges, to the base graph or to the current extra context,
    /// mapping identifiers to the indices the graph assigns as it goes.
    fn fill(
        &mut self,
        nodes: &[ContextoidRecord],
        edges: &[RelationRecord],
        into_extra: bool,
    ) -> Result<(), ProjectionError> {
        let mut index_of: HashMap<ContextoidId, usize> = HashMap::with_capacity(nodes.len());
        for record in nodes {
            let id = record.id();
            if index_of.contains_key(&id) {
                return Err(ProjectionError::Identity(
                    id,
                    "a node identifier is carried twice",
                ));
            }
            let node = Contextoid::from_record(id, record.node().clone())?;
            let added = if into_extra {
                self.extra_ctx_add_node(node)
            } else {
                self.add_node(node)
            };
            let index =
                added.map_err(|_| ProjectionError::Identity(id, "a node could not be added"))?;
            index_of.insert(id, index);
        }
        let mut seen: HashSet<(ContextoidId, ContextoidId)> = HashSet::with_capacity(edges.len());
        for edge in edges {
            let unknown =
                |id| ProjectionError::Identity(id, "an edge names an identifier no node carries");
            let a = *index_of
                .get(&edge.from())
                .ok_or_else(|| unknown(edge.from()))?;
            let b = *index_of.get(&edge.to()).ok_or_else(|| unknown(edge.to()))?;
            if !seen.insert((edge.from(), edge.to())) {
                return Err(ProjectionError::Identity(
                    edge.from(),
                    "an edge is carried twice",
                ));
            }
            let added = if into_extra {
                self.extra_ctx_add_edge(a, b, edge.kind())
            } else {
                self.add_edge(a, b, edge.kind())
            };
            added.map_err(|_| {
                ProjectionError::Identity(edge.from(), "an edge could not be added")
            })?;
        }
        Ok(())
    }
}
