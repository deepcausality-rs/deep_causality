// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{NumericalValue, SymbolicResult};
use std::fmt::Display;

/// Unified inference outcome across reasoning modes.
#[derive(Debug, Clone, PartialEq)]
pub enum ReasoningOutcome {
    Deterministic(bool),
    Probabilistic(NumericalValue), // e.g., probability score
    Symbolic(SymbolicResult),
}

impl ReasoningOutcome {
    pub fn is_deterministic(&self) -> bool {
        match self {
            ReasoningOutcome::Deterministic(b) => *b,
            _ => false,
        }
    }

    pub fn is_probabilistic(&self) -> bool {
        match self {
            ReasoningOutcome::Probabilistic(_) => true,
            _ => false,
        }
    }

    pub fn is_symbolic(&self) -> bool {
        match self {
            ReasoningOutcome::Symbolic(_) => true,
            _ => false,
        }
    }
}

impl ReasoningOutcome {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ReasoningOutcome::Deterministic(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_probability(&self) -> Option<NumericalValue> {
        match self {
            ReasoningOutcome::Probabilistic(p) => Some(*p),
            _ => None,
        }
    }

    pub fn as_symbolic(&self) -> Option<SymbolicResult> {
        match self {
            ReasoningOutcome::Symbolic(s) => Some(*s),
            _ => None,
        }
    }
}

impl Display for ReasoningOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasoningOutcome::Deterministic(b) => write!(f, "{}", b),
            ReasoningOutcome::Probabilistic(p) => write!(f, "{}", p),
            ReasoningOutcome::Symbolic(s) => write!(f, "{}", s),
        }
    }
}
