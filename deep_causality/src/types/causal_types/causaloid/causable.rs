/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::causal_types::causaloid::causable_utils;
use crate::{
    Causable, CausableGraph, CausalMonad, CausalityError, Causaloid, CausaloidRegistry,
    CausaloidType, Datable, EffectValue, IntoEffectValue, MonadicCausable,
    MonadicCausableGraphReasoning, PropagatingEffect, SpaceTemporal, Spatial, Symbolic, Temporal,
    monadic_collection_utils,
};

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
    fn is_singleton(&self) -> bool {
        matches!(self.causal_type, CausaloidType::Singleton)
    }
}

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
    /// Evaluates the causaloid against a given incoming effect, producing a new effect.
    ///
    /// This function is the core of the causal reasoning engine for a single causaloid.
    /// It dispatches the evaluation logic based on the `CausaloidType` (Singleton,
    /// Collection, or Graph) and returns the resulting `PropagatingEffect`.
    ///
    /// # Arguments
    ///
    /// * `registry`: A reference to the `CausaloidRegistry` used to look up
    ///   other causaloids during collection or graph evaluation.
    /// * `incoming_effect`: The `PropagatingEffect` that triggers this evaluation.
    ///
    /// # Returns
    ///
    /// A new `PropagatingEffect` containing the value, error status, and full
    /// log history of the computation.
    ///
    /// ## Log Provenance
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
    fn evaluate(
        &self,
        registry: &CausaloidRegistry,
        incoming_effect: &PropagatingEffect,
    ) -> PropagatingEffect {
        match self.causal_type {
            CausaloidType::Singleton => {
                // 1. Get an owned copy of the effect.
                let mut initial_monad = incoming_effect.clone();

                // 2. Add the new, contextual log message while preserving the exiting logs.
                initial_monad.logs.add_entry(&format!(
                    "Causaloid {}: Incoming effect: {:?}",
                    self.id, incoming_effect.value
                ));

                // 3. Chain the operations and return the final monad.
                initial_monad
                    .bind(|effect_val| causable_utils::convert_input(effect_val, self.id))
                    .bind(|input| causable_utils::execute_causal_logic(input, self))
                    .bind(|output| causable_utils::convert_output(output, self.id))
            }
            CausaloidType::Collection => {
                // 1. Get an owned copy of the effect and add the initial log.
                let mut initial_monad = incoming_effect.clone();
                initial_monad.logs.add_entry(&format!(
                    "Causaloid {}: Incoming effect for Collection: {:?}",
                    self.id, incoming_effect.value
                ));

                let coll_ids = match self.causal_coll.as_ref() {
                    Some(c) => c,
                    None => {
                        let err_msg = "Causaloid::evaluate: causal_collection is None".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs: initial_monad.logs,
                        };
                    }
                };

                let aggregate_logic = match self.coll_aggregate_logic {
                    Some(c) => c,
                    None => {
                        let err_msg =
                            "Causaloid::evaluate: aggregate_logic for causal collection is None"
                                .into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs: initial_monad.logs,
                        };
                    }
                };

                // Sequentially evaluate each causaloid, accumulating logs and chaining effects.
                let mut effects_from_collection = Vec::new();
                // Start the chain with the initial effect, which contains the full log history up to this point.
                let mut current_effect = initial_monad.clone();

                for &causaloid_id in coll_ids.iter() {
                    // The evaluate function will return a new effect with the combined logs.
                    let result_effect = registry.evaluate(causaloid_id, &current_effect);

                    // Short-circuit on error. The result_effect already contains the full log history.
                    if result_effect.is_err() {
                        return result_effect;
                    }

                    effects_from_collection.push(result_effect.value.clone());
                    // The full effect (value and logs) from this step becomes the input for the next.
                    current_effect = result_effect;
                }

                // Aggregate the collected effect values.
                match monadic_collection_utils::aggregate_effects(
                    effects_from_collection,
                    &aggregate_logic,
                    self.coll_threshold_value,
                ) {
                    Ok(aggregated_value) => PropagatingEffect {
                        value: aggregated_value,
                        error: None,
                        // Use the logs from the very last successful evaluation, which contains the full history.
                        logs: current_effect.logs,
                    },
                    Err(e) => PropagatingEffect {
                        value: EffectValue::None,
                        error: Some(e),
                        logs: current_effect.logs,
                    },
                }
            }
            CausaloidType::Graph => {
                // 1. Get an owned copy of the effect and add the initial log for this graph-causaloid.
                let mut initial_monad = incoming_effect.clone();
                initial_monad.logs.add_entry(&format!(
                    "Causaloid {}: Incoming effect for Graph: {:?}",
                    self.id, incoming_effect.value
                ));

                let graph_ids = match self.causal_graph.as_ref() {
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

                let root_index = match graph_ids.as_ref().get_root_index() {
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
                // The recursive call will handle its own log appending based on its input.
                let mut result_effect =
                    graph_ids.evaluate_subgraph_from_cause(registry, root_index, incoming_effect);

                // 3. Prepend this causaloid's log entry to the results from the subgraph evaluation.
                let mut final_logs = initial_monad.logs;
                final_logs.append(&mut result_effect.logs);
                result_effect.logs = final_logs;

                result_effect
            }
        }
    }
}
