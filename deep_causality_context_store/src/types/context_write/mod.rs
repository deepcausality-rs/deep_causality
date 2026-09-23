/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContainerRef, ContextoidId, ContextoidRecord, RelationRecord};

/// One write of a `ContextStorage::commit`. Each variant is the storage operation of the same
/// name, under the same refusals; a container is named by a `ContainerRef`, so a commit can link
/// into and attach a container it creates.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextWrite {
    /// `create_context` under this name.
    CreateContext(String),
    /// `create_node` of these records.
    CreateNode(Vec<ContextoidRecord>),
    /// `create_edge` of these relations.
    CreateEdge(Vec<RelationRecord>),
    /// `link` of these contextoids into a container.
    Link {
        context: ContainerRef,
        nodes: Vec<ContextoidId>,
    },
    /// `attach` of `extra` to `context`.
    Attach {
        context: ContainerRef,
        extra: ContainerRef,
    },
}
