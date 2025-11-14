/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! This module defines the core behavior of `Causaloid` instances within the DeepCausality framework,
//! specifically how they implement the `Causable` and `MonadicCausable` traits.
//!
//! It details the evaluation logic for different types of `Causaloid`s (Singleton, Collection, Graph),
//! ensuring proper error propagation and comprehensive log provenance through monadic operations.
use crate::types::causal_types::causaloid::causable_utils;
use crate::{
    Causable, CausableGraph, CausalMonad, CausalityError, Causaloid, CausaloidType, Datable,
    EffectValue, IntoEffectValue, MonadicCausable, MonadicCausableCollection,
    MonadicCausableGraphReasoning, PropagatingEffect, SpaceTemporal, Spatial, Symbolic, Temporal,
};

/// Implements the `Causable` trait for `Causaloid`.
///
/// This trait provides fundamental properties and methods for any entity that can
/// participate in a causal relationship. For `Causaloid`, it primarily defines
/// how to determine if a causaloid represents a single, atomic causal unit.
impl<I, O, D, S, T, ST, SYM, VS, VT> Causable for Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue,
    O: IntoEffectValue,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Checks if the `Causaloid` is of type `Singleton`.
    ///
    /// A singleton causaloid represents an atomic causal relationship that
    /// can be evaluated independently.
    ///
    /// # Returns
    /// `true` if the `CausaloidType` is `Singleton`, `false` otherwise.
    fn is_singleton(&self) -> bool {
        matches!(self.causal_type, CausaloidType::Singleton)
    }
}

/// Implements the `MonadicCausable` trait for `Causaloid`.
///
/// This implementation provides the core evaluation logic for `Causaloid`s,
/// leveraging monadic principles to handle the flow of effects, errors, and logs.
/// The evaluation strategy varies based on the `CausaloidType` (Singleton, Collection, or Graph).
#[allow(clippy::type_complexity)]
impl<I, O, D, S, T, ST, SYM, VS, VT> MonadicCausable<CausalMonad>
    for Causaloid<I, O, D, S, T, ST, SYM, VS, VT>
where
    I: IntoEffectValue + Default,
    O: IntoEffectValue + Default,
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Evaluates the causal effect of this `Causaloid` given an `incoming_effect`.
    ///
    /// The evaluation process is monadic, ensuring that errors are propagated
    /// and a comprehensive log of operations is maintained. The specific
    /// evaluation strategy depends on the `CausaloidType`.
    ///
    /// # Arguments
    /// * `incoming_effect` - The `PropagatingEffect` representing the input to this causaloid.
    ///
    /// # Returns
    /// A `PropagatingEffect` containing the result of the causal evaluation,
    /// any errors encountered, and a complete log of the operations performed.
    ///
    /// # Log Provenance
    ///
    /// To meet the strict requirements of auditable and provable reasoning, this
    /// implementation guarantees that the full, ordered history of operations is
    /// preserved in the `logs` field of the returned `PropagatingEffect`. Existing
    /// logs from the `incoming_effect` are always preserved and appended to.
    ///
    /// The mechanism differs slightly by type:
    ///
    /// - **`Singleton`:** Uses a monadic `bind` chain. Each step in the chain
    ///   (input conversion, execution, output conversion) automatically and safely
    ///   appends its logs to the accumulated logs of the previous step.
    ///
    /// - **`Collection`:** Sequentially evaluates each causaloid in the collection.
    ///   The full `PropagatingEffect` (value and all logs) from one evaluation step
    ///   is used as the complete input for the next, ensuring the log history
    ///   grows correctly throughout the sequential chain.
    ///
    /// - **`Graph`:** First, it logs its own evaluation context. It then delegates to a
    ///   recursive subgraph evaluation. Finally, it prepends its own context log to the
    ///   complete log history returned by the subgraph, ensuring the parent-child
    ///   reasoning hierarchy is captured in the final log.
    ///
    /// In all cases, if an error occurs, the evaluation short-circuits and returns
    /// an effect containing the error and all logs accumulated up to the point of failure.
    fn evaluate(&self, incoming_effect: &PropagatingEffect) -> PropagatingEffect {
        match self.causal_type {
            CausaloidType::Singleton => {
                // For a Singleton, the evaluation is a monadic chain of operations:
                // 1. Convert the incoming effect's value to the causaloid's input type.
                // 2. Execute the causal logic with the converted input.
                // 3. Convert the output of the causal logic to the desired effect value.
                // The `bind` operations ensure that logs are aggregated and errors short-circuit.
                incoming_effect
                    .clone()
                    .bind(|effect_val| causable_utils::convert_input(effect_val, self.id))
                    .bind(|input| causable_utils::execute_causal_logic(input, self))
                    .bind(|output| causable_utils::convert_output(output, self.id))
            }

            CausaloidType::Collection => {
                // 1. Get an owned copy of the effect and add an initial log entry
                //    to mark the start of this collection causaloid's evaluation.
                let mut initial_monad = incoming_effect.clone();
                initial_monad.logs.add_entry(&format!(
                    "Causaloid {}: Incoming effect for Collection: {:?}",
                    self.id, incoming_effect.value
                ));

                // Ensure the causal collection exists.
                let causal_collection = match self.causal_coll.as_ref() {
                    Some(coll_arc) => coll_arc.as_ref(), // Get &Vec<Self>
                    None => {
                        let err_msg = "Causaloid::evaluate: causal_collection is None".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs: initial_monad.logs,
                        };
                    }
                };

                // Delegate the evaluation to the `MonadicCausableCollection` trait implementation.
                // This handles the sequential evaluation of causaloids within the collection,
                // aggregating logs and propagating errors.
                causal_collection.evaluate_collection(
                    incoming_effect,
                    // unwrap is safe here because a collection is always initialized with an aggregate_logic
                    &self.coll_aggregate_logic.unwrap(),
                    self.coll_threshold_value,
                )
            }
            CausaloidType::Graph => {
                // 1. Get an owned copy of the effect and add an initial log entry
                //    to mark the start of this graph causaloid's evaluation.
                let mut initial_monad = incoming_effect.clone();
                initial_monad.logs.add_entry(&format!(
                    "Causaloid {}: Incoming effect for Graph: {:?}",
                    self.id, incoming_effect.value
                ));

                // Ensure the causal graph exists.
                let causal_graph = match self.causal_graph.as_ref() {
                    Some(g) => g,
                    None => {
                        let err_msg = "Causaloid::evaluate: Causal graph is None".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs: initial_monad.logs,
                        };
                    }
                };

                // Determine the root node of the graph for evaluation.
                let root_index = match causal_graph.as_ref().get_root_index() {
                    Some(index) => index,
                    None => {
                        let err_msg = "Cannot evaluate graph: Root node not found.".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs: initial_monad.logs,
                        };
                    }
                };

                // 2. Delegate to the subgraph reasoning algorithm.
                // This recursive call will handle its own log appending based on its input.
                let mut result_effect =
                    causal_graph.evaluate_subgraph_from_cause(root_index, incoming_effect);

                // 3. Prepend this causaloid's initial log entry to the results from the
                //    subgraph evaluation. This ensures that the parent-child reasoning
                //    hierarchy is accurately captured in the final log history.
                let mut final_logs = initial_monad.logs;
                final_logs.append(&mut result_effect.logs);
                result_effect.logs = final_logs;

                result_effect
            }
        }
    }
}
