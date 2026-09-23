/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ContextRecord, ContextoidRecord, ExtraContextSnapshot, RECORD_VERSION, RelationRecord,
};

/// One context as a store returns it: the container, its nodes, the edges among them, and the
/// contexts it references, materialised one level deep.
///
/// Every reference between parts is by identifier, never by graph index, because an index is a
/// property of one in-memory graph and an identifier is a property of the node.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextSnapshot {
    version: u16,
    context: ContextRecord,
    nodes: Vec<ContextoidRecord>,
    edges: Vec<RelationRecord>,
    extras: Vec<ExtraContextSnapshot>,
}

impl ContextSnapshot {
    /// A snapshot at the current [`RECORD_VERSION`].
    pub fn new(
        context: ContextRecord,
        nodes: Vec<ContextoidRecord>,
        edges: Vec<RelationRecord>,
        extras: Vec<ExtraContextSnapshot>,
    ) -> Self {
        Self::with_version(RECORD_VERSION, context, nodes, edges, extras)
    }

    /// A snapshot under an explicit version, for a backend reading one it stored earlier.
    pub fn with_version(
        version: u16,
        context: ContextRecord,
        nodes: Vec<ContextoidRecord>,
        edges: Vec<RelationRecord>,
        extras: Vec<ExtraContextSnapshot>,
    ) -> Self {
        Self {
            version,
            context,
            nodes,
            edges,
            extras,
        }
    }

    pub const fn version(&self) -> u16 {
        self.version
    }

    pub const fn context(&self) -> &ContextRecord {
        &self.context
    }

    pub fn nodes(&self) -> &[ContextoidRecord] {
        &self.nodes
    }

    pub fn edges(&self) -> &[RelationRecord] {
        &self.edges
    }

    pub fn extras(&self) -> &[ExtraContextSnapshot] {
        &self.extras
    }

    /// The five parts, for a reader that builds a context from them.
    #[allow(clippy::type_complexity)]
    pub fn into_parts(
        self,
    ) -> (
        u16,
        ContextRecord,
        Vec<ContextoidRecord>,
        Vec<RelationRecord>,
        Vec<ExtraContextSnapshot>,
    ) {
        (
            self.version,
            self.context,
            self.nodes,
            self.edges,
            self.extras,
        )
    }
}
