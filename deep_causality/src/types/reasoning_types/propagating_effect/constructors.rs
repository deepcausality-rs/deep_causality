/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    ComplexTensor, ContextId, ContextoidId, EffectGraph, IdentificationValue, NumericalValue,
    PropagatingEffect,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_uncertain::{
    MaybeUncertainBool, MaybeUncertainF64, UncertainBool, UncertainF64,
};
use std::collections::HashMap;
use std::sync::Arc;

// Constructors
impl PropagatingEffect {
    /// Creates a new `PropagatingEffect` of the `Deterministic` variant.
    ///
    /// # Arguments
    ///
    /// * `deterministic` - A boolean value representing the deterministic effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Deterministic` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect = PropagatingEffect::from_deterministic(true);
    /// assert!(matches!(effect, PropagatingEffect::Deterministic(true)));
    /// ```
    pub fn from_deterministic(deterministic: bool) -> Self {
        Self::Deterministic(deterministic)
    }

    /// Creates a new `PropagatingEffect` of the `Numerical` variant.
    ///
    /// # Arguments
    ///
    /// * `numerical` - A `NumericalValue` representing the numerical effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Numerical` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect = PropagatingEffect::from_numerical(123.45);
    /// assert!(matches!(effect, PropagatingEffect::Numerical(123.45)));
    /// ```
    pub fn from_numerical(numerical: NumericalValue) -> Self {
        Self::Numerical(numerical)
    }

    /// Creates a new `PropagatingEffect` of the `Probabilistic` variant.
    ///
    /// # Arguments
    ///
    /// * `numerical` - A `NumericalValue` representing the probabilistic effect (e.g., a probability score).
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Probabilistic` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect = PropagatingEffect::from_probabilistic(0.75);
    /// assert!(matches!(effect, PropagatingEffect::Probabilistic(0.75)));
    /// ```
    pub fn from_probabilistic(numerical: NumericalValue) -> Self {
        Self::Probabilistic(numerical)
    }

    /// Creates a new `PropagatingEffect` of the `Tensor` variant.
    ///
    /// # Arguments
    ///
    /// * `tensor` - A `CausalTensor<f64>` representing the tensor effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Tensor` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let tensor = CausalTensor::new(vec![1.0, 2.0, 3.0], vec![3])?;
    ///     let effect = PropagatingEffect::from_tensor(tensor.clone());
    ///     assert!(matches!(effect, PropagatingEffect::Tensor(_)));
    ///     Ok(())
    /// }
    /// ```
    pub fn from_tensor(tensor: CausalTensor<f64>) -> Self {
        Self::Tensor(tensor)
    }

    /// Creates a new `PropagatingEffect` of the `ComplexTensor` variant.
    ///
    /// # Arguments
    ///
    /// * `complex_tensor` - A `ComplexTensor` representing the complex tensor effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::ComplexTensor` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_num::Complex;
    /// use deep_causality_tensor::CausalTensor;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let complex_tensor = CausalTensor::new(vec![Complex::new(1.0, 2.0)], vec![1])?;
    ///     let effect = PropagatingEffect::from_complex_tensor(complex_tensor.clone());
    ///     assert!(matches!(effect, PropagatingEffect::ComplexTensor(_)));
    ///     Ok(())
    /// }
    /// ```
    pub fn from_complex_tensor(complex_tensor: ComplexTensor) -> Self {
        Self::ComplexTensor(complex_tensor)
    }

    /// Creates a new `PropagatingEffect` of the `UncertainBool` variant.
    ///
    /// # Arguments
    ///
    /// * `uncertain` - An `UncertainBool` value representing the uncertain boolean effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::UncertainBool` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::UncertainBool;
    ///
    /// let uncertain_bool = UncertainBool::point(true);
    /// let effect = PropagatingEffect::from_uncertain_bool(uncertain_bool.clone());
    /// assert!(matches!(effect, PropagatingEffect::UncertainBool(_)));
    /// ```
    pub fn from_uncertain_bool(uncertain: UncertainBool) -> Self {
        Self::UncertainBool(uncertain)
    }

    /// Creates a new `PropagatingEffect` of the `UncertainFloat` variant.
    ///
    /// # Arguments
    ///
    /// * `uncertain` - An `UncertainF64` value representing the uncertain float effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::UncertainFloat` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::UncertainF64;
    ///
    /// let uncertain_float = UncertainF64::point(1.0);
    /// let effect = PropagatingEffect::from_uncertain_float(uncertain_float.clone());
    /// assert!(matches!(effect, PropagatingEffect::UncertainFloat(_)));
    /// ```
    pub fn from_uncertain_float(uncertain: UncertainF64) -> Self {
        Self::UncertainFloat(uncertain)
    }

    /// Creates a new `PropagatingEffect` of the `MaybeUncertainBool` variant.
    ///
    /// # Arguments
    ///
    /// * `maybe_uncertain` - A `MaybeUncertainBool` value representing the possibly uncertain boolean effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::MaybeUncertainBool` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::MaybeUncertainBool;
    ///
    /// let maybe_uncertain_bool = MaybeUncertainBool::from_value(true);
    /// let effect = PropagatingEffect::from_maybe_uncertain_bool(maybe_uncertain_bool.clone());
    /// assert!(matches!(effect, PropagatingEffect::MaybeUncertainBool(_)));
    /// ```
    pub fn from_maybe_uncertain_bool(maybe_uncertain: MaybeUncertainBool) -> Self {
        Self::MaybeUncertainBool(maybe_uncertain)
    }

