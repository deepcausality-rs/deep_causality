/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{
    AggregateLogic, CausalMonad, CausalityError, Causaloid, CausaloidType, EffectValue,
    PropagatingEffect, StandardPropagatingEffect,
};
use crate::{
    Causable, CausableCollectionExplaining, CausableCollectionReasoning, Datable, MonadicCausable,
    SpaceTemporal, Spatial, Symbolic, Temporal,
};

#[allow(clippy::type_complexity)]
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
    fn evaluate(&self, effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        let effect = match self.causal_type {
            CausaloidType::Singleton => {
                if !matches!(self.causal_type, CausaloidType::Singleton) {
                    return Err(CausalityError(
                        "reason_singleton called on a non-singleton Causaloid".into(),
                    ));
                }

                // Check if a context-aware function should be used
                if let Some(context_fn) = self.context_causal_fn {
                    let context = self.context.as_ref().ok_or_else(|| {
                        CausalityError(format!(
                            "Causaloid {} has a context_causal_fn but is missing a context",
                            self.id
                        ))
                    })?;
                    context_fn(effect, context)
                } else {
                    // Standard causal function w/o context
                    let causal_fn = self.causal_fn.ok_or_else(|| {
                        CausalityError(format!("Causaloid {} is missing a causal_fn", self.id))
                    })?;
                    causal_fn(effect)
                }?
            }

            CausaloidType::Collection => {
                let coll = self.causal_coll.as_ref().ok_or_else(|| {
                    CausalityError("Causaloid::evaluate: causal collection is None".into())
                })?;

                coll.evaluate_mixed(effect, &AggregateLogic::All, 0.5)?
            }

            CausaloidType::Graph => {
                let graph = self.causal_graph.as_ref().ok_or_else(|| {
                    CausalityError("Causaloid::evaluate: Causal graph is None".into())
                })?;
                // Delegate evaluation to the graph, which also implements Causable.
                graph.evaluate(effect)?
            }
        };

        // Store the resulting effect for later inspection by is_active() and explain().
        let mut effect_guard = self.effect.write().unwrap();
        *effect_guard = Some(effect.clone());

        Ok(effect)
    }

    fn explain(&self) -> Result<String, CausalityError> {
        match self.causal_type {
            CausaloidType::Singleton => {
                let effect_guard = self.effect.read().unwrap();
                if let Some(effect) = effect_guard.as_ref() {
                    let reason = format!(
                        "Causaloid: {} '{}' evaluated to: {:?}",
                        self.id, self.description, effect
                    );
                    Ok(reason)
                } else {
                    let reason = format!(
                        "Causaloid: {} has not been evaluated. Call evaluate() to get its effect.",
                        self.id
                    );
                    Err(CausalityError(reason))
                }
            }

            CausaloidType::Collection => {
                // Safely unwrap the collection or return a descriptive error.
                self.causal_coll
                    .as_ref()
                    .ok_or_else(|| {
                        CausalityError(format!(
                            "Causaloid {} is type Collection but its collection is None",
                            self.id
                        ))
                    })?
                    .explain() // Delegate to the collection's explain method.
            }

            CausaloidType::Graph => {
                // Safely unwrap the graph or return a descriptive error.
                self.causal_graph
                    .as_ref()
                    .ok_or_else(|| {
                        CausalityError(format!(
                            "Causaloid {} is type Graph but its graph is None",
                            self.id
                        ))
                    })?
                    .explain() // Delegate to the graph's explain method.
            }
        }
    }

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
    fn evaluate_monadic(
        &self,
        incoming_effect: StandardPropagatingEffect,
    ) -> StandardPropagatingEffect {
        match self.causal_type {
            CausaloidType::Singleton => {
                // The causal_fn and context_causal_fn fields currently hold functions
                // with the old signature (returning Result<PropagatingEffect, CausalityError>).
                // They need to be updated to the new monadic signature (returning StandardPropagatingEffect).
                // For now, we return an error indicating this incompatibility.
                StandardPropagatingEffect {
                    value: EffectValue::None,
                    error: Some(CausalityError(
                        "Causaloid's internal functions (causal_fn/context_causal_fn) are not yet updated to monadic signatures.".to_string(),
                    )),
                    logs: incoming_effect.logs,
                }
            }
            CausaloidType::Collection => {
                // Placeholder for monadic collection evaluation
                // This will eventually call MonadicCausableCollection::evaluate_collection_monadic
                StandardPropagatingEffect {
                    value: EffectValue::None,
                    error: Some(CausalityError(
                        "Monadic collection evaluation not yet implemented for Causaloid."
                            .to_string(),
                    )),
                    logs: incoming_effect.logs,
                }
            }
            CausaloidType::Graph => {
                // Placeholder for monadic graph evaluation
                // This will eventually call MonadicCausableGraphReasoning::evaluate_graph_monadic
                StandardPropagatingEffect {
                    value: EffectValue::None,
                    error: Some(CausalityError(
                        "Monadic graph evaluation not yet implemented for Causaloid.".to_string(),
                    )),
                    logs: incoming_effect.logs,
                }
            }
        }
    }
}
