/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ContainerRef, ContextEvent, ContextId, ContextRecord, ContextSnapshot, ContextWrite,
    ContextoidId, ContextoidRecord, ExtraContextSnapshot, MemoryStorageError, NodeRecord,
    RelationKind, RelationRecord,
};
use std::collections::{BTreeMap, BTreeSet};

/// One container: its name, the contextoids it links, and the containers it references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Container {
    pub(crate) name: String,
    pub(crate) links: BTreeSet<ContextoidId>,
    pub(crate) references: BTreeSet<ContextId>,
}

/// The store's state as a fold over `ContextEvent`.
///
/// Every operation validates against the current state, folds the events it emits, and returns
/// them for the log. `fold` alone is the state transition; replaying a log through it rebuilds the
/// state as of any cursor. Ordered maps keep every listing canonical.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct MemoryState {
    next_id: ContextoidId,
    reserved: BTreeSet<ContextoidId>,
    nodes: BTreeMap<ContextoidId, NodeRecord>,
    edges: BTreeMap<(ContextoidId, ContextoidId), RelationKind>,
    containers: BTreeMap<ContextId, Container>,
}

impl MemoryState {
    pub(crate) fn new() -> Self {
        Self {
            next_id: 1,
            reserved: BTreeSet::new(),
            nodes: BTreeMap::new(),
            edges: BTreeMap::new(),
            containers: BTreeMap::new(),
        }
    }

    /// The state transition for one event, with no validation.
    pub(crate) fn fold(&mut self, event: &ContextEvent) {
        match event {
            ContextEvent::ContextCreated(record) => {
                self.containers.insert(
                    record.id(),
                    Container {
                        name: record.name().to_string(),
                        links: BTreeSet::new(),
                        references: BTreeSet::new(),
                    },
                );
            }
            ContextEvent::ContextRetracted(id) => {
                self.containers.remove(id);
                for container in self.containers.values_mut() {
                    container.references.remove(id);
                }
            }
            ContextEvent::NodeCreated(record) => {
                self.reserved.remove(&record.id());
                self.nodes.insert(record.id(), record.node().clone());
            }
            ContextEvent::NodeRetracted(id) => {
                self.nodes.remove(id);
                self.edges.retain(|(from, to), _| from != id && to != id);
                for container in self.containers.values_mut() {
                    container.links.remove(id);
                }
            }
            ContextEvent::EdgeCreated(record) => {
                self.edges
                    .insert((record.from(), record.to()), record.kind());
            }
            ContextEvent::EdgeRetracted { from, to } => {
                self.edges.remove(&(*from, *to));
            }
            ContextEvent::NodeLinked { context, node, .. }
            | ContextEvent::NodeEntered { context, node, .. } => {
                if let Some(container) = self.containers.get_mut(context) {
                    container.links.insert(node.id());
                }
            }
            ContextEvent::NodeUnlinked { context, node }
            | ContextEvent::NodeLeft { context, node } => {
                if let Some(container) = self.containers.get_mut(context) {
                    container.links.remove(node);
                }
            }
            ContextEvent::ContextAttached { context, extra } => {
                if let Some(container) = self.containers.get_mut(context) {
                    container.references.insert(extra.id());
                }
            }
            ContextEvent::ContextDetached { context, extra } => {
                if let Some(container) = self.containers.get_mut(context) {
                    container.references.remove(extra);
                }
            }
        }
    }

    fn fold_all(&mut self, events: &[ContextEvent]) {
        events.iter().for_each(|event| self.fold(event));
    }

    fn container(&self, id: ContextId) -> Result<&Container, MemoryStorageError> {
        self.containers
            .get(&id)
            .ok_or(MemoryStorageError::UnknownContext(id))
    }

    fn node(&self, id: ContextoidId) -> Result<&NodeRecord, MemoryStorageError> {
        self.nodes
            .get(&id)
            .ok_or(MemoryStorageError::UnknownNode(id))
    }

    pub(crate) fn reserve(&mut self, n: usize) -> Vec<ContextoidId> {
        let ids: Vec<ContextoidId> = (0..n as ContextoidId).map(|i| self.next_id + i).collect();
        self.next_id += n as ContextoidId;
        self.reserved.extend(ids.iter().copied());
        ids
    }

    pub(crate) fn create_context(&mut self, name: &str) -> (ContextId, Vec<ContextEvent>) {
        let id = self.next_id;
        self.next_id += 1;
        let events = vec![ContextEvent::ContextCreated(ContextRecord::new(
            id,
            name.to_string(),
        ))];
        self.fold_all(&events);
        (id, events)
    }

    pub(crate) fn retract_context(
        &mut self,
        context: ContextId,
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        self.container(context)?;
        let events = vec![ContextEvent::ContextRetracted(context)];
        self.fold_all(&events);
        Ok(events)
    }

