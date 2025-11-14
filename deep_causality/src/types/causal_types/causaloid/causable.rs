/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::causal_types::causaloid::causable_utils;
use crate::{
    Causable, CausableGraph, CausalMonad, CausalityError, Causaloid, CausaloidType, Datable,
    EffectValue, IntoEffectValue, MonadicCausable, MonadicCausableCollection,
    MonadicCausableGraphReasoning, PropagatingEffect, SpaceTemporal, Spatial, Symbolic, Temporal,
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
    fn evaluate(&self, incoming_effect: &PropagatingEffect) -> PropagatingEffect {
        match self.causal_type {
            CausaloidType::Singleton => incoming_effect
                .clone()
                .bind(|effect_val| causable_utils::convert_input(effect_val, self.id))
                .bind(|input| causable_utils::execute_causal_logic(input, self))
                .bind(|output| causable_utils::convert_output(output, self.id)),

            CausaloidType::Collection => {
                // 1. Get an owned copy of the effect and add the initial log.
                let mut initial_monad = incoming_effect.clone();
                initial_monad.logs.add_entry(&format!(
                    "Causaloid {}: Incoming effect for Collection: {:?}",
                    self.id, incoming_effect.value
                ));

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

                // Call the trait method. `causal_collection` now directly implements MonadicCausableCollection.
                causal_collection.evaluate_collection(
                    incoming_effect,
                    // unwrap is save b/c a collection is always initialized with an aggregate_logic
                    &self.coll_aggregate_logic.unwrap(),
                    self.coll_threshold_value,
                )
            }
            CausaloidType::Graph => {
                // 1. Get an owned copy of the effect and add the initial log for this graph-causaloid.
                let mut initial_monad = incoming_effect.clone();
                initial_monad.logs.add_entry(&format!(
                    "Causaloid {}: Incoming effect for Graph: {:?}",
                    self.id, incoming_effect.value
                ));

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
                // The recursive call will handle its own log appending based on its input.
                let mut result_effect =
                    causal_graph.evaluate_subgraph_from_cause(root_index, incoming_effect);

                // 3. Prepend this causaloid's log entry to the results from the subgraph evaluation.
                let mut final_logs = initial_monad.logs;
                final_logs.append(&mut result_effect.logs);
                result_effect.logs = final_logs;

                result_effect
            }
        }
    }
}
