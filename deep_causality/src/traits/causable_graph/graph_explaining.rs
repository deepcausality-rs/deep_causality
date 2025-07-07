/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Causable, CausableGraph, CausalityGraphError};
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

        let mut stack = Vec::with_capacity(self.size());
        let mut explanation = String::new();

        let cause = self
            .get_causaloid(start_index)
            .expect("Failed to get causaloid");

        let explain = match cause.explain() {
            Ok(res) => res,
            Err(e) => return Err(CausalityGraphError(e.to_string())),
        };

        append_string(&mut explanation, &explain);

        // get all neighbors of the start causaloid
        let neighbors = match self.get_graph().outbound_edges(start_index) {
            Ok(neighbors) => neighbors,
            Err(e) => return Err(CausalityGraphError(e.to_string())),
        };

        stack.push(neighbors);

        while let Some(children) = stack.last_mut() {
            if let Some(child) = children.next() {
                let cause = self.get_causaloid(child).expect("Failed to get causaloid");

                append_string(&mut explanation, &cause.explain().unwrap());

                if child == stop_index {
                    return Ok(explanation);
                } else {
                    let neighbors = match self.get_graph().outbound_edges(child) {
                        Ok(neighbors) => neighbors,
                        Err(e) => return Err(CausalityGraphError(e.to_string())),
                    };

                    stack.push(neighbors);
                }
            } else {
                stack.pop();
            }
        }

        Ok(explanation)
    }

    /// Explains the full causal graph from the root node to the last node.
    ///
    /// Checks that the graph is not empty and contains a root node.
    ///
    /// Gets the root node index and last node index.
    ///
    /// Calls explain_from_to_cause() with the root and last node indices
    /// to generate the full explanation.
    ///
    /// Returns:
    /// - Ok(String): The full graph explanation if successful
    /// - Err(CausalityGraphError): If graph is empty or lacks a root node
    ///
    fn explain_all_causes(&self) -> Result<String, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.contains_root_causaloid() {
            return Err(CausalityGraphError(
                "Graph does not contains root causaloid".into(),
            ));
        }

        // These is safe as we have tested above that these exists
        let start_index = self.get_root_index().expect("Root causaloid not found.");
        let stop_index = self.size() - 1;

        match self.explain_from_to_cause(start_index, stop_index) {
            Ok(explanation) => Ok(explanation),
            Err(e) => Err(e),
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

        let mut explanation = String::new();

        for index in shortest_path {
            let cause = self.get_causaloid(index).expect("Failed to get causaloid");

            append_string(&mut explanation, &cause.explain().unwrap());
        }

        Ok(explanation)
    }
}

/// Appends a string to another string with newlines before and after.
///
/// s1: The string to append to
/// s2: The string to append
///
/// Inserts a newline, then the s2 string formatted with a bullet point,
/// then another newline before returning the modified s1.
///
/// This allows cleanly appending explain() strings with spacing.
///
fn append_string<'l>(s1: &'l mut String, s2: &'l str) -> &'l str {
    s1.push('\n');
    s1.push_str(format!(" * {s2}").as_str());
    s1.push('\n');
    s1
}
