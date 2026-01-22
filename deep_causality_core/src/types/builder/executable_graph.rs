/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ControlFlowProtocol;
use crate::ExecutableNode;
use crate::errors::graph_error::GraphError;
#[cfg(feature = "alloc")]
use alloc::collections::VecDeque;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// The finalized artifact ready for execution.
pub struct ExecutableGraph<P> {
    pub(crate) nodes: Vec<ExecutableNode<P>>,
    // Adjacency list: index -> list of target node indices
    pub(crate) adjacency: Vec<Vec<usize>>,
}

impl<P: ControlFlowProtocol> ExecutableGraph<P> {
    /// Executes the graph with an initial input using an iterative BFS approach.
    ///
    /// # Arguments
    /// * `input` - The initial protocol message to start execution.
    /// * `start_node` - The index of the starting node.
    /// * `max_steps` - Safety limit to prevent infinite loops in cyclic graphs.
    /// * `queue` - A mutable reference to a VecDeque to be used as the execution queue.
    ///
    /// The caller is responsible for pre-allocating queue capacity to avoid runtime allocations.
    ///
    /// # Returns
    /// * `Result<P, GraphError>` - The final result of the execution or an error.
    pub fn execute(
        &self,
        input: P,
        start_node: usize,
        max_steps: usize,
        queue: &mut VecDeque<(usize, P)>,
    ) -> Result<P, GraphError> {
        if start_node >= self.nodes.len() {
            return Err(GraphError::StartNodeOutOfBounds(start_node));
        }

        queue.clear();
        queue.push_back((start_node, input));

        let mut steps = 0;
        let mut last_result = None;

        // Track nodes currently in the queue to prevent duplicate entries (combining fan-in).
        // We use 'enqueued' state instead of 'visited' complexity to allow re-visiting nodes (cycles)
        // once they have been processed, while preventing the same node from accumulating
        // multiple times in the queue during a single wave.
        let mut enqueued = vec![false; self.nodes.len()];
        enqueued[start_node] = true;

        while let Some((node_idx, node_input)) = queue.pop_front() {
            // Node is processed, remove from enqueued state so it can be re-visited if the graph cycles
            enqueued[node_idx] = false;

            if steps >= max_steps {
                return Err(GraphError::MaxStepsExceeded(max_steps));
            }

            // Execute the current node's logic
            let node = &self.nodes[node_idx];
            let output = (node.func)(node_input);

            // Store result (cloned because we might need it for multiple neighbors)
            last_result = Some(output.clone());

            // Propagate to neighbors
            if let Some(neighbors) = self.adjacency.get(node_idx) {
                for &neighbor_idx in neighbors {
                    // Only push if not already waiting in the queue
                    if !enqueued[neighbor_idx] {
                        queue.push_back((neighbor_idx, output.clone()));
                        enqueued[neighbor_idx] = true;
                    }
                }
            }

            steps += 1;
        }

        last_result.ok_or(GraphError::GraphExecutionProducedNoResult)
    }
}

impl<P: ControlFlowProtocol> ExecutableGraph<P> {
    pub fn nodes(&self) -> &Vec<ExecutableNode<P>> {
        &self.nodes
    }

    pub fn adjacency(&self) -> &Vec<Vec<usize>> {
        &self.adjacency
    }
}
