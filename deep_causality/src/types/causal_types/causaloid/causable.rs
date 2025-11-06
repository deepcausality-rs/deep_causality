/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    Causable, CausalMonad, CausalityError, Causaloid, CausaloidType, Datable, EffectValue,
    MonadicCausable, MonadicCausableCollection, MonadicCausableGraphReasoning, PropagatingEffect,
    SpaceTemporal, Spatial, Symbolic, Temporal,
};

impl<D, S, T, ST, SYM, VS, VT> Causable for Causaloid<D, S, T, ST, SYM, VS, VT>
where
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
impl<D, S, T, ST, SYM, VS, VT> MonadicCausable<CausalMonad> for Causaloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn evaluate_monadic(&self, incoming_effect: PropagatingEffect) -> PropagatingEffect {
        match self.causal_type {
            CausaloidType::Singleton => {
                // Check if a context-aware function should be used
                if let Some(context_fn) = self.context_causal_fn {
                    let context = match self.context.as_ref() {
                        Some(c) => c,
                        None => {
                            return PropagatingEffect {
                                value: EffectValue::None,
                                error: Some(CausalityError(format!(
                                    "Causaloid {} has a context_causal_fn but is missing a context",
                                    self.id
                                ))),
                                logs: incoming_effect.logs,
                            };
                        }
                    };
                    context_fn(&incoming_effect, context)
                } else {
                    // Standard causal function w/o context
                    let causal_fn = match self.causal_fn {
                        Some(f) => f,
                        None => {
                            return PropagatingEffect {
                                value: EffectValue::None,
                                error: Some(CausalityError(format!(
                                    "Causaloid {} is missing a causal_fn",
                                    self.id
                                ))),
                                logs: incoming_effect.logs,
                            };
                        }
                    };
                    causal_fn(&incoming_effect)
                }
            }
            CausaloidType::Collection => {
                let coll = match self.causal_coll.as_ref() {
                    Some(c) => c,
                    None => {
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(
                                "Causaloid::evaluate_monadic: causal collection is None".into(),
                            )),
                            logs: incoming_effect.logs,
                        };
                    }
                };
                coll.evaluate_collection_monadic(incoming_effect)
            }
            CausaloidType::Graph => {
                let graph = match self.causal_graph.as_ref() {
                    Some(g) => g,
                    None => {
                        return PropagatingEffect {
                            value: EffectValue::None,
                            error: Some(CausalityError(
                                "Causaloid::evaluate_monadic: Causal graph is None".into(),
                            )),
                            logs: incoming_effect.logs,
                        };
                    }
                };
                graph.evaluate_graph_monadic(incoming_effect)
            }
        }
    }
}
