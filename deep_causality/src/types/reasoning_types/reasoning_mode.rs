/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::Display;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ReasoningMode {
    Deterministic,
    Probabilistic,
    ContextualLink,
}

impl Display for ReasoningMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
