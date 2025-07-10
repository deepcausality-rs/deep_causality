/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextId, ContextoidId, NumericalValue};
use std::fmt::Display;

/// Unified inference outcome across reasoning modes.
/// Represents the effect of a causaloid's execution and dictates how that
/// effect propagates through the causal graph. This is the core data structure
/// that drives the Effect Propagation Process.
#[derive(Debug, Clone, PartialEq)]
pub enum PropagatingEffect {
    /// A terminal effect representing a definitive boolean outcome.
    /// The propagation typically halts here, especially on a `false` value.
    /// This is the simplest form of a realized effect.
    Deterministic(bool),

    /// A terminal effect representing a final quantitative outcome, such as a
    /// probability score, a confidence level, or a calculated value.
    /// Propagation also halts here.
    Probabilistic(NumericalValue),

    /// A propagating effect that directs the flow of causality.
    /// It instructs the next Causaloid to located the propagation effect
    /// at the linked Contextoid. The linked Contextoid
    /// itself becomes the `Evidence` for the next step in the reasoning process.
    /// This is the primary mechanism for data flow in the graph.
    ContextualLink(ContextId, ContextoidId),

    Halting,
}

impl PropagatingEffect {
    pub fn is_deterministic(&self) -> bool {
        matches!(self, PropagatingEffect::Deterministic(_))
    }

    pub fn is_probabilistic(&self) -> bool {
        matches!(self, PropagatingEffect::Probabilistic(_))
    }

    pub fn is_contextual_link(&self) -> bool {
        matches!(self, PropagatingEffect::ContextualLink(_, _))
    }
    pub fn is_halting(&self) -> bool {
        matches!(self, PropagatingEffect::Halting)
    }
}

impl PropagatingEffect {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropagatingEffect::Deterministic(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_probability(&self) -> Option<NumericalValue> {
        match self {
            PropagatingEffect::Probabilistic(p) => Some(*p),
            _ => None,
        }
    }

    pub fn as_contextual_link(&self) -> Option<(ContextId, ContextoidId)> {
        match self {
            PropagatingEffect::ContextualLink(context_id, contextoid_id) => {
                Some((*context_id, *contextoid_id))
            }
            _ => None,
        }
    }
}

impl Display for PropagatingEffect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PropagatingEffect::Deterministic(b) => write!(f, "Deterministic: {b}"),
            PropagatingEffect::Probabilistic(p) => write!(f, "Probabilistic: {p}"),
            PropagatingEffect::ContextualLink(context_id, contextoid_id) => {
                write!(
                    f,
                    "ContextualLink: {context_id} Contextoid: {contextoid_id}"
                )
            }
            PropagatingEffect::Halting => write!(f, "Halting"),
        }
    }
}
