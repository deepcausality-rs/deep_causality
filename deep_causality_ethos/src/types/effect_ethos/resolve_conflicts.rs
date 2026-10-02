/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::HashSet;

use crate::TeloidStorable;
use crate::{DeonticError, EffectEthos, Teloid, TeloidRelation};
use deep_causality_context::{Datable, SpaceTemporal, Spatial, Temporal};
use ultragraph::{GraphError, GraphTraversal, GraphView, TopologicalGraphAlgorithms};

#[allow(clippy::type_complexity)]
impl<D, S, T, ST> EffectEthos<D, S, T, ST>
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    /// Resolves conflicts among the active teloids and returns the norms that stand, in the
    /// graph's topological order.
    ///
    /// A norm is held when it is active or when a standing norm passes it on along an `Inherits`
    /// edge. A held norm falls when a standing norm defeats it along a `Defeats` edge and outranks
    /// it. Rank compares priority first (Lex Superior), then specificity (Lex Specialis), then
    /// timestamp (Lex Posterior); a norm of equal rank does not defeat. A norm that falls defeats
    /// nothing and passes nothing on.
    ///
    /// The graph is acyclic once verified, so every defeater and every parent is settled before
    /// the norm it acts on. The result therefore depends only on the norms, their edges and which
    /// of them are active.
    ///
    /// # Errors
    ///
    /// `TeloidNotFound` when an active teloid or a graph node has no entry in the store, and a
    /// graph error when the graph cannot be read.
    pub(super) fn resolve_conflicts(
        &self,
        active_teloids: &[&Teloid<D, S, T, ST>],
    ) -> Result<Vec<Teloid<D, S, T, ST>>, DeonticError> {
        let mut held: HashSet<usize> = HashSet::with_capacity(active_teloids.len());
        for teloid in active_teloids {
            let index = self
                .id_to_index_map
                .get(&teloid.id())
                .ok_or(DeonticError::TeloidNotFound { id: teloid.id() })?;
            held.insert(*index);
        }

        let order = self
            .teloid_graph
            .graph
            .topological_sort()?
            .ok_or(DeonticError::GraphIsCyclic(GraphError::GraphContainsCycle))?;

        let mut standing: HashSet<usize> = HashSet::with_capacity(held.len());
        let mut survivors = Vec::with_capacity(held.len());
        for index in order {
            if !held.contains(&index) {
                continue;
            }
            let teloid = self.teloid_at(index)?;

            let mut defeated = false;
            for defeater_index in self.teloid_graph.graph.inbound_edges(index)? {
                if standing.contains(&defeater_index)
                    && self.relation(defeater_index, index) == Some(TeloidRelation::Defeats)
                    && outranks(self.teloid_at(defeater_index)?, teloid)
                {
                    defeated = true;
                    break;
                }
            }
            if defeated {
                continue;
            }

            standing.insert(index);
            survivors.push(teloid.clone());
            for child_index in self.teloid_graph.graph.outbound_edges(index)? {
                if self.relation(index, child_index) == Some(TeloidRelation::Inherits) {
                    held.insert(child_index);
                }
            }
        }
        Ok(survivors)
    }

    /// The teloid stored for the graph node at `index`.
    fn teloid_at(&self, index: usize) -> Result<&Teloid<D, S, T, ST>, DeonticError> {
        let id = self
            .teloid_graph
            .graph
            .get_node(index)
            .copied()
            .ok_or(DeonticError::TeloidNotFound { id: 0 })?;
        self.teloid_store
            .get(&id)
            .ok_or(DeonticError::TeloidNotFound { id })
    }

    /// The relation the edge from `from` to `to` carries, if there is one.
    fn relation(&self, from: usize, to: usize) -> Option<TeloidRelation> {
        self.teloid_graph
            .graph
            .get_edges(from)?
            .iter()
            .find(|(target, _)| *target == to)
            .map(|(_, relation)| **relation)
    }
}

/// Whether `a` outranks `b`: higher priority, then higher specificity, then a later timestamp.
fn outranks<D, S, T, ST>(a: &Teloid<D, S, T, ST>, b: &Teloid<D, S, T, ST>) -> bool
where
    D: Datable + Clone,
    S: Spatial + Clone,
    T: Temporal + Clone,
    ST: SpaceTemporal + Clone,
{
    (a.priority(), a.specificity(), a.timestamp()) > (b.priority(), b.specificity(), b.timestamp())
}
