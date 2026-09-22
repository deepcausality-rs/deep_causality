/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextId, ContextoidRecord, RelationRecord};

/// A context a stored context references, materialised: the referenced container's identifier
/// and name, its nodes, and the edges among them.
#[derive(Debug, Clone, PartialEq)]
pub struct ExtraContextSnapshot {
    id: ContextId,
    name: String,
    nodes: Vec<ContextoidRecord>,
    edges: Vec<RelationRecord>,
}

impl ExtraContextSnapshot {
    pub fn new(
        id: ContextId,
        name: String,
        nodes: Vec<ContextoidRecord>,
        edges: Vec<RelationRecord>,
    ) -> Self {
        Self {
            id,
            name,
            nodes,
            edges,
        }
    }

    pub const fn id(&self) -> ContextId {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn nodes(&self) -> &[ContextoidRecord] {
        &self.nodes
    }

    pub fn edges(&self) -> &[RelationRecord] {
        &self.edges
    }

    /// The four parts, for a reader that builds a graph from them.
    pub fn into_parts(
        self,
    ) -> (
        ContextId,
        String,
        Vec<ContextoidRecord>,
        Vec<RelationRecord>,
    ) {
        (self.id, self.name, self.nodes, self.edges)
    }
}