    /// Creates a new `PropagatingEffect` of the `MaybeUncertainFloat` variant.
    ///
    /// # Arguments
    ///
    /// * `maybe_uncertain` - A `MaybeUncertainF64` value representing the possibly uncertain float effect.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::MaybeUncertainFloat` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality_uncertain::MaybeUncertainF64;
    ///
    /// let maybe_uncertain_float = MaybeUncertainF64::from_value(1.0);
    /// let effect = PropagatingEffect::from_maybe_uncertain_float(maybe_uncertain_float.clone());
    /// assert!(matches!(effect, PropagatingEffect::MaybeUncertainFloat(_)));
    /// ```
    pub fn from_maybe_uncertain_float(maybe_uncertain: MaybeUncertainF64) -> Self {
        Self::MaybeUncertainFloat(maybe_uncertain)
    }

    /// Creates a new `PropagatingEffect` of the `ContextualLink` variant.
    ///
    /// # Arguments
    ///
    /// * `context_id` - The `ContextId` of the context.
    /// * `contextoid_id` - The `ContextoidId` of the linked contextoid.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::ContextualLink` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality::{ContextId, ContextoidId};
    ///
    /// let context_id = 1u64;
    /// let contextoid_id = 2u64;
    /// let effect = PropagatingEffect::from_contextual_link(context_id, contextoid_id);
    /// assert!(matches!(effect, PropagatingEffect::ContextualLink(_, _)));
    /// ```
    pub fn from_contextual_link(context_id: ContextId, contextoid_id: ContextoidId) -> Self {
        Self::ContextualLink(context_id, contextoid_id)
    }

    /// Creates a new empty `PropagatingEffect` of the `Map` variant.
    ///
    /// # Returns
    ///
    /// An empty `PropagatingEffect::Map` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use std::collections::HashMap;
    ///
    /// let effect = PropagatingEffect::new_map();
    /// assert!(matches!(effect, PropagatingEffect::Map(_)));
    /// if let PropagatingEffect::Map(map) = effect {
    ///     assert!(map.is_empty());
    /// }
    /// ```
    pub fn new_map() -> Self {
        Self::Map(HashMap::new())
    }

    /// Creates a new `PropagatingEffect` of the `Map` variant from an existing `HashMap`.
    ///
    /// # Arguments
    ///
    /// * `map` - A `HashMap` containing `IdentificationValue` keys and boxed `PropagatingEffect` values.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Map` instance initialized with the given map.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    /// use deep_causality::IdentificationValue;
    /// use std::collections::HashMap;
    ///
    /// let mut initial_map = HashMap::new();
    /// initial_map.insert(IdentificationValue::from(1u64), Box::new(PropagatingEffect::Numerical(1.0)));
    /// let effect = PropagatingEffect::from_map(initial_map);
    /// assert!(matches!(effect, PropagatingEffect::Map(_)));
    /// ```
    pub fn from_map(map: HashMap<IdentificationValue, Box<PropagatingEffect>>) -> Self {
        Self::Map(map)
    }

    /// Creates a new empty `PropagatingEffect` of the `Graph` variant.
    ///
    /// # Returns
    ///
    /// An empty `PropagatingEffect::Graph` instance wrapped in an `Arc`.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect = PropagatingEffect::new_graph();
    /// assert!(matches!(effect, PropagatingEffect::Graph(_)));
    /// ```
    pub fn new_graph() -> Self {
        Self::Graph(Arc::new(EffectGraph::new()))
    }

    /// Creates a new `PropagatingEffect` of the `Graph` variant from an existing `EffectGraph`.
    ///
    /// # Arguments
    ///
    /// * `graph` - An `Arc` wrapped `EffectGraph`.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::Graph` instance initialized with the given graph.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::{PropagatingEffect, EffectGraph};
    /// use std::sync::Arc;
    /// use ultragraph::UltraGraph;
    ///
    /// let graph = Arc::new(EffectGraph::new());
    /// let effect = PropagatingEffect::from_graph(graph.clone());
    /// assert!(matches!(effect, PropagatingEffect::Graph(_)));
    /// ```
    pub fn from_graph(graph: Arc<EffectGraph>) -> Self {
        Self::Graph(graph)
    }

    /// Creates a new `PropagatingEffect` of the `RelayTo` variant.
    ///
    /// This variant is used to dispatch a command that directs the reasoning engine to
    /// dynamically jump to a specific causaloid within the graph, passing an effect as input.
    ///
    /// # Arguments
    ///
    /// * `id` - The `usize` index of the target causaloid.
    /// * `effect` - A `Box<PropagatingEffect>` representing the effect to be passed as input to the target causaloid.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect::RelayTo` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use deep_causality::PropagatingEffect;
    ///
    /// let effect_to_relay = Box::new(PropagatingEffect::Numerical(10.0));
    /// let effect = PropagatingEffect::from_relay_to(5, effect_to_relay);
    /// assert!(matches!(effect, PropagatingEffect::RelayTo(5, _)));
    /// ```
    pub fn from_relay_to(id: usize, effect: Box<PropagatingEffect>) -> Self {
        Self::RelayTo(id, effect)
    }
}
