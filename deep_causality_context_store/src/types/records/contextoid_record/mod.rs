/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, NodeRecord};

/// A stored contextoid: its identifier and its payload.
#[derive(Debug, Clone, PartialEq)]
pub struct ContextoidRecord {
    id: ContextoidId,
    node: NodeRecord,
}

impl ContextoidRecord {
    pub const fn new(id: ContextoidId, node: NodeRecord) -> Self {
        Self { id, node }
    }

    pub const fn id(&self) -> ContextoidId {
        self.id
    }

    pub const fn node(&self) -> &NodeRecord {
        &self.node
    }
}
