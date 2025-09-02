/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{CausalityError, ContextId, ContextoidId, IdentificationValue, NumericalValue};
use deep_causality_uncertain::Uncertain;
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::UltraGraph;

mod debug;
mod display;
mod partial_eq;

// The graph type alias, updated to be recursive on the new unified enum.
pub type EffectGraph = UltraGraph<PropagatingEffect>;

/// Unified data and control-flow container for causal reasoning.
///
/// This enum serves as both the input (evidence) and output (effect) for a causaloid,
/// creating a single, uniform signal that flows through the causal graph. Its variants
/// can represent simple data, complex structures, terminal states, or explicit
/// commands for the reasoning engine.
#[derive(Clone, Default)]
pub enum PropagatingEffect {
    /// Represents the absence of a signal or evidence. Serves as the default.
    #[default]
    None,
    /// Represents a simple boolean value. This effect propagates like any other,
    /// and its interpretation (e.g., whether it prunes a traversal) is left to the
    /// consuming logic or explicit error handling within Causaloids.
    Deterministic(bool),
    /// Represents a standard numerical value.
    Numerical(NumericalValue),
    /// Represents a quantitative outcome, such as a probability score or confidence level.
    Probabilistic(NumericalValue),
    /// Represents a value with inherent uncertainty, modeled as a probability distribution.
    UncertainBool(Uncertain<bool>),
    UncertainFloat(Uncertain<f64>),
    /// A link to a complex, structured result in a Contextoid. As an output, this
    /// can be interpreted by a reasoning engine as a command to fetch data.
    ContextualLink(ContextId, ContextoidId),
    /// A collection of named values, allowing for complex, structured data passing.
    Map(HashMap<IdentificationValue, Box<PropagatingEffect>>),
    /// A graph of effects, for passing complex relational data.
    Graph(Arc<EffectGraph>),
    /// A dispatch command that directs the reasoning engine to dynamically jump to a specific
    /// causaloid within the graph. The `usize` is the target causaloid's index, and the `Box<PropagatingEffect>`
    /// is the effect to be passed as input to that target causaloid. This enables adaptive reasoning.
    RelayTo(usize, Box<PropagatingEffect>),
}

// Predicate methods
impl PropagatingEffect {
    pub fn is_none(&self) -> bool {
        matches!(self, PropagatingEffect::None)
    }
    pub fn is_deterministic(&self) -> bool {
        matches!(self, PropagatingEffect::Deterministic(_))
    }
    pub fn is_numerical(&self) -> bool {
        matches!(self, PropagatingEffect::Numerical(_))
    }
    pub fn is_probabilistic(&self) -> bool {
        matches!(self, PropagatingEffect::Probabilistic(_))
    }
    pub fn is_contextual_link(&self) -> bool {
        matches!(self, PropagatingEffect::ContextualLink(_, _))
    }

    pub fn is_uncertain_bool(&self) -> bool {
        matches!(self, PropagatingEffect::UncertainBool(_))
    }

    pub fn is_uncertain_float(&self) -> bool {
        matches!(self, PropagatingEffect::UncertainFloat(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, PropagatingEffect::Map(_))
    }
    pub fn is_graph(&self) -> bool {
        matches!(self, PropagatingEffect::Graph(_))
    }
    pub fn is_relay_to(&self) -> bool {
        matches!(self, PropagatingEffect::RelayTo(_, _))
    }
}

// Extractor methods
impl PropagatingEffect {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropagatingEffect::Deterministic(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_numerical(&self) -> Option<NumericalValue> {
        match self {
            PropagatingEffect::Numerical(p) => Some(*p),
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

// Map-specific methods
impl PropagatingEffect {
    /// Creates a new empty Effect Map.
    pub fn new_map() -> Self {
        PropagatingEffect::Map(HashMap::new())
    }

    /// Inserts a key-value pair into an Effect Map.
    /// Panics if the Effect is not a Map variant.
    pub fn insert(&mut self, key: IdentificationValue, value: PropagatingEffect) {
        if let PropagatingEffect::Map(map) = self {
            map.insert(key, Box::new(value));
        } else {
            panic!("Cannot insert into PropagatingEffect that is not a Map variant");
        }
    }

    /// Retrieves a numerical value from an Effect::Map by key.
    pub fn get_numerical_from_map(
        &self,
        key: IdentificationValue,
    ) -> Result<NumericalValue, CausalityError> {
        if let PropagatingEffect::Map(map) = self {
            match map.get(&key) {
                Some(effect) => {
                    if let PropagatingEffect::Numerical(val) = **effect {
                        Ok(val)
                    } else {
                        Err(CausalityError(format!(
                            "Effect for key '{key}' is not of type Numerical"
                        )))
                    }
                }
                None => Err(CausalityError(format!("No effect found for key '{key}'"))),
            }
        } else {
            Err(CausalityError(
                "Cannot get value by key from PropagatingEffect that is not a Map variant".into(),
            ))
        }
    }

    /// Retrieves a deterministic boolean value from an Effect::Map by key.
    pub fn get_deterministic_from_map(
        &self,
        key: IdentificationValue,
    ) -> Result<bool, CausalityError> {
        if let PropagatingEffect::Map(map) = self {
            match map.get(&key) {
                Some(effect) => {
                    if let PropagatingEffect::Deterministic(val) = **effect {
                        Ok(val)
                    } else {
                        Err(CausalityError(format!(
                            "Effect for key '{key}' is not of type Deterministic"
                        )))
                    }
                }
                None => Err(CausalityError(format!("No effect found for key '{key}'"))),
            }
        } else {
            Err(CausalityError(
                "Cannot get value by key from PropagatingEffect that is not a Map variant".into(),
            ))
        }
    }
}
