/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::prelude::{NumericalValue, SymbolicRepresentation};
use std::fmt::Display;

/// Generalized evidence container for causal reasoning.
#[derive(Debug, Clone, PartialEq)]
pub enum Evidence {
    Deterministic(bool),
    Numerical(NumericalValue),
    Symbolic(SymbolicRepresentation),
    Probability(NumericalValue), // often the same as Numerical, but semantically distinct
}

impl Display for Evidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
