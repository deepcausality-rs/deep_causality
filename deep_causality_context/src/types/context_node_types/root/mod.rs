/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ContextoidId;
use std::fmt::{Display, Formatter};

use deep_causality_core::Identifiable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Root {
    id: ContextoidId,
}

impl Root {
    pub fn new(id: ContextoidId) -> Self {
        Self { id }
    }
}

impl Identifiable for Root {
    fn id(&self) -> ContextoidId {
        self.id
    }
}

impl Display for Root {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Root ID: {}", self.id,)
    }
}
