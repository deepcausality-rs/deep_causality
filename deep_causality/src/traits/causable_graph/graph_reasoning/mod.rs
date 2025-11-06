/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::VecDeque;

use ultragraph::*;

use crate::{CausableGraph, CausalMonad, CausalityError, MonadicCausable, PropagatingEffect};

/// Describes signatures for causal reasoning and explaining
/// in causality hyper graph.
pub trait CausableGraphReasoning<T>: CausableGraph<T>
where
    T: MonadicCausable<crate::CausalMonad> + PartialEq + Clone,
{
    /// Evaluates a single, specific causaloid within the graph by its index.
    ///
    /// This is a convenience method that locates the causaloid and calls its `evaluate_monadic` method.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the causaloid to evaluate.
    /// * `effect` - The runtime effect to be passed to the node's evaluation function.
    ///
    /// # Returns
    ///
    /// The `PropagatingEffect` from the evaluated causaloid, or a `PropagatingEffect` with an error if
    /// the node is not found or the evaluation fails.
    fn evaluate_single_cause(
        &self,
        index: usize,
        effect: PropagatingEffect,
    ) -> PropagatingEffect {
        let cause = match self.get_causaloid(index) {
            Some(c) => c,
            None => return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(CausalityError(format!("Causaloid with index {index} not found in graph"))),
                logs: effect.logs,
            },
        };

        cause.evaluate_monadic(effect)
    }

    /// Reasons over a subgraph by traversing all nodes reachable from a given start index.
    ///
    /// This method performs a Breadth-First Search (BFS) traversal of all descendants
    /// of the `start_index`. The `PropagatingEffect` is passed sequentially:
    /// the output effect of a parent node becomes the input effect for its child node.
    /// The traversal continues as long as no `CausalityError` is returned.
    ///
    /// ## Adaptive Reasoning
    ///
    /// If a `Causaloid` returns a `PropagatingEffect::RelayTo(target_index, inner_effect)`,
    /// the BFS traversal dynamically jumps to  `target_index`, and `inner_effect` becomes
    /// the new input for the relayed path. This enables *adaptive reasoning* conditional to the deciding
    /// causaloid. To illustrate adaptive reasoning, an example clinical patent risk model may operate
    /// very differently for patients with normal blood pressure compared to high blood pressure patients.
    /// Therefore, two highly specialized models are defined and a dedicated dispatch causaloid.
    /// The dispatch causaloid analyses blood pressure and then, conditional on its finding, dispatches
    /// further reasoning to the matching model i.e. a dedicated sub-graph. Ensure that all possible
    /// values of  target_index exists in the graph before implementing adaptive reasoning.
    /// For more details, see section 5.10.3 Adaptive Reasoning in The EPP reference paper:
    /// <https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf>
    ///
    /// # Arguments
    ///
    /// * `start_index` - The index of the node to start the traversal from.
    /// * `initial_effect` - The initial runtime effect to be passed to the starting node's evaluation function.
    ///
    /// # Returns
    ///
    /// * `Ok(PropagatingEffect)`: The final `PropagatingEffect` from the last successfully evaluated node
    ///   in the main traversal path. `Deterministic(false)` now propagates and does not implicitly halt propagation.
    ///   Only a `Causaloid` returning a `CausalityError` will abort the traversal.
    /// * `Err(CausalityError)` if the graph is not frozen, a node is missing, a RelayTo target cannot be found or an evaluation fails.
    fn evaluate_subgraph_from_cause(
        &self,
        start_index: usize,
        initial_effect: PropagatingEffect,
    ) -> PropagatingEffect {
        if !self.is_frozen() {
            return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(CausalityError(
                    "Graph is not frozen. Call freeze() first".into(),
                )),
                logs: initial_effect.logs,
            };
        }

        if !self.contains_causaloid(start_index) {
            return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(CausalityError(format!(
                    "Graph does not contain start causaloid with index {start_index}"
                ))),
                logs: initial_effect.logs,
            };
        }

        // Queue stores (node_index, incoming_effect_for_this_node)
        let mut queue = VecDeque::<(usize, PropagatingEffect)>::with_capacity(self.number_nodes());
        let mut visited = vec![false; self.number_nodes()];

        // Initialize the queue with the starting node and the initial effect
        queue.push_back((start_index, initial_effect.clone()));
        visited[start_index] = true;

        // This will hold the effect of the last successfully processed node.
        // It's initialized with the initial_effect, in case the start_index node
        // itself prunes the path or is the only node.
        let mut last_propagated_effect = initial_effect.clone();

        while let Some((current_index, incoming_effect)) = queue.pop_front() {
            let cause = match self.get_causaloid(current_index) {
                Some(c) => c,
                None => return PropagatingEffect {
                    value: crate::EffectValue::None,
                    error: Some(CausalityError(format!("Failed to get causaloid at index {current_index}"))),
                    logs: incoming_effect.logs,
                },
            };

            // Evaluate the current cause using the incoming_effect.
            let result_effect = cause.evaluate_monadic(incoming_effect);

            if let Some(err) = result_effect.error {
                return PropagatingEffect {
                    value: crate::EffectValue::None,
                    error: Some(err),
                    logs: result_effect.logs,
                };
            }

            // Update the last_propagated_effect with the result of the current node's evaluation.
            // This ensures the function returns the effect of the last node on the path.
            last_propagated_effect = result_effect.clone();

            match result_effect.value {
                // Adaptive reasoning:
                // The Causaloid itself determines the next step in the reasoning process
                // conditional on its reasoning outcome. Based on its own internal logic,
                // a Causaloid then dynamically dispatches the flow of causality
                // to another Causaloid in the graph, enabling adaptive reasoning.
                crate::EffectValue::RelayTo(target_index, inner_effect) => {
                    // If a RelayTo effect is returned, clear the queue and add the target_index
                    // with the inner_effect as the new starting point for traversal.
                    queue.clear();

                    // Validate target_index before proceeding
                    if !self.contains_causaloid(target_index) {
                        return PropagatingEffect {
                            value: crate::EffectValue::None,
                            error: Some(CausalityError(format!(
                                "RelayTo target causaloid with index {target_index} not found in graph."
                            ))),
                            logs: result_effect.logs,
                        };
                    }

                    if !visited[target_index] {
                        visited[target_index] = true;
                        queue.push_back((target_index, *inner_effect));
                    }
                    // Update last_propagated_effect to reflect the effect of the relayed node.
                    // This is already handled by the line above: last_propagated_effect = result_effect.clone();
                }
                _ => {
                    // Only a CausalityError returned from cause.evaluate() will abort the traversal.
                    let children = match self.get_graph().outbound_edges(current_index) {
                        Ok(c) => c,
                        Err(e) => return PropagatingEffect {
                            value: crate::EffectValue::None,
                            error: Some(CausalityError(format!("{e}"))),
                            logs: result_effect.logs,
                        },
                    };
                    for child_index in children {
                        if !visited[child_index] {
                            visited[child_index] = true;
                            // Pass the result_effect of the current node to its children.
                            queue.push_back((child_index, result_effect.clone()));
                        }
                    }
                }
            }
        }

        // If the loop completes, return the effect of the last node processed.
        last_propagated_effect
    }

    /// Reasons over the shortest path between a start and stop cause.
    ///
    /// It evaluates each node sequentially along the path. The `PropagatingEffect` returned by
    /// one causaloid becomes the input for the next causaloid in the path. If any node
    /// fails evaluation or returns a non-propagating effect that prunes the path, the reasoning stops.
    ///
    /// If a `Causaloid` returns a `PropagatingEffect::RelayTo(target_index, inner_effect)`,
    /// the shortest path traversal is immediately interrupted, and the `RelayTo` effect
    /// is returned to the caller, signaling a dynamic redirection. The runtime behavior differs
    /// from `evaluate_subgraph_from_cause` because a shortest path is assumed to be a fixed path
    /// and thus RelayTo is not supposed to happen in the middle of the path. Therefore, the
    /// call-site must handle the occurrence i.e. when its a known final effect.
    /// For more details, see section 5.10.3 Adaptive Reasoning in The EPP reference paper:
    /// <https://github.com/deepcausality-rs/papers/blob/main/effect_propagation_process/epp.pdf>
    ///
    /// # Arguments
    ///
    /// * `start_index` - The index of the start cause.
    /// * `stop_index` - The index of the stop cause.
    /// * `initial_effect` - The runtime effect to be passed as input to the first node's evaluation function.
    ///
    /// # Returns
    ///
    /// * `Ok(PropagatingEffect)`: The final `PropagatingEffect` from the last evaluated node on the path.
    ///   If a `RelayTo` effect is encountered, that effect is returned immediately.
    /// * `Err(CausalityError)` if the path cannot be found or an evaluation fails.
    fn evaluate_shortest_path_between_causes(
        &self,
        start_index: usize,
        stop_index: usize,
        initial_effect: PropagatingEffect,
    ) -> PropagatingEffect {
        if !self.is_frozen() {
            return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(CausalityError(
                    "Graph is not frozen. Call freeze() first".into(),
                )),
                logs: initial_effect.logs,
            };
        }

        // Handle the single-node case explicitly before calling the pathfinder.
        if start_index == stop_index {
            let cause = match self.get_causaloid(start_index) {
                Some(c) => c,
                None => return PropagatingEffect {
                    value: crate::EffectValue::None,
                    error: Some(CausalityError(format!("Failed to get causaloid at index {start_index}"))),
                    logs: initial_effect.logs,
                },
            };
            return cause.evaluate_monadic(initial_effect);
        }

        // get_shortest_path will handle checks for missing nodes.
        let path = match self.get_shortest_path(start_index, stop_index) {
            Ok(p) => p,
            Err(e) => return PropagatingEffect {
                value: crate::EffectValue::None,
                error: Some(e),
                logs: initial_effect.logs,
            },
        };

        let mut current_effect = initial_effect.clone();

        for index in path {
            let cause = match self.get_causaloid(index) {
                Some(c) => c,
                None => return PropagatingEffect {
                    value: crate::EffectValue::None,
                    error: Some(CausalityError(format!("Failed to get causaloid at index {index}"))),
                    logs: current_effect.logs,
                },
            };

            // Evaluate the current cause with the effect propagated from the previous node.
            // Then, overwrite the current_effect with the result of the evaluation, which then
            // serves as the input for the next node.
            // For normal traversal, a CausalityError returned from cause.evaluate() will abort the traversal.
            current_effect = cause.evaluate_monadic(current_effect);

            if let Some(err) = current_effect.error {
                return PropagatingEffect {
                    value: crate::EffectValue::None,
                    error: Some(err),
                    logs: current_effect.logs,
                };
            }

            // If a RelayTo effect is returned, stop the shortest path traversal and return it
            // because it breaks the assumption of a fixed shortest path.
            if let crate::EffectValue::RelayTo(_, _) = current_effect.value {
                return current_effect;
            }
        }

        // If the loop completes, all nodes on the path were successfully evaluated.
        current_effect
    }
}
