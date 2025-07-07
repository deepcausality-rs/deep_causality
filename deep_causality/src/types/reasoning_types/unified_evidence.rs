/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextId, ContextoidId, NumericalValue};
use std::fmt::Display;

/// Generalized evidence container for causal reasoning.
#[derive(Debug, Clone, PartialEq)]
pub enum Evidence {
    Deterministic(bool),
    Numerical(NumericalValue),
    Probability(NumericalValue),
    /// A link to a complex, structured result encapsulated in a Contextoid.
    /// The values are the (ContextId, ContextoidId) used for lookup.
    ContextualLink(ContextId, ContextoidId),
}

impl Display for Evidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
