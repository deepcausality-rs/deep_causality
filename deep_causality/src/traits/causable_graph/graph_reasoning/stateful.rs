/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Stateful counterpart to [`super::MonadicCausableGraphReasoning`].
//!
//! Mirrors the BFS / shortest-path traversal of the stateless trait but
//! invokes [`crate::StatefulMonadicCausable::evaluate_stateful`] on each node,
//! threading the per-node `state` and `context` into the next node's incoming
//! process. The `RelayTo` adaptive-jump branch is preserved: when a node
//! returns a `RelayTo` command the relayed-to node receives
//! a `PropagatingProcess` whose `state` and `context` are the ones the
//! relaying node carried at the moment of relay.
//!
//! Statefulness is selected by calling these methods instead of the stateless
//! ones. No new graph constructor is required — use the existing
//! [`crate::Causaloid::from_causal_graph_with_context`].

use crate::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use ultragraph::{GraphTraversal, TopologicalGraphAlgorithms};

/// Builds an errored process from borrowed channels (clone-and-raise helper).
fn raise_from<V, S, C>(
    err: CausalityError,
    source: &PropagatingProcess<V, S, C>,
) -> PropagatingProcess<V, S, C>
where
    S: Clone,
    C: Clone,
{
    PropagatingProcess::new(
        Err(err),
        source.state().clone(),
        source.context().clone(),
        source.logs().clone(),
    )
}

/// Stateful counterpart to [`crate::MonadicCausableGraphReasoning`].
pub trait StatefulMonadicCausableGraphReasoning<V, S, C>:
    CausableGraph<Causaloid<V, V, S, C>>
