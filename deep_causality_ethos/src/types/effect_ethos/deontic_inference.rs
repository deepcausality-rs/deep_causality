/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{HashMap, HashSet};

use ultragraph::GraphView;

use crate::{DeonticError, EffectEthos, Teloid, TeloidID, TeloidTag, Verdict};
use crate::{DeonticInferable, TeloidStorable};
use deep_causality::{Context, ProposedAction};
use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> DeonticInferable<D, S, T, ST, SYM, VS, VT>
    for EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Evaluates a proposed action against the established norms and beliefs within the `EffectEthos` system.
    ///
    /// This method orchestrates the deontic inference process, which involves several key steps:
    /// 1. **Graph State Check**: Ensures the internal teloid graph is frozen and verified for acyclicity
    ///    to guarantee a stable and reliable evaluation.
    /// 2. **Filtering (Step 1)**: Identifies a candidate set of `Teloid`s (norms/beliefs) relevant to the
    ///    `ProposedAction` using a list of tags.
    /// 3. **Activation (Step 2)**: Filters the candidate `Teloid`s based on their activation predicates,
    ///    which determine if a `Teloid` is applicable in the given `Context`.
    /// 4. **Belief Inference & Conflict Resolution (Steps 3 & 4)**: Resolves any conflicts among the
    ///    active `Teloid`s to arrive at a final set of applicable norms.
    /// 5. **Verdict Finding (Step 5)**: Derives a final `Verdict` based on the resolved norms.
    ///
    /// # Arguments
    /// * `action` - A reference to the `ProposedAction` to be evaluated.
    /// * `context` - A reference to the `Context` providing environmental and situational data.
    /// * `tags` - A slice of `TeloidTag`s used to retrieve relevant norms from the tag index.
    ///
    /// # Returns
    /// A `Result` which is:
    /// * `Ok(Verdict)` if a verdict can be successfully derived.
    /// * `Err(DeonticError)` if the graph is not frozen, is cyclic, no relevant norms are found,
    ///   no teloids are active, or if conflict resolution fails.
    fn evaluate_action(
        &self,
        action: &ProposedAction,
        context: &Context<D, S, T, ST, SYM, VS, VT>,
        tags: &[TeloidTag],
    ) -> Result<Verdict, DeonticError> {
        // Mitigation for Risk A2: Explicitly check if the graph is frozen.
        if !self.teloid_graph.graph.is_frozen() {
            return Err(DeonticError::GraphNotFrozen);
        }

        // Mitigation for Risk C3: Check if the graph has been verified for acyclicity.
        if !self.is_verified {
            // Suggesting verification instead of just erroring out.
            return Err(DeonticError::GraphIsCyclic);
        }

        // Step 1: Filtering - Get a candidate set of Teloids using the provided tags.
        let candidate_ids: HashSet<TeloidID> = tags
            .iter()
            .flat_map(|tag| self.tag_index.get(tag).into_iter().flatten())
            .copied()
            .collect();

        if candidate_ids.is_empty() {
            // No relevant norms found, so the action cannot be decided.
            return Err(DeonticError::NoRelevantNormsFound);
        };

        // Mitigation for Risk P1: Create a local cache for this evaluation.
        let mut teloid_cache: HashMap<TeloidID, &Teloid<D, S, T, ST, SYM, VS, VT>> = HashMap::new();

        // Step 2: Activation - Filter candidates by their activation predicate.
        let active_teloids: Vec<&Teloid<D, S, T, ST, SYM, VS, VT>> = candidate_ids
            .iter()
            .filter_map(|id| {
                let teloid = self.teloid_store.get(id)?;
                teloid_cache.insert(*id, teloid);

                let is_active = if let Some(predicate) = teloid.activation_predicate() {
                    // Handle deterministic predicate
                    predicate(context, action)
                } else if let Some(uncertain_predicate) = teloid.uncertain_activation_predicate() {
                    // Handle uncertain predicate
                    match uncertain_predicate(context, action) {
                        Ok(uncertain_bool) => {
                            // An uncertain predicate MUST have parameters.
                            if let Some(params) = teloid.uncertain_parameter() {
                                uncertain_bool
                                    .probability_exceeds(
                                        params.threshold(),
                                        params.confidence(),
                                        params.epsilon(),
                                        params.max_samples(),
                                    )
                                    .unwrap_or(false) // Treat test error as inactive
                            } else {
                                // This case should be logically impossible if constructors are used correctly.
                                // Treat as inactive as a safeguard.
                                false
                            }
                        }
                        Err(_) => false, // If predicate function fails, treat as inactive
                    }
                } else {
                    // No predicate defined, treat as inactive
                    false
                };

                if is_active { Some(teloid) } else { None }
            })
            .collect();

        if active_teloids.is_empty() {
            return Err(DeonticError::InconclusiveVerdict);
        }

        // Steps 3 & 4: Belief Inference and Conflict Resolution (combined in traversal)
        let final_norms = self.resolve_conflicts(&active_teloids)?;

        // Step 5: Verdict Finding
        self.derive_verdict(final_norms)
    }
}
