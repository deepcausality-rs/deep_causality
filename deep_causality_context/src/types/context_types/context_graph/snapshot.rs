/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Context, Contextoid, Datable, RelationKind, SpaceTemporal, Spatial, Temporal};
use deep_causality_context_store::{
    ContextRecord, ContextSnapshot, ContextoidRecord, DataRecord, ExtraContextSnapshot,
    ProjectionError, Recordable, RelationRecord, SpaceRecord, SpaceTimeRecord, TimeRecord,
};
use deep_causality_core::Identifiable;
use ultragraph::{GraphView, UltraGraphWeighted};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> Context<D, S, T, ST>
where
    D: Datable + Clone + Recordable<DataRecord>,
    S: Spatial + Clone + Recordable<SpaceRecord>,
    T: Temporal + Clone + Recordable<TimeRecord>,
    ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
{
    /// The context as a store holds it: every node and edge of the base graph and of each extra,
    /// by identifier, in canonical order (nodes by identifier, edges by `(from, to)`, extras by
    /// identifier). The index maps, the current extra context and the frozen form are run-time
    /// state and are not recorded.
    pub fn snapshot(&self) -> Result<ContextSnapshot, ProjectionError> {
        let (nodes, edges) = Self::walk(&self.base_context)?;
        let mut extras = match &self.extra_contexts {
            Some(extra_contexts) => extra_contexts
                .iter()
                .map(|(id, extra)| {
                    Self::walk(&extra.graph).map(|(nodes, edges)| {
                        ExtraContextSnapshot::new(*id, extra.name.clone(), nodes, edges)
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => Vec::new(),
        };
        extras.sort_by_key(ExtraContextSnapshot::id);
        Ok(ContextSnapshot::new(
            ContextRecord::new(self.id, self.name.clone()),
            nodes,
            edges,
            extras,
        ))
    }

    /// Every live node with its record and every edge with its relation, by identifier. The
    /// graph's `get_edges` reports only edges to live nodes, so every target has an identifier.
    fn walk(
        graph: &UltraGraphWeighted<Contextoid<D, S, T, ST>, RelationKind>,
    ) -> Result<(Vec<ContextoidRecord>, Vec<RelationRecord>), ProjectionError> {
        let mut nodes = Vec::with_capacity(graph.number_nodes());
        let mut edges = Vec::with_capacity(graph.number_edges());
        let last = graph.get_last_index().map_or(0, |last| last + 1);
        for index in 0..last {
            let Some(node) = graph.get_node(index) else {
                continue;
            };
            nodes.push(ContextoidRecord::new(node.id(), node.to_record()?));
            let outgoing = graph.get_edges(index).unwrap_or_default();
            edges.extend(outgoing.into_iter().filter_map(|(target, kind)| {
                graph
                    .get_node(target)
                    .map(|to| RelationRecord::new(node.id(), to.id(), *kind))
            }));
        }
        nodes.sort_by_key(ContextoidRecord::id);
        edges.sort_by_key(|edge| (edge.from(), edge.to()));
        Ok((nodes, edges))
    }
}
