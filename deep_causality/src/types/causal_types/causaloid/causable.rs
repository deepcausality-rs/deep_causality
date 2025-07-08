/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::CausalityError;
use crate::traits::causable::causable_reasoning::CausableReasoning;
use crate::traits::contextuable::space_temporal::SpaceTemporal;
use crate::traits::contextuable::spatial::Spatial;
use crate::traits::contextuable::temporal::Temporal;
use crate::types::causal_types::causaloid::causal_type::CausaloidType;
use crate::{Causable, Causaloid, Datable, Evidence, PropagatingEffect, Symbolic};

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
    fn evaluate(&self, evidence: &Evidence) -> Result<PropagatingEffect, CausalityError> {
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
                    context_fn(evidence, context)
                } else {
                    // Standard path
                    let causal_fn = self.causal_fn.ok_or_else(|| {
                        CausalityError(format!("Causaloid {} is missing a causal_fn", self.id))
                    })?;
                    causal_fn(evidence)
                }?
            }

            CausaloidType::Collection => {
                let coll = self.causal_coll.as_ref().ok_or_else(|| {
                    CausalityError("Causaloid::evaluate: causal collection is None".into())
                })?;

                // Default aggregation: "any true" logic.
                // Prioritizes Halting, then looks for the first Deterministic(true).
                let mut has_true = false;
                for cause in coll.iter() {
                    match cause.evaluate(evidence)? {
                        PropagatingEffect::Halting => return Ok(PropagatingEffect::Halting),
                        PropagatingEffect::Deterministic(true) => {
                            has_true = true;
                            break; // Short-circuit
                        }
                        _ => (), // Other effects are ignored for this aggregation
                    }
                }
                PropagatingEffect::Deterministic(has_true)
            }

            CausaloidType::Graph => {
                let graph = self.causal_graph.as_ref().ok_or_else(|| {
                    CausalityError("Causaloid::evaluate: Causal graph is None".into())
                })?;
                // Delegate evaluation to the graph, which also implements Causable.
                graph.evaluate(evidence)?
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
                // Delegates to the `explain` method provided by the `CausableReasoning` trait.
                Ok(self
                    .causal_coll
                    .as_ref()
                    .expect("Causaloid collection should not be None")
                    .explain()?)
            }

            CausaloidType::Graph => {
                // Delegates to the `explain` method on the graph itself.
                self.causal_graph
                    .as_ref()
                    .expect("Causaloid graph should not be None")
                    .explain()
            }
        }
    }

    fn is_singleton(&self) -> bool {
        matches!(self.causal_type, CausaloidType::Singleton)
    }
}
