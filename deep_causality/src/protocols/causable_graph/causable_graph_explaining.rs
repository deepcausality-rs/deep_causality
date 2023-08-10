// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{Causable, CausableGraph, CausalityGraphError};
use crate::protocols::causable_graph::NodeIndex;
use crate::utils::reasoning_utils;

pub trait CausableGraphExplaining<T> : CausableGraph<T>
    where
        T: Causable + PartialEq,
{

    fn explain_from_to_cause(
        &self,
        start_index: NodeIndex,
        stop_index: NodeIndex,
    )
        -> Result<String, CausalityGraphError>;

    /// Explains the line of reasoning across the entire graph.
    /// Returns: String representing the explanation or an error
    fn explain_all_causes(
        &self
    )
        -> Result<String, CausalityGraphError>
    {
        let root_index = self.get_root_index().expect("Root causaloid not found.");
        let start_index = NodeIndex::new(root_index);
        let stop_index = match self.get_last_index() {
            Ok(stop_index) => stop_index,
            Err(e) => return Err(e),
        };

        let stop_index = NodeIndex::new(stop_index);

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
    )
        -> Result<String, CausalityGraphError>
    {
        let stop_index = match self.get_last_index() {
            Ok(stop_index) => stop_index,
            Err(e) => return Err(e),
        };

        let start_index = NodeIndex::new(start_index);
        let stop_index = NodeIndex::new(stop_index);

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
    )
        -> Result<String, CausalityGraphError>
    {
        let start_index = NodeIndex::new(start_index);
        let stop_index = NodeIndex::new(stop_index);

        if self.is_empty()
        {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.contains_causaloid(start_index.index()) {
            return Err(CausalityGraphError("Graph does not contains start causaloid".into()));
        }

        if !self.contains_causaloid(stop_index.index()) {
            return Err(CausalityGraphError("Graph does not contains stop causaloid".into()));
        }

        let shortest_path = match self.get_shortest_path(start_index, stop_index) {
            Ok(shortest_path) => shortest_path,
            Err(e) => return Err(e)
        };

        let mut explanation = String::new();

        for index in shortest_path {
            let cause = self.get_causaloid(index.index())
                .expect("Failed to get causaloid");

            reasoning_utils::append_string(&mut explanation, &cause.explain().unwrap());
        }

        Ok(explanation)
    }
}

