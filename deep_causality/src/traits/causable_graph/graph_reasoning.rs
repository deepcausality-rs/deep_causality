/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::collections::{HashMap, VecDeque};

use ultragraph::*;

use crate::errors::CausalityGraphError;
use crate::prelude::{Causable, CausableGraph, IdentificationValue, NumericalValue};
use crate::traits::causable_graph::graph_reasoning_utils;

/// Describes signatures for causal reasoning and explaining
/// in causality hyper graph.
pub trait CausableGraphReasoning<T>: CausableGraph<T>
where
    T: Causable + PartialEq + Clone,
{
    /// Reason over single node given by its index
    /// index: NodeIndex - index of the node
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_single_cause(
        &self,
        index: usize,
        data: &[NumericalValue],
    ) -> Result<bool, CausalityGraphError> {
        if !self.contains_causaloid(index) {
            return Err(CausalityGraphError(
                "Graph does not contain causaloid".to_string(),
            ));
        }

        if data.is_empty() {
            return Err(CausalityGraphError("Data are empty (len ==0).".into()));
        }

        let causaloid = self.get_causaloid(index).expect("Failed to get causaloid");

        if data.len() == 1 {
            let obs = data.first().expect("Failed to get data");
            return match causaloid.verify_single_cause(obs) {
                Ok(res) => Ok(res),
                Err(e) => Err(CausalityGraphError(e.0)),
            };
        }

        // In case of multiple data points, all must verify the cause.
        for obs in data.iter() {
            if !causaloid
                .verify_single_cause(obs)
                .expect("Failed to verify data")
            {
                // If any observation fails, the reasoning is false.
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Reasons over the entire graph, starting from the root and traversing all reachable nodes.
    ///
    /// data: &[NumericalValue] - data applied to the subgraph
    /// Optional: data_index - provide when the data have a different index sorting than
    /// the causaloids.
    ///
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_all_causes(
        &self,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityGraphError> {
        if !self.contains_root_causaloid() {
            return Err(CausalityGraphError(
                "Graph does not contain root causaloid".into(),
            ));
        }

        // This is safe as we have tested above that a root exists.
        let start_index = self.get_root_index().expect("Root causaloid not found.");

        // Delegate to the robust subgraph reasoning implementation.
        self.reason_subgraph_from_cause(start_index, data, data_index)
    }

    /// Reasons over a subgraph by traversing all nodes reachable from a given start index.
    ///
    /// This method performs a full traversal (BFS) of all descendants of the start_index,
    /// applying reasoning logic to each one. If any node fails its verification, the
    /// entire process stops and returns `Ok(false)`.
    ///
    /// start_index: usize - index of the starting node
    /// data: &[NumericalValue] - data applied to the subgraph
    /// Optional: data_index - provide when the data have a different index sorting than
    /// the causaloids.
    ///
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_subgraph_from_cause(
        &self,
        start_index: usize,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.is_frozen() {
            return Err(CausalityGraphError(
                "Graph is not frozen. Call g.freeze() first".to_string(),
            ));
        }

        if data.is_empty() {
            return Err(CausalityGraphError("Data are empty (len ==0).".into()));
        }

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError(
                "Graph does not contain start causaloid".into(),
            ));
        }

        let mut queue = VecDeque::with_capacity(self.number_nodes());
        let mut visited = vec![false; self.number_nodes()];

        queue.push_back(start_index);
        visited[start_index] = true;

        while let Some(current_index) = queue.pop_front() {
            let cause = self
                .get_causaloid(current_index)
                .expect("Failed to get causaloid");

            let obs = graph_reasoning_utils::get_obs(cause.id(), data, &data_index);

            let res = if cause.is_singleton() {
                cause.verify_single_cause(&obs)
            } else {
                cause.verify_all_causes(data, data_index)
            };

            match res {
                Ok(true) => {
                    // The cause is valid, so add its children to the queue.
                    // Using `?` ensures that any error from `outbound_edges` (like
                    // GraphNotFrozen) is correctly propagated up.
                    let children = self.get_graph().outbound_edges(current_index)?;
                    for child_index in children {
                        if !visited[child_index] {
                            visited[child_index] = true;
                            queue.push_back(child_index);
                        }
                    }
                }
                Ok(false) => {
                    // If any cause evaluates to false, the entire reasoning chain is false.
                    return Ok(false);
                }
                Err(e) => return Err(CausalityGraphError(e.0)),
            }
        }

        // If the loop completes without any cause returning false, the reasoning is successful.
        Ok(true)
    }

    /// Reasons over the shortest path between a start and stop cause.
    ///
    /// # Preconditions
    /// The graph must be in a `Static` (frozen) state.
    ///
    /// # Errors
    /// Returns `CausalityGraphError` if the graph is not frozen.
    ///
    /// start_index: usize - index of the start cause
    /// stop_index: usize - index of the stop cause
    /// data: &[NumericalValue] - data applied to the subgraph
    /// Optional: data_index - provide when the data have a different index sorting than
    /// the causaloids.
    ///
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_shortest_path_between_causes(
        &self,
        start_index: usize,
        stop_index: usize,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if !self.is_frozen() {
            return Err(CausalityGraphError(
                "Graph is not frozen. Call graph.freeze() first".to_string(),
            ));
        }

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError(
                "Graph does not contain start causaloid".into(),
            ));
        }

        if !self.contains_causaloid(stop_index) {
            return Err(CausalityGraphError(
                "Graph does not contain stop causaloid".into(),
            ));
        }

        let path = self.get_shortest_path(start_index, stop_index)?;

        for index in path {
            let cause = self.get_causaloid(index).expect("Failed to get causaloid");

            let obs = graph_reasoning_utils::get_obs(cause.id(), data, &data_index);

            let res = match cause.verify_single_cause(&obs) {
                Ok(res) => res,
                Err(e) => return Err(CausalityGraphError(e.0)),
            };

            if !res {
                return Ok(false);
            }
        }

        Ok(true)
    }
}
