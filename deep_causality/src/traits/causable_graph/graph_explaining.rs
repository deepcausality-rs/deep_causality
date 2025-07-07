/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Causable, CausableGraph, CausalityError, CausalityGraphError};
use std::collections::VecDeque;
use ultragraph::GraphTraversal;

/// The CausableGraphExplaining trait provides methods to generate
/// natural language explanations from a causal graph.
///
/// It requires the graph to implement CausableGraph, where nodes
/// are Causable (can explain themselves).
///
/// Provides methods to:
///
/// - Explain between two node indices
/// - Explain the full graph
/// - Explain a subgraph
/// - Explain the shortest path between nodes
///
/// Uses a depth-first search to traverse the graph and collect
/// explanations.
///
/// The explain_from_to_cause() method is the core implementation
/// that supports the other methods.
///
#[allow(clippy::type_complexity)]
pub trait CausableGraphExplaining<T>: CausableGraph<T>
where
    T: Causable + PartialEq + Clone,
{
    /// Generates an explanation by traversing the graph from start_index to stop_index.
    ///
    /// Uses a depth-first search to visit all nodes on the path.
    ///
    /// start_index: The index of the starting node
    /// stop_index: The index of the target node
    ///
    /// Returns:
    /// - Ok(String): The concatenated explanation if successful
    /// - Err(CausalityGraphError): If indices are invalid or traversal fails
    ///
    /// Gets the explanation from start node.
    /// Gets start node's neighbors and adds them to stack.
    ///
    /// While stack is not empty:
    /// - Get next node from stack top
    /// - Get its explanation and append
    /// - If node is stop_index, return result
    /// - Else get node's neighbors and push to stack
    ///
    /// This traverses all nodes from start to stop depth-first.
    ///
    fn explain_from_to_cause(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<String, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.is_frozen() {
            return Err(CausalityGraphError(
                "Graph is not frozen. Call g.freeze() first".to_string(),
            ));
        }

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError(
                "Graph does not contains start causaloid".into(),
            ));
        }

        if !self.contains_causaloid(stop_index) {
            return Err(CausalityGraphError(
                "Graph does not contains stop causaloid".into(),
            ));
        }

        let mut explanation = String::new();
        let mut queue = VecDeque::with_capacity(self.number_nodes());
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back(start_index);
        visited[start_index] = true;

        while let Some(current_index) = queue.pop_front() {
            if let Some(cause) = self.get_causaloid(current_index) {
                if let Ok(single_explanation) = cause.explain() {
                    append_string(&mut explanation, &single_explanation);
                }

                // If we've reached the stop index for this path, do not explore its children.
                if current_index == stop_index {
                    continue;
                }

                // Otherwise, add all unvisited children to the queue for processing.
                let children = self.get_graph().outbound_edges(current_index)?;
                for child_index in children {
                    if !visited[child_index] {
                        visited[child_index] = true;
                        queue.push_back(child_index);
                    }
                }
            }
        }

        if explanation.is_empty() {
            Ok(
                "No nodes in the specified sub-graph have been evaluated or produced an explainable effect."
                    .to_string(),
            )
        } else {
            Ok(explanation)
        }
    }

    /// Explains the full causal graph by traversing all reachable nodes from the root.
    ///
    /// This method performs a Breadth-First Search (BFS) starting from the root node
    /// to ensure every node in every causal pathway is visited. For each visited node,
    /// it attempts to get its explanation. It only includes explanations from nodes
    /// where `explain()` succeeds (i.e., the node has an effect to report).
    ///
    /// # Returns
    ///
    /// A `Result` containing a single, formatted string of all available explanations,
    /// or a `CausalityError` if the graph is empty or has no root.
    fn explain_all_causes(&self) -> Result<String, CausalityError> {
        if self.is_empty() {
            return Ok("The causal graph is empty.".to_string());
        }

        let start_index = self.get_root_index().ok_or_else(|| {
            CausalityError("Cannot explain all causes: Graph has no root node.".into())
        })?;

        let mut all_explanations = String::new();
        let mut queue = VecDeque::with_capacity(self.number_nodes());
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back(start_index);
        visited[start_index] = true;

        while let Some(current_index) = queue.pop_front() {
            if let Some(cause) = self.get_causaloid(current_index) {
                // cause.explain() returns a Result. We only care about the Ok variants.
                // If a node hasn't been evaluated, its explain() will return Err,
                // which we correctly and safely ignore.
                if let Ok(explanation) = cause.explain() {
                    append_string(&mut all_explanations, &explanation);
                }

                // Add all unvisited children to the queue for processing.
                let children = self.get_graph().outbound_edges(current_index)?;
                for child_index in children {
                    if !visited[child_index] {
                        visited[child_index] = true;
                        queue.push_back(child_index);
                    }
                }
            }
        }

        if all_explanations.is_empty() {
            Ok(
                "No nodes in the graph have been evaluated or produced an explainable effect."
                    .to_string(),
            )
        } else {
            Ok(all_explanations)
        }
    }

    /// Explains the line of reasoning across a subgraph starting from a given node index until
    /// the end of the graph.
    ///
    /// start_index: The index of the starting node
    ///
    /// Gets the index of the last node.
    ///
    /// Calls explain_from_to_cause() with the start index and last index
    /// to generate the subgraph explanation.
    ///
    /// Returns:
    /// - Ok(String): The subgraph explanation if successful
    /// - Err(CausalityGraphError): If graph is empty
    ///
    fn explain_subgraph_from_cause(
        &self,
        start_index: usize,
    ) -> Result<String, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        let stop_index = self.size() - 1;

        match self.explain_from_to_cause(start_index, stop_index) {
            Ok(explanation) => Ok(explanation),
            Err(e) => Err(e),
        }
    }

    /// Explains the line of reasoning of the shortest path
    /// between a start and stop cause.
    ///
    /// start_index: The start node index
    /// stop_index: The target node index
    ///
    /// Gets the shortest path between the indices using get_shortest_path().
    ///
    /// Iterates the path indices:
    /// - Get each node
    /// - Append its explanation to the result
    ///
    /// Returns:
    /// - Ok(String): The concatenated shortest path explanation
    /// - Err(CausalityGraphError): If indices invalid or no path found
    fn explain_shortest_path_between_causes(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<String, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.is_frozen() {
            return Err(CausalityGraphError(
                "Graph is not frozen. Call g.freeze() first".to_string(),
            ));
        }

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError(
                "Graph does not contains start causaloid".into(),
            ));
        }

        if !self.contains_causaloid(stop_index) {
            return Err(CausalityGraphError(
                "Graph does not contains stop causaloid".into(),
            ));
        }

        let shortest_path = self.get_shortest_path(start_index, stop_index)?;
        if shortest_path.is_empty() {
            return Err(CausalityGraphError(format!(
                "No shortest path found between start Causaloid: {start_index} and stop Causaloid: {stop_index} "
            )));
        }

        let mut explanation = String::new();

        for index in shortest_path {
            // Safely get the causaloid, returning an error if it's missing.
            let cause = self.get_causaloid(index).ok_or_else(|| {
                CausalityGraphError(format!("Failed to get causaloid at index {index}"))
            })?;

            // Safely get the explanation, propagating any error.
            let single_explanation = cause.explain()?;
            append_string(&mut explanation, &single_explanation);
        }

        Ok(explanation)
    }
}

/// Appends a string to another string with a formatted bullet point.
fn append_string(s1: &mut String, s2: &str) {
    if !s1.is_empty() {
        s1.push('\n');
    }
    s1.push_str(&format!(" * {s2}"));
}