    pub(crate) fn create_node(
        &mut self,
        nodes: &[ContextoidRecord],
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        let mut pending: BTreeMap<ContextoidId, &NodeRecord> = BTreeMap::new();
        let mut events = Vec::new();
        for record in nodes {
            let held = self
                .nodes
                .get(&record.id())
                .or_else(|| pending.get(&record.id()).copied());
            match held {
                Some(held) if held == record.node() => {}
                Some(_) => return Err(MemoryStorageError::NodeConflict(record.id())),
                None if self.reserved.contains(&record.id()) => {
                    pending.insert(record.id(), record.node());
                    events.push(ContextEvent::NodeCreated(record.clone()));
                }
                None => return Err(MemoryStorageError::IdentityNotReserved(record.id())),
            }
        }
        self.fold_all(&events);
        Ok(events)
    }

    pub(crate) fn retract_node(
        &mut self,
        node: ContextoidId,
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        self.node(node)?;
        let events = vec![ContextEvent::NodeRetracted(node)];
        self.fold_all(&events);
        Ok(events)
    }

    pub(crate) fn create_edge(
        &mut self,
        edges: &[RelationRecord],
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        let mut pending: BTreeMap<(ContextoidId, ContextoidId), RelationKind> = BTreeMap::new();
        let mut events = Vec::new();
        for record in edges {
            self.node(record.from())?;
            self.node(record.to())?;
            let key = (record.from(), record.to());
            let held = self.edges.get(&key).or_else(|| pending.get(&key)).copied();
            match held {
                Some(kind) if kind == record.kind() => {}
                Some(_) => {
                    return Err(MemoryStorageError::EdgeConflict(record.from(), record.to()));
                }
                None => {
                    pending.insert(key, record.kind());
                    events.push(ContextEvent::EdgeCreated(*record));
                }
            }
        }
        self.fold_all(&events);
        Ok(events)
    }

    pub(crate) fn retract_edge(
        &mut self,
        from: ContextoidId,
        to: ContextoidId,
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        if !self.edges.contains_key(&(from, to)) {
            return Err(MemoryStorageError::UnknownEdge(from, to));
        }
        let events = vec![ContextEvent::EdgeRetracted { from, to }];
        self.fold_all(&events);
        Ok(events)
    }

    pub(crate) fn link(
        &mut self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        let container = self.container(context)?;
        let mut pending: BTreeSet<ContextoidId> = BTreeSet::new();
        let mut events = Vec::new();
        for id in nodes {
            let record = self.node(*id)?;
            if !container.links.contains(id) && pending.insert(*id) {
                let member = |other: &ContextoidId| {
                    container.links.contains(other) || pending.contains(other)
                };
                events.push(ContextEvent::NodeLinked {
                    context,
                    node: ContextoidRecord::new(*id, record.clone()),
                    edges: self.incident(*id, member),
                });
            }
        }
        self.fold_all(&events);
        Ok(events)
    }

    /// The relations the store holds between `id` and each node for which `member` returns true,
    /// in either direction and ordered by `(from, to)`. `link` sends them as the `edges` of the
    /// `NodeLinked` event it emits for `id`.
    fn incident(
        &self,
        id: ContextoidId,
        member: impl Fn(&ContextoidId) -> bool,
    ) -> Vec<RelationRecord> {
        self.edges
            .iter()
            .filter(|((from, to), _)| (*from == id && member(to)) || (*to == id && member(from)))
            .map(|((from, to), kind)| RelationRecord::new(*from, *to, *kind))
            .collect()
    }

    pub(crate) fn unlink(
        &mut self,
        context: ContextId,
        nodes: &[ContextoidId],
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        let container = self.container(context)?;
        let mut pending: BTreeSet<ContextoidId> = BTreeSet::new();
        let events: Vec<ContextEvent> = nodes
            .iter()
            .filter(|id| container.links.contains(id) && pending.insert(**id))
            .map(|id| ContextEvent::NodeUnlinked { context, node: *id })
            .collect();
        self.fold_all(&events);
        Ok(events)
    }

    pub(crate) fn attach(
        &mut self,
        context: ContextId,
        extra: ContextId,
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        let container = self.container(context)?;
        let referenced = self.container(extra)?;
        if context == extra {
            return Err(MemoryStorageError::SelfReference(context));
        }
        let events = if container.references.contains(&extra) {
            Vec::new()
        } else {
            vec![ContextEvent::ContextAttached {
                context,
                extra: ContextRecord::new(extra, referenced.name.clone()),
            }]
        };
        self.fold_all(&events);
        Ok(events)
    }

    pub(crate) fn detach(
        &mut self,
        context: ContextId,
        extra: ContextId,
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        let container = self.container(context)?;
        let events = if container.references.contains(&extra) {
            vec![ContextEvent::ContextDetached { context, extra }]
        } else {
            Vec::new()
        };
        self.fold_all(&events);
        Ok(events)
    }

