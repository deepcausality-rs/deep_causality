/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{CausalityError, Identifiable, PropagatingEffect};

pub mod causable_reasoning;
mod causable_reasoning_deterministic;
mod causable_reasoning_explain;
mod causable_reasoning_mixed;
mod causable_reasoning_probabilistic;

/// The Causable trait defines the core behavior for all causal elements.
///
/// It requires implementing the Identifiable trait and provides a unified interface
/// for evaluation and state inspection, regardless of whether the element is a single
/// causaloid, a collection, or a graph.
pub trait Causable: Identifiable {
    /// Evaluates the causal element against the provided runtime effect.
    ///
    /// This is the primary method for executing the reasoning logic of a causal element.
    /// - For a `Singleton`, this typically involves executing its `CausalFn`.
    /// - For a `Collection` or `Graph`, this involves aggregating the evaluation
    ///   of its constituent elements.
    ///
    /// # Arguments
    ///
    /// * `effect` - A reference to the `PropagatingEffect` flowing through the causal graph.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `PropagatingEffect` on success, or a `CausalityError`
    /// if the evaluation fails.
    fn evaluate(&self, effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError>;

    /// Generates a human-readable explanation of the causaloid's current state.
    ///
    /// The nature of the explanation depends on the implementor's type:
    /// - For a `Singleton`, it might describe whether the causaloid is active.
    /// - For a `Collection`, it might aggregate the explanations of its members.
    /// - For a `Graph`, it might explain the evaluated causal paths.
    ///
    /// # Returns
    ///
    /// - `Ok(String)` containing the explanation.
    /// - `Err(CausalityError)` if the state cannot be explained (e.g., not yet evaluated).
    fn explain(&self) -> Result<String, CausalityError>;

    /// Determines if the causaloid represents a single, indivisible causal unit.
    ///
    /// This method helps distinguish base-case causaloids from composite structures
    /// like collections or graphs.
    ///
    /// # Returns
    ///
    /// `true` if the implementor is a `Singleton` type, `false` otherwise.
    fn is_singleton(&self) -> bool;
}
