/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ContextId, ContextoidId, IdentificationValue, NumericalValue};
use deep_causality_num::Complex;
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;
use std::sync::Arc;
use ultragraph::UltraGraph;

mod constructors;
mod debug;
mod display;
mod extractors;
mod extractors_map;
mod partial_eq;
mod predicates;

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
    /// Represents a Tensor via Causal Tensor.
    /// Note, when you import the  CausalTensorWitness from the deep_causality_tensor crate,
    /// you can apply monadic composition and monadic transformation to tensors.
    Tensor(CausalTensor<f64>),
    /// Represents a Tensor over complex numbers via Causal Tensor.
    /// Note, when you import the  CausalTensorWitness from the deep_causality_tensor crate,
    /// you can apply monadic composition and monadic transformation to complex tensors.
    ComplexTensor(CausalTensor<Complex<f64>>),
    /// Represents a value with inherent uncertainty, modeled as a probability distribution.
    UncertainBool(UncertainBool),
    UncertainFloat(UncertainF64),
    /// Represents a value that is probabilistic present or absent with uncertainty when present
    MaybeUncertainBool(MaybeUncertainBool),
    MaybeUncertainFloat(MaybeUncertainF64),
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

// Update predicates, extractors, and debug in case of changing field types.
