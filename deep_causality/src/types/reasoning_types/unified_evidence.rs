/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalityError, ContextId, ContextoidId, IdentificationValue, NumericalValue};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
use ultragraph::{GraphView, UltraGraph};

// This type alias makes the code clearer.
pub type EvidenceGraph = UltraGraph<Evidence>;

/// Generalized evidence container for causal reasoning.
#[derive(Clone, Default)]
pub enum Evidence {
    /// Represents the absence of evidence.
    #[default]
    None,
    /// Represents a simple boolean value.
    Deterministic(bool),
    /// Represents a standard numerical value.
    Numerical(NumericalValue),
    /// Represents a probability value, typically between 0.0 and 1.0.
    Probability(NumericalValue),
    /// A link to a complex, structured result encapsulated in a Contextoid.
    ContextualLink(ContextId, ContextoidId),
    /// A collection of named evidence values, allowing for complex data passing.
    Map(HashMap<IdentificationValue, Evidence>),
    /// A graph of evidence, for passing complex relational data.
    /// Wrapped in an Arc for efficient cloning.
    Graph(Arc<EvidenceGraph>),
}

impl Evidence {
    /// Creates a new empty Evidence Map.
    pub fn new_map() -> Self {
        Evidence::Map(HashMap::new())
    }

    /// Inserts a key-value pair into an Evidence Map.
    /// Panics if the Evidence is not a Map variant.
    pub fn insert(&mut self, key: IdentificationValue, value: Evidence) {
        if let Evidence::Map(map) = self {
            map.insert(key, value);
        } else {
            panic!("Cannot insert into Evidence that is not a Map variant");
        }
    }

    /// Retrieves a numerical value from an Evidence::Map by key.
    pub fn get_numerical_from_map(
        &self,
        key: IdentificationValue,
    ) -> Result<NumericalValue, CausalityError> {
        if let Evidence::Map(map) = self {
            match map.get(&key) {
                Some(Evidence::Numerical(val)) => Ok(*val),
                Some(_) => Err(CausalityError(format!(
                    "Evidence for key '{key}' is not of type Numerical"
                ))),
                None => Err(CausalityError(format!("No evidence found for key '{key}'"))),
            }
        } else {
            Err(CausalityError(
                "Cannot get value by key from Evidence that is not a Map variant".into(),
            ))
        }
    }

    /// Retrieves a deterministic boolean value from an Evidence::Map by key.
    pub fn get_deterministic_from_map(
        &self,
        key: IdentificationValue,
    ) -> Result<bool, CausalityError> {
        if let Evidence::Map(map) = self {
            match map.get(&key) {
                Some(Evidence::Deterministic(val)) => Ok(*val),
                Some(_) => Err(CausalityError(format!(
                    "Evidence for key '{key}' is not of type Deterministic"
                ))),
                None => Err(CausalityError(format!("No evidence found for key '{key}'"))),
            }
        } else {
            Err(CausalityError(
                "Cannot get value by key from Evidence that is not a Map variant".into(),
            ))
        }
    }
}

impl Debug for Evidence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Evidence::None => write!(f, "Evidence::None"),
            Evidence::Deterministic(val) => write!(f, "Evidence::Deterministic({val})"),
            Evidence::Numerical(val) => write!(f, "Evidence::Numerical({val:?})"),
            Evidence::Probability(val) => write!(f, "Evidence::Probability({val:?})"),
            Evidence::ContextualLink(id, val) => write!(f, "Evidence::ContextualLink({id}, {val})"),
            Evidence::Map(map) => {
                write!(f, "Evidence::Map({map:?})")
            }
            Evidence::Graph(g) => write!(
                f,
                "Evidence::Graph(nodes: {}, edges: {})",
                g.number_nodes(),
                g.number_edges()
            ),
        }
    }
}

impl Display for Evidence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // Delegate to the Debug implementation to prevent infinite recursion.
        write!(f, "{:?}", self)
    }
}
