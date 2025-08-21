/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashMap;

use ultragraph::GraphView;

use crate::{
    Context, DeonticError, EffectEthos, ProposedAction, Teloid, TeloidID, TeloidStorable, Verdict,
};
use crate::{Datable, DeonticInferable, SpaceTemporal, Spatial, Symbolic, Temporal};

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
    ///    `ProposedAction` using a tag index.
    /// 3. **Activation (Step 2)**: Filters the candidate `Teloid`s based on their activation predicates,
    ///    which determine if a `Teloid` is applicable in the given `Context`.
    /// 4. **Belief Inference & Conflict Resolution (Steps 3 & 4)**: Resolves any conflicts among the
    ///    active `Teloid`s to arrive at a final set of applicable norms.
    /// 5. **Verdict Finding (Step 5)**: Derives a final `Verdict` based on the resolved norms.
    ///
    /// # Arguments
    /// * `action` - A reference to the `ProposedAction` to be evaluated.
    /// * `context` - A reference to the `Context` providing environmental and situational data.
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

        // Step 1: Filtering - Get a candidate set of Teloids using the tag index.
        let Some(candidate_ids) = self.tag_index.get(action.action_name()) else {
            // No relevant norms found, so the action is inconclusive by default.
            return Err(DeonticError::InconclusiveVerdict);
        };

        // Mitigation for Risk P1: Create a local cache for this evaluation.
        let mut teloid_cache: HashMap<TeloidID, &Teloid<D, S, T, ST, SYM, VS, VT>> = HashMap::new();

        // Step 2: Activation - Filter candidates by their activation predicate.
        let active_teloids: Vec<&Teloid<D, S, T, ST, SYM, VS, VT>> = candidate_ids
            .iter()
            .filter_map(|id| {
                let teloid = self.teloid_store.get(id)?;
                teloid_cache.insert(*id, teloid);

                if (teloid.activation_predicate())(context, action) {
                    Some(teloid)
                } else {
                    None
                }
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
