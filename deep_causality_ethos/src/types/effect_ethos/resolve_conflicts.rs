/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{HashMap, HashSet, VecDeque};

use crate::TeloidStorable;
use crate::{DeonticError, EffectEthos, Teloid, TeloidID, TeloidRelation};
use deep_causality::{Datable, SpaceTemporal, Spatial, Symbolic, Temporal};
use ultragraph::{GraphTraversal, GraphView};
#[allow(clippy::type_complexity)]
impl<D, S, T, ST, SYM, VS, VT> EffectEthos<D, S, T, ST, SYM, VS, VT>
where
    D: Datable + Clone,
    S: Spatial<VS> + Clone,
    T: Temporal<VT> + Clone,
    ST: SpaceTemporal<VS, VT> + Clone,
    SYM: Symbolic + Clone,
    VS: Clone,
    VT: Clone,
{
    /// Resolves conflicts among teloids based on a given set of active teloids.
    ///
    /// This method performs a Breadth-First Search (BFS) traversal starting from the
    /// `active_teloids`. During the traversal, it identifies and resolves conflicts
    /// using the following rules:
    ///
    /// 1. **Defeaters**: If a teloid is defeated by another teloid (connected via a `Defeats`
    ///    relation), it is removed from the set of inferred beliefs. Defeat is determined
    ///    by applying Lex Specialis (more specific rules override general ones) and
    ///    Lex Posterior (newer rules override older ones).
    /// 2. **Inheritance**: If a teloid is not defeated, its inheriting children (connected via
    ///    an `Inherits` relation) are added to the queue for further processing, and their
    ///    beliefs are inferred.
    ///
    /// # Arguments
    ///
    /// * `active_teloids` - A slice of references to `Teloid` instances that are considered
    ///   active and from which the conflict resolution process begins.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec` of `Teloid` instances that represent the resolved
    /// set of beliefs after conflict resolution, or a `DeonticError` if any error occurs
    /// during the process (e.g., teloid not found).
    pub(super) fn resolve_conflicts(
        &self,
        active_teloids: &[&Teloid<D, S, T, ST, SYM, VS, VT>],
    ) -> Result<Vec<Teloid<D, S, T, ST, SYM, VS, VT>>, DeonticError> {
        let mut queue: VecDeque<usize> = VecDeque::new();
        let mut visited: HashSet<usize> = HashSet::new();
        let mut inferred_beliefs: HashMap<TeloidID, Teloid<D, S, T, ST, SYM, VS, VT>> =
            HashMap::new();

        // Use the authoritative mapping maintained by EffectEthos
        let id_to_index = &self.id_to_index_map;

        // Initialize the queue with the indices of the active teloids.
        for &teloid in active_teloids {
            let start_index = id_to_index
                .get(&teloid.id())
                .ok_or(DeonticError::TeloidNotFound { id: teloid.id() })?;
            if visited.insert(*start_index) {
                queue.push_back(*start_index);
                // Clone the teloid into the belief set.
                inferred_beliefs.insert(teloid.id(), teloid.clone());
            }
        }

        // Start BFS traversal
        while let Some(current_idx) = queue.pop_front() {
            let current_id = self
                .teloid_graph
                .graph
                .get_node(current_idx)
                .copied()
                .ok_or(DeonticError::TeloidNotFound { id: 0 })?;

            // To avoid borrow checker issues, we clone the current teloid to get ownership.
            let current_teloid = self
                .teloid_store
                .get(&current_id)
                .ok_or(DeonticError::TeloidNotFound { id: current_id })?
                .clone();

            // Check for defeaters for the current node.
            let mut is_defeated = false;
            for defeater_idx in self.teloid_graph.graph.inbound_edges(current_idx)? {
                if let Some(edges) = self.teloid_graph.graph.get_edges(defeater_idx)
                    && edges.iter().any(|(target, relation)| {
                        *target == current_idx && *relation == &TeloidRelation::Defeats
                    })
                {
                    let defeater_id = self
                        .teloid_graph
                        .graph
                        .get_node(defeater_idx)
                        .copied()
                        .ok_or(DeonticError::TeloidNotFound { id: 0 })?;
                    if let Some(defeater_teloid) = inferred_beliefs.get(&defeater_id) {
                        // Apply Lex Specialis, Lex Posterior, and Lex Superior rules
                        if defeater_teloid.specificity() > current_teloid.specificity()
                            || defeater_teloid.timestamp() > current_teloid.timestamp()
                            || defeater_teloid.priority() > current_teloid.priority()
                        {
                            is_defeated = true;
                            break;
                        }
                    }
                }
            }

            if is_defeated {
                inferred_beliefs.remove(&current_id);
                continue; // Stop processing this path if the norm is defeated.
            }

            // If not defeated, continue traversal to inheriting children.
            for child_idx in self.teloid_graph.graph.outbound_edges(current_idx)? {
                if let Some(edges) = self.teloid_graph.graph.get_edges(current_idx)
                    && edges.iter().any(|(target, relation)| {
                        *target == child_idx && *relation == &TeloidRelation::Inherits
                    })
                    && visited.insert(child_idx)
                {
                    let child_id = self
                        .teloid_graph
                        .graph
                        .get_node(child_idx)
                        .copied()
                        .ok_or(DeonticError::TeloidNotFound { id: 0 })?;
                    let child_teloid = self
                        .teloid_store
                        .get(&child_id)
                        .ok_or(DeonticError::TeloidNotFound { id: child_id })?;

                    inferred_beliefs.insert(child_id, child_teloid.clone());
                    queue.push_back(child_idx);
                }
            }
        }

        Ok(inferred_beliefs.values().cloned().collect())
    }
}
