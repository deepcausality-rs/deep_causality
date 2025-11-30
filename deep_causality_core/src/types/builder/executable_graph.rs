/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ControlFlowProtocol;
use crate::ExecutableNode;
use crate::types::builder::execution_graph_error::GraphError;
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

        while let Some((node_idx, node_input)) = queue.pop_front() {
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
                    queue.push_back((neighbor_idx, output.clone()));
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
