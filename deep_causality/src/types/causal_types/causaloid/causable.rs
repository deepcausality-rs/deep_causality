/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, Causable, CausalMonad, CausalityError, Causaloid, CausaloidType, Datable,
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
                // Error checking happens in evaluate_singleton
                self.evaluate_singleton(incoming_effect)
            }
            CausaloidType::Collection => {
                let coll = match self.causal_coll.as_ref() {
                    Some(c) => c,
                    None => {
                        return PropagatingEffect::from_error(CausalityError(
                            "Causaloid::evaluate_monadic: causal collection is None".into(),
                        ));
                    }
                };
                coll.evaluate_collection(&incoming_effect, &AggregateLogic::All, 0.80)
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
                graph.evaluate_graph(incoming_effect)
            }
        }
    }
}

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> Causaloid<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    fn evaluate_singleton(&self, incoming_effect: PropagatingEffect) -> PropagatingEffect {
        // Check if a context-aware function should be used
        if let Some(context_fn) = self.context_causal_fn {
            let context = match self.context.as_ref() {
                Some(c) => c,
                None => {
                    return PropagatingEffect::from_error(CausalityError(format!(
                        "Causaloid {} has a context_causal_fn but is missing a context",
                        self.id
                    )));
                }
            };
            context_fn(&incoming_effect, context)
        } else {
            // Standard causal function w/o context
            let causal_fn = match self.causal_fn {
                Some(f) => f,
                None => {
                    return PropagatingEffect::from_error(CausalityError(format!(
                        "Causaloid {} is missing a causal_fn",
                        self.id
                    )));
                }
            };
            causal_fn(&incoming_effect)
        }
    }
}
