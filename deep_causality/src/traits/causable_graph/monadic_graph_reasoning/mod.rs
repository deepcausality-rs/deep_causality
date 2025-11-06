/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Causable, CausableGraph, CausalMonad, Identifiable, MonadicCausable, PropagatingEffect,
};

/// Provides default implementations for monadic reasoning over `CausableGraph` items.
///
/// Any graph type that implements `CausableGraph<T>` where `T` is `MonadicCausable<CausalMonad>`
/// will automatically gain a suite of useful default methods for monadic evaluation.
pub trait MonadicCausableGraphReasoning<T>: CausableGraph<T>
where
    T: MonadicCausable<CausalMonad> + Causable + Identifiable + PartialEq + Clone,
{
    /// Evaluates a graph of `MonadicCausable` items, aggregating their monadic effects.
    ///
    /// # Arguments
    /// * `incoming_effect` - A `PropagatingEffect` to be passed to the root `MonadicCausable` item.
    ///
    /// # Returns
    /// A `PropagatingEffect` representing the aggregated monadic effect of the graph.
    fn evaluate_graph(&self, incoming_effect: PropagatingEffect) -> PropagatingEffect;
}