    /// Performs every write in order and returns the containers created with the events emitted.
    /// A refusal returns early with the state part-folded, so a caller commits on a clone.
    pub(crate) fn commit(
        &mut self,
        writes: &[ContextWrite],
    ) -> Result<(Vec<ContextId>, Vec<ContextEvent>), MemoryStorageError> {
        let mut created: Vec<ContextId> = Vec::new();
        let mut events = Vec::new();
        let at = |created: &[ContextId], container: &ContainerRef| match container {
            ContainerRef::Held(id) => Ok(*id),
            ContainerRef::Created(index) => created
                .get(*index)
                .copied()
                .ok_or(MemoryStorageError::UnknownCreated(*index)),
        };
        for write in writes {
            let emitted = match write {
                ContextWrite::CreateContext(name) => {
                    let (id, emitted) = self.create_context(name);
                    created.push(id);
                    emitted
                }
                ContextWrite::CreateNode(nodes) => self.create_node(nodes)?,
                ContextWrite::CreateEdge(edges) => self.create_edge(edges)?,
                ContextWrite::Link { context, nodes } => {
                    self.link(at(&created, context)?, nodes)?
                }
                ContextWrite::Attach { context, extra } => {
                    self.attach(at(&created, context)?, at(&created, extra)?)?
                }
            };
            events.extend(emitted);
        }
        Ok((created, events))
    }

    pub(crate) fn lookup(&self, ids: &[ContextoidId]) -> Vec<Option<ContextoidRecord>> {
        ids.iter()
            .map(|id| {
                self.nodes
                    .get(id)
                    .map(|node| ContextoidRecord::new(*id, node.clone()))
            })
            .collect()
    }

    /// A container's members and the edges among them, in canonical order.
    fn members(&self, container: &Container) -> (Vec<ContextoidRecord>, Vec<RelationRecord>) {
        let nodes = container
            .links
            .iter()
            .filter_map(|id| {
                self.nodes
                    .get(id)
                    .map(|node| ContextoidRecord::new(*id, node.clone()))
            })
            .collect();
        let edges = self
            .edges
            .iter()
            .filter(|((from, to), _)| {
                container.links.contains(from) && container.links.contains(to)
            })
            .map(|((from, to), kind)| RelationRecord::new(*from, *to, *kind))
            .collect();
        (nodes, edges)
    }

    pub(crate) fn hydrate(
        &self,
        context: ContextId,
    ) -> Result<ContextSnapshot, MemoryStorageError> {
        let container = self.container(context)?;
        let (nodes, edges) = self.members(container);
        let extras = container
            .references
            .iter()
            .filter_map(|id| self.containers.get(id).map(|referenced| (*id, referenced)))
            .map(|(id, referenced)| {
                let (nodes, edges) = self.members(referenced);
                ExtraContextSnapshot::new(id, referenced.name.clone(), nodes, edges)
            })
            .collect();
        Ok(ContextSnapshot::new(
            ContextRecord::new(context, container.name.clone()),
            nodes,
            edges,
            extras,
        ))
    }

    /// The containers a subscription to `context` covers: the container and what it references.
    pub(crate) fn scope(
        &self,
        context: ContextId,
    ) -> Result<BTreeSet<ContextId>, MemoryStorageError> {
        let container = self.container(context)?;
        let mut scope = container.references.clone();
        scope.insert(context);
        Ok(scope)
    }

    /// Performs the operation an event names, under the same refusals.
    pub(crate) fn apply(
        &mut self,
        event: &ContextEvent,
    ) -> Result<Vec<ContextEvent>, MemoryStorageError> {
        match event {
            ContextEvent::ContextCreated(_) => {
                Err(MemoryStorageError::EventNotApplicable("ContextCreated"))
            }
            ContextEvent::ContextRetracted(id) => self.retract_context(*id),
            ContextEvent::NodeCreated(record) => self.create_node(std::slice::from_ref(record)),
            ContextEvent::NodeRetracted(id) => self.retract_node(*id),
            ContextEvent::EdgeCreated(record) => self.create_edge(std::slice::from_ref(record)),
            ContextEvent::EdgeRetracted { from, to } => self.retract_edge(*from, *to),
            ContextEvent::NodeLinked { context, node, .. } => {
                if self.node(node.id())? != node.node() {
                    return Err(MemoryStorageError::NodeConflict(node.id()));
                }
                self.link(*context, &[node.id()])
            }
            ContextEvent::NodeUnlinked { context, node } => self.unlink(*context, &[*node]),
            ContextEvent::ContextAttached { context, extra } => {
                if self.container(extra.id())?.name != extra.name() {
                    return Err(MemoryStorageError::ContextConflict(extra.id()));
                }
                self.attach(*context, extra.id())
            }
            ContextEvent::ContextDetached { context, extra } => self.detach(*context, *extra),
            ContextEvent::NodeEntered { .. } => {
                Err(MemoryStorageError::EventNotApplicable("NodeEntered"))
            }
            ContextEvent::NodeLeft { .. } => {
                Err(MemoryStorageError::EventNotApplicable("NodeLeft"))
            }
        }
    }
}
