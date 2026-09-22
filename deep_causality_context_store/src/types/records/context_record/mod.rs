/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextId;

/// A stored context: its identifier and its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRecord {
    id: ContextId,
    name: String,
}

impl ContextRecord {
    pub fn new(id: ContextId, name: String) -> Self {
        Self { id, name }
    }

    pub const fn id(&self) -> ContextId {
        self.id
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}
