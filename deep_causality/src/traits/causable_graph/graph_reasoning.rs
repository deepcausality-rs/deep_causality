/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::VecDeque;

use ultragraph::*;

use crate::{Causable, CausableGraph, CausalityError, PropagatingEffect};

/// Describes signatures for causal reasoning and explaining
/// in causality hyper graph.
pub trait CausableGraphReasoning<T>: CausableGraph<T>
where
    T: Causable + PartialEq + Clone,
{
    /// Evaluates a single, specific causaloid within the graph by its index.
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
    /// The `PropagatingEffect` from the evaluated causaloid, or a `CausalityError` if
    /// the node is not found or the evaluation fails.
    fn evaluate_single_cause(
        &self,
        index: usize,
        effect: &PropagatingEffect,
    ) -> Result<PropagatingEffect, CausalityError> {
        let cause = self.get_causaloid(index).ok_or_else(|| {
            CausalityError(format!("Causaloid with index {index} not found in graph"))
        })?;

        cause.evaluate(effect)
    }

    /// Reasons over a subgraph by traversing all nodes reachable from a given start index.
    ///
    /// This method performs a Breadth-First Search (BFS) traversal of all descendants
    /// of the `start_index`. It calls `evaluate` on each node and uses the resulting
    /// `PropagatingEffect` to decide whether to continue the traversal down that path.
    ///
    /// # Arguments
    ///
    /// * `start_index` - The index of the node to start the traversal from.
    /// * `effect` - The runtime effect to be passed to each node's evaluation function.
    ///
    /// # Returns
    ///
    /// * `Ok(PropagatingEffect::Halting)` if any node in the traversal returns `Halting`.
    /// * `Ok(PropagatingEffect::Deterministic(true))` if the traversal completes.
    /// * `Err(CausalityError)` if the graph is not frozen, a node is missing, or an evaluation fails.
    fn evaluate_subgraph_from_cause(
        &self,
        start_index: usize,
        effect: &PropagatingEffect,
    ) -> Result<PropagatingEffect, CausalityError> {
        if !self.is_frozen() {
            return Err(CausalityError(
                "Graph is not frozen. Call freeze() first".into(),
            ));
        }

        if !self.contains_causaloid(start_index) {
            return Err(CausalityError(format!(
                "Graph does not contain start causaloid with index {start_index}"
            )));
        }

        let mut queue = VecDeque::with_capacity(self.number_nodes());
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back(start_index);
        visited[start_index] = true;

        while let Some(current_index) = queue.pop_front() {
            let cause = self.get_causaloid(current_index).ok_or_else(|| {
                CausalityError(format!("Failed to get causaloid at index {current_index}"))
            })?;

            // Evaluate the current cause using the new unified method.
            // The same `effect` is passed to each node, and the node's CausalFn
            // is responsible for extracting the data it needs from the effect map.
            let effect = cause.evaluate(effect)?;

            match effect {
                // If any cause halts, the entire subgraph reasoning halts immediately.
                PropagatingEffect::Halting => return Ok(PropagatingEffect::Halting),

                // If the cause is deterministically false, we stop traversing this path,
                // but the overall subgraph reasoning does not fail or halt.
                PropagatingEffect::Deterministic(false) => continue,

                // For any other propagating effect (true, probabilistic, etc.),
                // we continue the traversal to its children.
                _ => {
                    // The cause is valid, so add its children to the queue.
                    let children = self.get_graph().outbound_edges(current_index)?;
                    for child_index in children {
                        if !visited[child_index] {
                            visited[child_index] = true;
                            queue.push_back(child_index);
                        }
                    }
                }
            }
        }

        // If the loop completes without halting, the reasoning is considered successful.
        Ok(PropagatingEffect::Deterministic(true))
    }

    /// Reasons over the shortest path between a start and stop cause.
    ///
    /// It evaluates each node along the path. If any node fails evaluation or returns
    /// a non-propagating effect, the reasoning stops.
    ///
    /// # Arguments
    ///
    /// * `start_index` - The index of the start cause.
    /// * `stop_index` - The index of the stop cause.
    /// * `effect` - The runtime effect to be passed to each node's evaluation function.
    ///
    /// # Returns
    ///
    /// * `Ok(PropagatingEffect::Halting)` if any node on the path returns `Halting`.
    /// * `Ok(PropagatingEffect::Deterministic(false))` if any node returns `Deterministic(false)`.
    /// * `Ok(PropagatingEffect::Deterministic(true))` if all nodes on the path propagate successfully.
    /// * `Err(CausalityError)` if the path cannot be found or an evaluation fails.
    fn evaluate_shortest_path_between_causes(
        &self,
        start_index: usize,
        stop_index: usize,
        effect: &PropagatingEffect,
    ) -> Result<PropagatingEffect, CausalityError> {
        if !self.is_frozen() {
            return Err(CausalityError(
                "Graph is not frozen. Call freeze() first".into(),
            ));
        }

        // Handle the single-node case explicitly before calling the pathfinder.
        if start_index == stop_index {
            let cause = self.get_causaloid(start_index).ok_or_else(|| {
                CausalityError(format!("Failed to get causaloid at index {start_index}"))
            })?;
            return cause.evaluate(effect);
        }

        // get_shortest_path will handle checks for missing nodes.
        let path = self.get_shortest_path(start_index, stop_index)?;

        for index in path {
            let cause = self.get_causaloid(index).ok_or_else(|| {
                CausalityError(format!("Failed to get causaloid at index {index}"))
            })?;

            let effect = cause.evaluate(effect)?;

            match effect {
                // If any node on the path is false or halts, the entire path fails.
                PropagatingEffect::Deterministic(false) | PropagatingEffect::Halting => {
                    return Ok(effect);
                }
                // Otherwise, continue to the next node.
                _ => continue,
            }
        }

        // If the loop completes, all nodes on the path were successfully evaluated.
        Ok(PropagatingEffect::Deterministic(true))
    }
}
