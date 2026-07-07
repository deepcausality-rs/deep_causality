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
use std::collections::VecDeque;
use std::fmt::Debug;
use ultragraph::GraphTraversal;

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

        let mut queue =
            VecDeque::<(usize, PropagatingProcess<V, S, C>)>::with_capacity(self.number_nodes());
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back((start_index, initial_effect.clone()));
        visited[start_index] = true;

        let mut last_propagated = initial_effect.clone();

        while let Some((current_index, incoming)) = queue.pop_front() {
            let causaloid = match self.get_causaloid(current_index) {
                Some(c) => c,
                None => {
                    return raise_from(
                        CausalityError(CausalityErrorEnum::Custom(format!(
                            "Failed to get causaloid at index {current_index}"
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

            // Interpret the output effect (the `Free::fold` handler, inlined for `RelayTo`).
            match result.command_target() {
                Some(target_idx) => {
                    visited.fill(false);
                    queue.clear();

                    if !self.contains_causaloid(target_idx) {
                        return raise_from(
                            CausalityError(CausalityErrorEnum::Custom(format!(
                                "RelayTo target causaloid with index {target_idx} not found in graph."
                            ))),
                            &last_propagated,
                        );
                    }

                    visited[target_idx] = true;

                    // The relayed input is the command's sub-program itself: a value/`None` is fed to
                    // the target as-is, and a NESTED command is preserved (not collapsed to `None`) so
                    // the engine's next iteration relays it in turn — recursion via the BFS loop. The
                    // relaying node's state, context, and logs are carried forward.
                    let relayed_effect = result
                        .into_parts()
                        .0
                        .ok()
                        .and_then(CausalEffect::into_command)
                        .map(|(_, sub)| sub)
                        .unwrap_or_else(CausalEffect::none);
                    let relayed: PropagatingProcess<V, S, C> = PropagatingProcess::new(
                        Ok(relayed_effect),
                        last_propagated.state().clone(),
                        last_propagated.context().clone(),
                        last_propagated.logs().clone(),
                    );
                    queue.push_back((target_idx, relayed));
                }
                None => {
                    let children = match self.get_graph().outbound_edges(current_index) {
                        Ok(c) => c,
                        Err(e) => {
                            return raise_from(
                                CausalityError(CausalityErrorEnum::Custom(format!("{e}"))),
                                &last_propagated,
                            );
                        }
                    };
                    for child_index in children {
                        if !visited[child_index] {
                            visited[child_index] = true;
                            queue.push_back((child_index, result.clone()));
                        }
                    }
                }
            }
        }

        last_propagated
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
