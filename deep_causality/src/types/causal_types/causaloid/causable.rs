/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, Causable, CausableGraph, CausalMonad, CausalityError, Causaloid, CausaloidType,
    Datable, MonadicCausable, MonadicCausableCollection, MonadicCausableGraphReasoning,
    PropagatingEffect, SpaceTemporal, Spatial, Symbolic, Temporal,
};
use deep_causality_haft::MonadEffect3;

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
    fn evaluate(&self, incoming_effect: &PropagatingEffect) -> PropagatingEffect {
        match self.causal_type {
            // Logs are chained in CausalMonad::bind
            CausaloidType::Singleton => CausalMonad::bind(
                incoming_effect.clone(),
                |current_effect_value: crate::EffectValue| {
                    if let Some(context_fn) = self.context_causal_fn {
                        if let Some(context) = self.context.as_ref() {
                            context_fn(current_effect_value, context)
                        } else {
                            PropagatingEffect::from_error(CausalityError(format!(
                                "Causaloid {} has a context_causal_fn but is missing a context",
                                self.id
                            )))
                        }
                    } else if let Some(causal_fn) = self.causal_fn {
                        causal_fn(current_effect_value)
                    } else {
                        PropagatingEffect::from_error(CausalityError(format!(
                            "Causaloid {} is missing both causal_fn and context_causal_fn",
                            self.id
                        )))
                    }
                },
            ),
            CausaloidType::Collection => {
                let coll = match self.causal_coll.as_ref() {
                    Some(c) => c,
                    None => {
                        return PropagatingEffect::from_error(CausalityError(
                            "Causaloid::evaluate_monadic: causal collection is None".into(),
                        ));
                    }
                };
                coll.evaluate_collection(incoming_effect, &AggregateLogic::All, Some(0.80))
            }
            CausaloidType::Graph => {
                let graph = match self.causal_graph.as_ref() {
                    Some(g) => g,
                    None => {
                        return PropagatingEffect::from_error(CausalityError(
                            "Causaloid::evaluate_monadic: Causal graph is None".into(),
                        ));
                    }
                };

                // Since the graph is guaranteed to have a single root, start evaluation there.
                let root_index = match graph.get_root_index() {
                    Some(index) => index,
                    None => {
                        return PropagatingEffect::from_error(CausalityError(
                            "Cannot evaluate graph: Root node not found.".into(),
                        ));
                    }
                };

                // Delegate to the reasoning algorithm from the `CausableGraphReasoning` trait.
                // This will traverse and evaluate the entire graph from the root.
                graph.evaluate_subgraph_from_cause(root_index, incoming_effect)
            }
        }
    }
}
