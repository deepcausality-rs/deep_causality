/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextoidId, RelationKind};

/// A stored edge: the two contextoids it joins, by identifier, and the relation it carries.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RelationRecord {
    from: ContextoidId,
    to: ContextoidId,
    kind: RelationKind,
}

impl RelationRecord {
    pub const fn new(from: ContextoidId, to: ContextoidId, kind: RelationKind) -> Self {
        Self { from, to, kind }
    }

    pub const fn from(&self) -> ContextoidId {
        self.from
    }

    pub const fn to(&self) -> ContextoidId {
        self.to
    }

    pub const fn kind(&self) -> RelationKind {
        self.kind
    }
}
