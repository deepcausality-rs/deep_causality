/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::extra_context::ExtraContext;
use crate::{Context, Contextoid, Datable, RelationKind, SpaceTemporal, Spatial, Temporal};
use deep_causality_context_store::{
    ContextEvent, ContextId, ContextoidId, ContextoidRecord, DataRecord, ProjectionError,
    Recordable, RelationRecord, SpaceRecord, SpaceTimeRecord, TimeRecord,
};
use deep_causality_core::Identifiable;
use ultragraph::{GraphMut, GraphView, UltraGraphWeighted};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> Context<D, S, T, ST>
where
    D: Datable + Clone + Recordable<DataRecord>,
    S: Spatial + Clone + Recordable<SpaceRecord>,
    T: Temporal + Clone + Recordable<TimeRecord>,
    ST: SpaceTemporal + Clone + Recordable<SpaceTimeRecord>,
{
    /// Applies one event from a store, idempotently, routing by the container the event names:
    /// this context's own identifier is the base graph, an extra's identifier is that extra.
    ///
    /// | Event | Effect |
    /// |---|---|
    /// | `NodeCreated`, `ContextCreated` | none |
    /// | `NodeLinked`, `NodeEntered` | add the record to the named graph, then each carried edge whose ends that graph holds; present already: none |
    /// | `NodeUnlinked`, `NodeLeft` | remove from the named graph; absent: none |
    /// | `NodeRetracted` | remove from every graph; absent: none |
    /// | `EdgeCreated` | add to every graph holding both ends; present already: none |
    /// | `EdgeRetracted` | remove from every graph holding it; absent: none |
    /// | `ContextAttached` on this context | add an empty extra under the attached container's identifier and name; present already: none |
    /// | `ContextDetached` on this context | drop that extra; absent: none |
    /// | `ContextAttached`, `ContextDetached` on any other container | none |
    /// | `ContextRetracted` | an extra: drop it; this context: `Identity`; other: none |
    ///
    /// A membership event naming a container this context does not hold is
    /// `ProjectionError::Identity`. An attached container under identifier 0 is refused the same
    /// way. An echo of the host's own write lands on a state it already produced.
    ///
    /// The store assigns container identifiers. An extra created locally, through
    /// `extra_ctx_add_new` or `extra_ctx_add_new_with_id`, is not a container of the store, so a
    /// store event naming its identifier treats it as not held: a membership event is
    /// `Identity`, a detach or retraction is none and the local extra survives, and an attachment
    /// under that identifier is `Identity`, because the store's container and the local graph
    /// would share one identifier.
    pub fn apply(&mut self, event: &ContextEvent) -> Result<(), ProjectionError> {
        match event {
            ContextEvent::NodeCreated(_) | ContextEvent::ContextCreated(_) => Ok(()),
            ContextEvent::NodeLinked {
                context,
                node,
                edges,
            }
            | ContextEvent::NodeEntered {
                context,
                node,
                edges,
            } => {
                self.store_container(*context)?;
                self.hold(*context, node)?;
                let graph = self.graph_mut(*context)?;
                edges.iter().try_for_each(|edge| Self::connect(graph, edge))
            }
            ContextEvent::NodeUnlinked { context, node }
            | ContextEvent::NodeLeft { context, node } => {
                self.store_container(*context)?;
                self.release(*context, *node)
            }
            ContextEvent::NodeRetracted(id) => {
                if self.id_to_index_map.contains_key(id) {
                    self.release(self.id, *id)?;
                }
                let held: Vec<ContextId> = self.extras_holding(*id);
                for extra in held {
                    self.release(extra, *id)?;
                }
                Ok(())
            }
            ContextEvent::EdgeCreated(edge) => self.join(edge),
            ContextEvent::EdgeRetracted { from, to } => self.sever(*from, *to),
            ContextEvent::ContextAttached { context, extra } => {
                if *context != self.id {
                    return Ok(());
                }
                if extra.id() == 0 {
                    return Err(ProjectionError::Identity(
                        0,
                        "an extra context identifier is 0",
                    ));
                }
                let extras = self.extra_contexts.get_or_insert_with(Default::default);
                match extras.get(&extra.id()) {
                    Some(held) if !held.stored => {
                        return Err(ProjectionError::Identity(
                            extra.id(),
                            "an attached container's identifier is held by a local extra context",
                        ));
                    }
                    Some(_) => {}
                    None => {
                        let mut attached = ExtraContext::new(extra.name(), 0);
                        attached.stored = true;
                        extras.insert(extra.id(), attached);
                    }
                }
                self.highest_extra_context_id = self.highest_extra_context_id.max(extra.id());
                Ok(())
            }
            ContextEvent::ContextDetached { context, extra } => {
                if *context == self.id {
                    self.drop_extra(*extra);
                }
                Ok(())
            }
            ContextEvent::ContextRetracted(id) => {
                if *id == self.id {
                    return Err(ProjectionError::Identity(
                        *id,
                        "the held context cannot retract itself",
                    ));
                }
                self.drop_extra(*id);
                Ok(())
            }
        }
    }

    /// The graph a container identifier names, or `Identity` when this context holds none.
    fn graph_mut(
        &mut self,
        context: ContextId,
    ) -> Result<&mut UltraGraphWeighted<Contextoid<D, S, T, ST>, RelationKind>, ProjectionError>
    {
        if context == self.id {
            return Ok(&mut self.base_context);
        }
        self.extra_contexts
            .as_mut()
            .and_then(|extras| extras.get_mut(&context))
            .map(|extra| &mut extra.graph)
            .ok_or(ProjectionError::Identity(
                context,
                "an event names a container this context does not hold",
            ))
    }

    /// Adds the record's node to the named graph; present already: nothing.
    fn hold(
        &mut self,
        context: ContextId,
        record: &ContextoidRecord,
    ) -> Result<(), ProjectionError> {
        let id = record.id();
        let graph = self.graph_mut(context)?;
        if Self::index_of(graph, id).is_some() {
            return Ok(());
        }
        let node = Contextoid::from_record(id, record.node().clone())?;
        let index = graph
            .add_node(node)
            .map_err(|_| ProjectionError::Identity(id, "a node could not be added"))?;
        if context == self.id {
            self.id_to_index_map.insert(id, index);
        }
        Ok(())
    }

    /// Removes the node from the named graph; absent: nothing.
    fn release(&mut self, context: ContextId, id: ContextoidId) -> Result<(), ProjectionError> {
        let graph = self.graph_mut(context)?;
        if let Some(index) = Self::index_of(graph, id) {
            graph
                .remove_node(index)
                .map_err(|_| ProjectionError::Identity(id, "a node could not be removed"))?;
            if context == self.id {
                self.id_to_index_map.remove(&id);
            }
        }
        Ok(())
    }

    /// Adds the edge to every graph holding both ends; present already: nothing.
    fn join(&mut self, edge: &RelationRecord) -> Result<(), ProjectionError> {
        self.graphs_mut()
            .try_for_each(|graph| Self::connect(graph, edge))
    }

    /// Adds the edge to the graph when it holds both ends; either end absent, or the edge
    /// present already: nothing.
    fn connect(
        graph: &mut UltraGraphWeighted<Contextoid<D, S, T, ST>, RelationKind>,
        edge: &RelationRecord,
    ) -> Result<(), ProjectionError> {
        let (Some(a), Some(b)) = (
            Self::index_of(graph, edge.from()),
            Self::index_of(graph, edge.to()),
        ) else {
            return Ok(());
        };
        if graph.contains_edge(a, b) {
            return Ok(());
        }
        graph
            .add_edge(a, b, edge.kind())
            .map_err(|_| ProjectionError::Identity(edge.from(), "an edge could not be added"))
    }

    /// Removes the edge from every graph holding it; absent: nothing.
    fn sever(&mut self, from: ContextoidId, to: ContextoidId) -> Result<(), ProjectionError> {
        for graph in self.graphs_mut() {
            let (Some(a), Some(b)) = (Self::index_of(graph, from), Self::index_of(graph, to))
            else {
                continue;
            };
            if graph.contains_edge(a, b) {
                graph
                    .remove_edge(a, b)
                    .map_err(|_| ProjectionError::Identity(from, "an edge could not be removed"))?;
            }
        }
        Ok(())
    }

    /// Drops the stored extra under `id`, and the current extra with it when it is the one
    /// dropped. A local extra under `id` is not the store's container and survives.
    fn drop_extra(&mut self, id: ContextId) {
        let Some(extras) = self.extra_contexts.as_mut() else {
            return;
        };
        if extras.get(&id).is_some_and(|extra| extra.stored) {
            extras.remove(&id);
            if self.extra_context_id == id {
                self.extra_context_id = 0;
            }
        }
    }

    /// `Ok` when `context` names the base graph or a stored extra; a local extra is not a
    /// container of the store, so an event naming it is `Identity` as for any container not held.
    fn store_container(&self, context: ContextId) -> Result<(), ProjectionError> {
        let local = context != self.id
            && self
                .extra_contexts
                .as_ref()
                .and_then(|extras| extras.get(&context))
                .is_some_and(|extra| !extra.stored);
        if local {
            return Err(ProjectionError::Identity(
                context,
                "an event names a container this context does not hold",
            ));
        }
        Ok(())
    }

    /// The base graph, then every extra's graph.
    fn graphs_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut UltraGraphWeighted<Contextoid<D, S, T, ST>, RelationKind>> {
        core::iter::once(&mut self.base_context).chain(
            self.extra_contexts
                .iter_mut()
                .flat_map(|extras| extras.values_mut().map(|extra| &mut extra.graph)),
        )
    }

    /// Every extra holding a node under `id`.
    fn extras_holding(&self, id: ContextoidId) -> Vec<ContextId> {
        self.extra_contexts
            .iter()
            .flat_map(|extras| extras.iter())
            .filter(|(_, extra)| Self::index_of(&extra.graph, id).is_some())
            .map(|(context, _)| *context)
            .collect()
    }

    /// The index of the live node carrying `id`, by a scan of the graph.
    fn index_of(
        graph: &UltraGraphWeighted<Contextoid<D, S, T, ST>, RelationKind>,
        id: ContextoidId,
    ) -> Option<usize> {
        let last = graph.get_last_index().map_or(0, |last| last + 1);
        (0..last).find(|&index| graph.get_node(index).is_some_and(|node| node.id() == id))
    }
}