where
    V: Default + Clone + Send + Sync + 'static + Debug,
    S: Default + Clone + Send + Sync + 'static + Debug,
    C: Clone + Send + Sync + 'static,
    Causaloid<V, V, S, C>: MonadicCausable<V, V> + StatefulMonadicCausable<V, V, S, C>,
{
    /// Stateful counterpart to
    /// [`crate::MonadicCausableGraphReasoning::evaluate_single_cause`].
    fn evaluate_single_cause_stateful(
        &self,
        index: usize,
        effect: &PropagatingProcess<V, S, C>,
    ) -> PropagatingProcess<V, S, C> {
        // Short-circuit if the incoming process already carries an error.
        if let Err(err) = effect.outcome() {
            return raise_from(err.clone(), effect);
        }

        if !self.is_frozen() {
            return raise_from(
                CausalityError(CausalityErrorEnum::Custom(
                    "Graph is not frozen. Call freeze() first".into(),
                )),
                effect,
            );
        }

        let causaloid = match self.get_causaloid(index) {
            Some(c) => c,
            None => {
                return raise_from(
                    CausalityError(CausalityErrorEnum::Custom(format!(
                        "Causaloid with index {index} not found in graph"
                    ))),
                    effect,
                );
            }
        };

        causaloid.evaluate_stateful(effect)
    }

    /// Stateful counterpart to
    /// [`crate::MonadicCausableGraphReasoning::evaluate_subgraph_from_cause`].
    fn evaluate_subgraph_from_cause_stateful(
        &self,
        start_index: usize,
        initial_effect: &PropagatingProcess<V, S, C>,
    ) -> PropagatingProcess<V, S, C> {
        // Short-circuit if the incoming process already carries an error.
        if let Err(err) = initial_effect.outcome() {
            return raise_from(err.clone(), initial_effect);
        }

        if !self.is_frozen() {
            return raise_from(
                CausalityError(CausalityErrorEnum::Custom(
                    "Graph is not frozen. Call freeze() first".into(),
                )),
                initial_effect,
            );
        }

        if !self.contains_causaloid(start_index) {
            return raise_from(
                CausalityError(CausalityErrorEnum::Custom(format!(
                    "Graph does not contain start causaloid with index {start_index}"
                ))),
                initial_effect,
            );
        }

        // The stateful evaluator mirrors the stateless wire-slot engine (reachability pre-pass,
        // ascending-index canonical schedule, `RelayTo` as sequential round composition), so it too
        // requires a frozen acyclic graph.
        if self.get_graph().has_cycle().unwrap_or(true) {
            return raise_from(
                CausalityError(CausalityErrorEnum::Custom(
                    "Graph contains a directed cycle; the reconvergence-join evaluator requires an \
                     acyclic (frozen DAG) graph"
                        .into(),
                )),
                initial_effect,
            );
        }

        let n_nodes = self.number_nodes();
        let mut round_start = start_index;
        let mut round_input = initial_effect.clone();

        'rounds: loop {
            // Reachability pre-pass: only `round_start` and its descendants can fire.
            let mut reachable = vec![false; n_nodes];
            reachable[round_start] = true;
            let mut stack = vec![round_start];
            while let Some(node) = stack.pop() {
                if let Ok(children) = self.get_graph().outbound_edges(node) {
                    for c in children {
                        if !reachable[c] {
                            reachable[c] = true;
                            stack.push(c);
                        }
                    }
                }
            }

            let mut pending = vec![0usize; n_nodes];
            let mut fired: Vec<BTreeMap<usize, PropagatingProcess<V, S, C>>> =
                (0..n_nodes).map(|_| BTreeMap::new()).collect();
            let mut processed = vec![false; n_nodes];

            for node in 0..n_nodes {
                if !reachable[node] || node == round_start {
                    continue;
                }
                if let Ok(parents) = self.get_graph().inbound_edges(node) {
                    pending[node] = parents.filter(|p| reachable[*p]).count();
                }
            }

            let mut ready: BTreeSet<usize> = BTreeSet::new();
            ready.insert(round_start);

            let mut last_propagated = round_input.clone();

            while let Some(node) = ready.pop_first() {
                if processed[node] {
                    continue;
                }
                processed[node] = true;

                let incoming = if node == round_start {
                    round_input.clone()
                } else {
                    let parents = std::mem::take(&mut fired[node]);
                    match parents.len() {
                        0 => {
                            // Unreachable invariant guard (see the stateless engine): the reachability
                            // pre-pass prunes dead paths at the wire level, so a non-start node that
                            // becomes ready always has at least one fired parent.
                            return raise_from(
                                CausalityError(CausalityErrorEnum::Custom(format!(
                                    "internal invariant: node {node} became ready with no fired parents"
                                ))),
                                &last_propagated,
                            );
                        }
                        1 => {
                            // Join of one fired parent is the identity: thread its process through.
                            parents.into_values().next().expect("len == 1")
                        }
                        _ => {
                            // A stateful reconvergence join (combining ≥2 full carriers) is deferred:
                            // no stateful multi-parent graph exists in the suite, and the join
                            // mechanisms produce the stateless carrier. Fail loudly rather than pick a
                            // silent state/context combine (D5 / blast-radius scan).
                            let keys: Vec<usize> = parents.keys().copied().collect();
                            return raise_from(
                                CausalityError(CausalityErrorEnum::Custom(format!(
                                    "Node {node} is a stateful reconvergence with {} fired parents \
                                     {keys:?}; stateful fan-in joins are not yet supported (deferred \
                                     — see comonoid-graph-join D5)",
                                    keys.len()
                                ))),
                                &last_propagated,
                            );
                        }
                    }
                };

                let causaloid = match self.get_causaloid(node) {
                    Some(c) => c,
                    None => {
                        return raise_from(
                            CausalityError(CausalityErrorEnum::Custom(format!(
                                "Failed to get causaloid at index {node}"
                            ))),
                            &last_propagated,
                        );
                    }
                };

                let result = causaloid.evaluate_stateful(&incoming);
                last_propagated = result.clone();

                if result.is_err() {
                    return result;
                }

                match result.command_target() {
                    Some(target_idx) => {
                        if !self.contains_causaloid(target_idx) {
                            return raise_from(
                                CausalityError(CausalityErrorEnum::Custom(format!(
                                    "RelayTo target causaloid with index {target_idx} not found in graph."
                                ))),
                                &last_propagated,
                            );
                        }
                        // Carry the relaying node's state, context, and logs forward into the new round.
                        let relayed: PropagatingProcess<V, S, C> = PropagatingProcess::new(
                            Ok(result
                                .into_parts()
                                .0
                                .ok()
                                .and_then(CausalEffect::into_command)
                                .map(|(_, sub)| sub)
                                .unwrap_or_else(CausalEffect::none)),
                            last_propagated.state().clone(),
                            last_propagated.context().clone(),
                            last_propagated.logs().clone(),
                        );
                        round_start = target_idx;
                        round_input = relayed;
                        continue 'rounds;
                    }
                    None => {
                        let children = match self.get_graph().outbound_edges(node) {
                            Ok(c) => c,
                            Err(e) => {
                                return raise_from(
                                    CausalityError(CausalityErrorEnum::Custom(format!("{e}"))),
                                    &last_propagated,
                                );
                            }
                        };
                        for c in children {
                            if reachable[c] && !processed[c] {
                                fired[c].insert(node, result.clone());
                                pending[c] = pending[c].saturating_sub(1);
                                if pending[c] == 0 {
                                    ready.insert(c);
                                }
                            }
                        }
                    }
                }
            }

            return last_propagated;
        }
    }

    /// Stateful counterpart to
    /// [`crate::MonadicCausableGraphReasoning::evaluate_shortest_path_between_causes`].
    fn evaluate_shortest_path_between_causes_stateful(
        &self,
        start_index: usize,
        stop_index: usize,
        initial_effect: &PropagatingProcess<V, S, C>,
    ) -> PropagatingProcess<V, S, C> {
        // Short-circuit if the incoming process already carries an error.
        if let Err(err) = initial_effect.outcome() {
            return raise_from(err.clone(), initial_effect);
        }

        if !self.is_frozen() {
            return raise_from(
                CausalityError(CausalityErrorEnum::Custom(
                    "Graph is not frozen. Call freeze() first".into(),
                )),
                initial_effect,
            );
        }

        if start_index == stop_index {
            let causaloid = match self.get_causaloid(start_index) {
                Some(c) => c,
                None => {
                    return raise_from(
                        CausalityError(CausalityErrorEnum::Custom(format!(
                            "Failed to get causaloid at index {start_index}"
                        ))),
                        initial_effect,
                    );
                }
            };
            return causaloid.evaluate_stateful(initial_effect);
        }

        let path = match self.get_shortest_path(start_index, stop_index) {
            Ok(p) => p,
            Err(e) => {
                return raise_from(
                    CausalityError(CausalityErrorEnum::Custom(format!("{:?}", e))),
                    initial_effect,
                );
            }
        };

        let mut current = initial_effect.clone();

        for index in path {
            let causaloid = match self.get_causaloid(index) {
                Some(c) => c,
                None => {
                    return raise_from(
                        CausalityError(CausalityErrorEnum::Custom(format!(
                            "Failed to get causaloid at index {index}"
                        ))),
                        &current,
                    );
                }
            };

            current = causaloid.evaluate_stateful(&current);

            if current.is_err() {
                return current;
            }

            if current.command_target().is_some() {
                return current;
            }
        }

        current
    }
}
