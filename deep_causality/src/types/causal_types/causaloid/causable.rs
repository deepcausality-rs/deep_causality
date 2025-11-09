/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::traits::causable_collection::collection_reasoning::private;
use crate::{
    Causable, CausableGraph, CausalEffectLog, CausalMonad, CausalityError, Causaloid,
    CausaloidRegistry, CausaloidType, Datable, EffectValue, IntoEffectValue, MonadicCausable,
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
    fn evaluate(
        &self,
        registry: &CausaloidRegistry,
        incoming_effect: &PropagatingEffect,
    ) -> PropagatingEffect {
        // Log the incoming effect
        let mut logs: Vec<CausalEffectLog> = vec![vec![format!(
            "Causaloid {}: Incoming effect: {:?}",
            self.id, incoming_effect.value
        )]];

        match self.causal_type {
            CausaloidType::Singleton => {
                // 1. Convert incoming_effect.value to input type I
                let input_value = match I::try_from_effect_value(incoming_effect.value.clone()) {
                    Ok(val) => val,
                    Err(e) => {
                        logs.push(vec![format!(
                            "Causaloid {}: Input conversion failed: {}",
                            self.id, e
                        )]);
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(e),
                            logs,
                        };
                    }
                };

                let result_o = if let Some(context_fn) = self.context_causal_fn {
                    if let Some(context) = self.context.as_ref() {
                        context_fn(input_value, context)
                    } else {
                        let err_msg = "Causaloid::evaluate: context is None".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs,
                        };
                    }
                } else if let Some(causal_fn) = self.causal_fn {
                    causal_fn(input_value)
                } else {
                    let err_msg = format!(
                        "Causaloid {} is missing both causal_fn and context_causal_fn",
                        self.id
                    );
                    logs.push(vec![err_msg.clone()]);
                    return PropagatingEffect {
                        value: EffectValue::None,
                        error: Some(CausalityError(err_msg)),
                        logs,
                    };
                };

                match result_o {
                    Ok(output_value) => {
                        let effect_value = output_value.into_effect_value();
                        logs.push(vec![format!(
                            "Causaloid {}: Output effect: {:?}",
                            self.id, effect_value
                        )]);
                        PropagatingEffect {
                            value: effect_value,
                            error: None,
                            logs,
                        }
                    }
                    Err(e) => {
                        logs.push(vec![format!(
                            "Causaloid {}: Causal function failed: {}",
                            self.id, e
                        )]);
                        PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(e),
                            logs,
                        }
                    }
                }
            }
            CausaloidType::Collection => {
                let coll_ids = match self.causal_coll.as_ref() {
                    Some(c) => c,
                    None => {
                        let err_msg = "Causaloid::evaluate: causal_collectin is None".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs,
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
                            logs,
                        };
                    }
                };

                let threshold_value = self.coll_threshold_value;

                // Evaluate each causaloid in the collection using the registry
                let mut effects_from_collection = Vec::new();
                let mut current_input_effect = incoming_effect.clone();

                for &causaloid_id in coll_ids.iter() {
                    let mut result_effect = registry.evaluate(causaloid_id, &current_input_effect);
                    logs.append(&mut result_effect.logs); // Merge logs
                    if result_effect.is_err() {
                        result_effect.logs = logs; // Attach accumulated logs before returning error
                        return result_effect;
                    }
                    effects_from_collection.push(result_effect.value.clone());
                    current_input_effect = result_effect; // Output of one is input to next
                }

                // Aggregate the effects
                match private::aggregate_effects(
                    effects_from_collection,
                    &aggregate_logic,
                    threshold_value,
                ) {
                    Ok(aggregated_value) => PropagatingEffect {
                        value: aggregated_value,
                        error: None,
                        logs,
                    },
                    Err(e) => PropagatingEffect {
                        value: EffectValue::None,
                        error: Some(e),
                        logs,
                    },
                }
            }
            CausaloidType::Graph => {
                let graph_ids = match self.causal_graph.as_ref() {
                    Some(g) => g,
                    None => {
                        let err_msg = "Causaloid::evaluate: Causal graph is None".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs,
                        };
                    }
                };

                // Since the graph is guaranteed to have a single root, start evaluation there.
                let root_index = match graph_ids.as_ref().get_root_index() {
                    Some(index) => index,
                    None => {
                        let err_msg = "Cannot evaluate graph: Root node not found.".into();
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(err_msg)),
                            logs,
                        };
                    }
                };

                // Delegate to the reasoning algorithm from the `MonadicCausableGraphReasoning` trait.
                // This will traverse and evaluate the entire graph from the root using the registry.
                let mut graph_effect =
                    graph_ids.evaluate_subgraph_from_cause(registry, root_index, incoming_effect);
                logs.append(&mut graph_effect.logs); // Merge logs
                graph_effect.logs = logs; // Replace with merged logs
                graph_effect
            }
        }
    }
}
