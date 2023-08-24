// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use ultragraph::prelude::*;

use crate::prelude::{Causable, CausableGraph, CausalityGraphError};

pub trait CausableGraphExplaining<T>: CausableGraph<T>
where
    T: Causable + PartialEq,
{
    fn explain_from_to_cause(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<String, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
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
        let neighbors = match self.get_graph().outgoing_edges(start_index) {
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
                    let neighbors = match self.get_graph().outgoing_edges(child) {
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

    /// Explains the line of reasoning across the entire graph.
    /// Returns: String representing the explanation or an error
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
    /// index: NodeIndex - index of the starting node
    /// Returns: String representing the explanation or an error
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

    /// Explains the line of reasoning of the shortest sub-graph
    /// between a start and stop cause.
    /// start_index: NodeIndex - index of the start cause
    /// stop_index: NodeIndex - index of the stop cause
    /// Returns: String representing the explanation or an error
    fn explain_shortest_path_between_causes(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<String, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
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

        let shortest_path = match self.get_shortest_path(start_index, stop_index) {
            Ok(shortest_path) => shortest_path,
            Err(e) => return Err(e),
        };

        let mut explanation = String::new();

        for index in shortest_path {
            let cause = self.get_causaloid(index).expect("Failed to get causaloid");

            append_string(&mut explanation, &cause.explain().unwrap());
        }

        Ok(explanation)
    }
}

fn append_string<'l>(s1: &'l mut String, s2: &'l str) -> &'l str {
    s1.push('\n');
    s1.push_str(format!(" * {}", s2).as_str());
    s1.push('\n');
    s1
}
