/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

pub mod stateful;

use crate::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use ultragraph::{GraphTraversal, TopologicalGraphAlgorithms};

/// Provides default implementations for monadic reasoning over `CausableGraph` items.
///
/// Any graph type that implements `CausableGraph<T>` where `T` is `MonadicCausable<I, O>`
/// will automatically gain a suite of useful default methods for monadic evaluation.
/// Provides default implementations for monadic reasoning over `CausableGraph` items.
///
/// Any graph type that implements `CausableGraph<T>` where `T` is `MonadicCausable<V, V>`
/// will automatically gain a suite of useful default methods for monadic evaluation.
pub trait MonadicCausableGraphReasoning<V, PS, C>: CausableGraph<Causaloid<V, V, PS, C>>
where
    V: Default + Clone + Send + Sync + 'static + Debug,
    PS: Default + Clone + Send + Sync + 'static,
    C: Clone + Send + Sync + 'static,
    Causaloid<V, V, PS, C>: MonadicCausable<V, V>,
{
    /// Evaluates a single, specific causaloid within the graph by its index using a monadic approach.
    ///
    /// This is a convenience method that locates the causaloid and calls its `evaluate` method.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the causaloid to evaluate.
    /// * `effect` - The runtime effect to be passed to the node's evaluation function.
    ///
    /// # Returns
    ///
    /// The `PropagatingEffect` from the evaluated causaloid, or a `PropagatingEffect` containing
    /// a `CausalityError` if the node is not found or the evaluation fails.
    fn evaluate_single_cause(
        &self,
        index: usize,
        effect: &PropagatingEffect<V>,
    ) -> PropagatingEffect<V> {
        if !self.is_frozen() {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Graph is not frozen. Call freeze() first".into(),
            )));
        }

        let causaloid = match self.get_causaloid(index) {
            Some(c) => c,
            None => {
                return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                    format!("Causaloid with index {index} not found in graph"),
                )));
            }
        };

        causaloid.evaluate(effect)
    }

    /// Reasons over a subgraph by traversing all nodes reachable from a given start index,
    /// using a monadic approach.
    ///
    /// This method performs a Breadth-First Search (BFS) traversal of all descendants
    /// of the `start_index`. The `PropagatingEffect` is passed sequentially:
    /// the output effect of a parent node becomes the input effect for its child node.
    /// The traversal continues as long as no `CausalityError` is returned within the `PropagatingEffect`.
    ///
    /// ## Adaptive Reasoning
    ///
    /// If a `Causaloid` returns a `RelayTo` command effect, the BFS traversal dynamically jumps to
    /// its `target` index, and the command's sub-program becomes the new input for the relayed path.
    /// This enables *adaptive reasoning* conditional to the deciding causaloid.
    ///
    /// # Arguments
    ///
    /// * `start_index` - The index of the node to start the traversal from.
    /// * `initial_effect` - The initial runtime effect to be passed to the starting node's evaluation function.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` representing the final aggregated monadic effect of the traversal.
    /// If an error occurs during evaluation, the returned `PropagatingEffect` will contain the error.
    fn evaluate_subgraph_from_cause(
        &self,
        start_index: usize,
        initial_effect: &PropagatingEffect<V>,
    ) -> PropagatingEffect<V> {
        if !self.is_frozen() {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Graph is not frozen. Call freeze() first".into(),
            )));
        }

        if !self.contains_causaloid(start_index) {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                format!("Graph does not contain start causaloid with index {start_index}"),
            )));
        }

        // The classical fan-in evaluator requires a topological order, so the frozen graph must be
        // acyclic. A Kahn-style ready-set would otherwise silently skip nodes inside a cycle.
        if self.get_graph().has_cycle().unwrap_or(true) {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Graph contains a directed cycle; the reconvergence-join evaluator requires an \
                 acyclic (frozen DAG) graph"
                    .into(),
            )));
        }

        let n_nodes = self.number_nodes();

        // Evaluation composes rounds sequentially: a round folds the reachable acyclic sub-DAG with
        // labeled fan-in; a `RelayTo` ends the round and starts a fresh one at the relay target with
        // the command's sub-program as the new seed (sequential composition of rounds).
        let mut round_start = start_index;
        let mut round_input = initial_effect.clone();

        'rounds: loop {
            // Reachability pre-pass: only `round_start` and its descendants can fire. Every in-wire
            // from a non-descendant is thereby resolved `Inactive` up front (it is never counted in
            // `pending`), which is what keeps mid-graph starts and abandoned relay cones deadlock-free.
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

            // Wire-slot bookkeeping. `pending[n]` counts the *reachable* parents of `n` not yet
            // resolved; a wire from an unreachable parent is pre-resolved `Inactive` (not counted).
            // `fired[n]` accumulates the effects of parents that fired, keyed by parent node index.
            let mut pending = vec![0usize; n_nodes];
            let mut fired: Vec<BTreeMap<usize, PropagatingEffect<V>>> =
                (0..n_nodes).map(|_| BTreeMap::new()).collect();
            let mut processed = vec![false; n_nodes];

            for node in 0..n_nodes {
                // The start node is seeded, so its parents are ignored (pending stays 0).
                if !reachable[node] || node == round_start {
                    continue;
                }
                if let Ok(parents) = self.get_graph().inbound_edges(node) {
                    pending[node] = parents.filter(|p| reachable[*p]).count();
                }
            }

            // Ready set ordered by ascending node index — the canonical schedule.
            let mut ready: BTreeSet<usize> = BTreeSet::new();
            ready.insert(round_start);

            let mut last_effect = round_input.clone();

            while let Some(node) = ready.pop_first() {
                if processed[node] {
                    continue;
                }
                processed[node] = true;

                // Resolve this node's incoming effect from its wire slots.
                let incoming = if node == round_start {
                    round_input.clone()
                } else {
                    let parents = std::mem::take(&mut fired[node]);
                    match parents.len() {
                        0 => {
                            // Every parent resolved `Inactive`: this node is `Inactive`. It does not
                            // evaluate, and its out-wires resolve `Inactive` too (discard / dead path).
                            if let Ok(children) = self.get_graph().outbound_edges(node) {
                                for c in children {
                                    if reachable[c] && !processed[c] {
                                        pending[c] = pending[c].saturating_sub(1);
                                        if pending[c] == 0 {
                                            ready.insert(c);
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                        1 => {
                            // Join of one fired parent is the identity: pass its effect through.
                            parents.into_values().next().expect("len == 1")
                        }
                        _ => {
                            // Two or more fired parents: reduce them with the node's declared join
                            // over the labeled parent effects. (Errored parents never reach here — a
                            // node error short-circuits the whole traversal below.)
                            let causaloid = match self.get_causaloid(node) {
                                Some(c) => c,
                                None => {
                                    return PropagatingEffect::from_error(CausalityError(
                                        CausalityErrorEnum::Custom(format!(
                                            "Failed to get causaloid at index {node}"
                                        )),
                                    ));
                                }
                            };
                            let parent_effects = ParentEffects::new(parents);
                            let joined = if let Some(join_fn) = causaloid.context_join_fn() {
                                join_fn(&parent_effects, causaloid.context())
                            } else if let Some(join_fn) = causaloid.join_fn() {
                                join_fn(&parent_effects)
                            } else {
                                let keys: Vec<usize> = parent_effects.parent_indices().collect();
                                return PropagatingEffect::from_error(CausalityError(
                                    CausalityErrorEnum::Custom(format!(
                                        "Node {node} is a reconvergence with {} fired parents \
                                         {keys:?} but declares no join mechanism; use \
                                         Causaloid::new_join / new_with_context_join",
                                        keys.len()
                                    )),
                                ));
                            };
                            // The join is a left-zero for errors: a failed combine short-circuits.
                            if joined.is_err() {
                                return joined;
                            }
                            joined
                        }
                    }
                };

                let causaloid = match self.get_causaloid(node) {
                    Some(c) => c,
                    None => {
                        return PropagatingEffect::from_error(CausalityError(
                            CausalityErrorEnum::Custom(format!(
                                "Failed to get causaloid at index {node}"
                            )),
                        ));
                    }
                };

                let result_effect = causaloid.evaluate(&incoming);
                last_effect = result_effect.clone();

                // A node error short-circuits the whole traversal (left-zero), preserving logs.
                if result_effect.is_err() {
                    return result_effect;
                }

                match result_effect.command_target() {
                    // Adaptive reasoning: `RelayTo(target, sub)` ends this round and starts a fresh
                    // one at the target with the command's sub-program (single-level relay). The
                    // abandoned cone resolves `Inactive` implicitly — the round simply stops here.
                    Some(target_idx) => {
                        if !self.contains_causaloid(target_idx) {
                            let (_, state, context, logs) = last_effect.into_parts();
                            return PropagatingEffect::new(
                                Err(CausalityError(CausalityErrorEnum::Custom(format!(
                                    "RelayTo target causaloid with index {target_idx} not found in graph."
                                )))),
                                state,
                                context,
                                logs,
                            );
                        }
                        let logs = last_effect.logs().clone();
                        let relayed_effect = result_effect
                            .into_parts()
                            .0
                            .ok()
                            .and_then(CausalEffect::into_command)
                            .map(|(_, sub)| sub)
                            .unwrap_or_else(CausalEffect::none);
                        round_start = target_idx;
                        round_input = PropagatingEffect::new(Ok(relayed_effect), (), None, logs);
                        continue 'rounds;
                    }
                    // A value/`None` result fires: publish it to each child's wire slot.
                    None => {
                        let children = match self.get_graph().outbound_edges(node) {
                            Ok(c) => c,
                            Err(e) => {
                                let (_, state, context, logs) = last_effect.into_parts();
                                return PropagatingEffect::new(
                                    Err(CausalityError(CausalityErrorEnum::Custom(format!("{e}")))),
                                    state,
                                    context,
                                    logs,
                                );
                            }
                        };
                        for c in children {
                            if reachable[c] && !processed[c] {
                                fired[c].insert(node, result_effect.clone());
                                pending[c] = pending[c].saturating_sub(1);
                                if pending[c] == 0 {
                                    ready.insert(c);
                                }
                            }
                        }
                    }
                }
            }

            // Round complete: return the effect of the last node processed.
            return last_effect;
        }
    }
    /// Reasons over the shortest path between a start and stop cause using a monadic approach.
    ///
    /// It evaluates each node sequentially along the path. The `PropagatingEffect` returned by
    /// one causaloid becomes the input for the next causaloid in the path. If any node
    /// fails evaluation (i.e., returns a `PropagatingEffect` containing an error) or returns
    /// a `RelayTo` effect, the reasoning stops.
    ///
    /// # Arguments
    ///
    /// * `start_index` - The index of the start cause.
    /// * `stop_index` - The index of the stop cause.
    /// * `initial_effect` - The runtime effect to be passed as input to the first node's evaluation function.
    ///
    /// # Returns
    ///
    /// A `PropagatingEffect` representing the final aggregated monadic effect of the path traversal.
    /// If an error occurs or a `RelayTo` effect is encountered, that `PropagatingEffect` is returned immediately.
    fn evaluate_shortest_path_between_causes(
        &self,
        start_index: usize,
        stop_index: usize,
        initial_effect: &PropagatingEffect<V>,
    ) -> PropagatingEffect<V> {
        if !self.is_frozen() {
            return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                "Graph is not frozen. Call freeze() first".into(),
            )));
        }

        // Handle the single-node case explicitly before calling the pathfinder.
        if start_index == stop_index {
            let causaloid = match self.get_causaloid(start_index) {
                Some(c) => c,
                None => {
                    return PropagatingEffect::from_error(CausalityError(
                        CausalityErrorEnum::Custom(format!(
                            "Failed to get causaloid at index {start_index}"
                        )),
                    ));
                }
            };
            return causaloid.evaluate(initial_effect);
        }

        let path = match self.get_shortest_path(start_index, stop_index) {
            Ok(p) => p,
            Err(e) => {
                return PropagatingEffect::from_error(CausalityError(CausalityErrorEnum::Custom(
                    format!("{:?}", e),
                )));
            }
        };

        let mut current_effect = initial_effect.clone();

        for index in path {
            let causaloid = match self.get_causaloid(index) {
                Some(c) => c,
                None => {
                    return PropagatingEffect::from_error(CausalityError(
                        CausalityErrorEnum::Custom(format!(
                            "Failed to get causaloid at index {index}"
                        )),
                    ));
                }
            };

            // Evaluate the current cause with the effect propagated from the previous node.
            current_effect = causaloid.evaluate(&current_effect);

            // If an error occurred, propagate it and stop.
            if current_effect.is_err() {
                return current_effect;
            }

            // If a RelayTo command is returned, stop the shortest path traversal and return it
            if current_effect.command_target().is_some() {
                return current_effect;
            }
        }

        // If the loop completes, all nodes on the path were successfully evaluated.
        current_effect
    }
}
