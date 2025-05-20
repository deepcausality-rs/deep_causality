// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::collections::HashMap;

use ultragraph::prelude::*;

use crate::errors::CausalityGraphError;
use crate::prelude::{Causable, CausableGraph, IdentificationValue, NumericalValue};
use crate::protocols::causable_graph::graph_reasoning_utils;

/// Describes signatures for causal reasoning and explaining
/// in causality hyper graph.
pub trait CausableGraphReasoning<T>: CausableGraph<T>
where
    T: Causable + PartialEq,
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

        if data.len() > 1 {
            for obs in data.iter() {
                if !causaloid
                    .verify_single_cause(obs)
                    .expect("Failed to verify data")
                {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Reason over the entire graph.
    /// data: &[NumericalValue] - data applied to the subgraph
    /// Optional: data_index - provide when the data have a different index sorting than
    /// the causaloids.
    ///
    /// Conventionally, the index of the causaloid is matched to the
    /// index of the data so that data at index i get applied to causaloid i.
    /// If, for any reason, the data use a different index, the the optional data_index
    /// is used to match a causaloid i to its data at a (different) index n.
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
                "Graph does not contains root causaloid".into(),
            ));
        }

        // These is safe as we have tested above that these exists
        let start_index = self.get_root_index().expect("Root causaloid not found.");
        let stop_index = self.get_last_index().expect("Last causaloid not found");

        match self.reason_from_to_cause(start_index, stop_index, data, data_index) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    /// Reason over a subgraph starting from a given node index.
    ///
    /// start_index: NodeIndex - index of the starting node
    /// data: &[NumericalValue] - data applied to the subgraph
    /// Optional: data_index - provide when the data have a different index sorting than
    /// the causaloids.
    ///
    /// Conventionally, the index of the causaloid is matched to the
    /// index of the data so that data at index i get applied to causaloid i.
    /// If, for any reason, the data use a different index, the the optional data_index
    /// is used to match a causaloid i to its data at a (different) index n.
    ///
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_subgraph_from_cause(
        &self,
        start_index: usize,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityGraphError> {
        if self.get_last_index().is_err() {
            return Err(CausalityGraphError(
                "Graph does not contains stop causaloid".into(),
            ));
        }

        let stop_index = self.get_last_index().expect("Last causaloid not found");

        match self.reason_from_to_cause(start_index, stop_index, data, data_index) {
            Ok(result) => Ok(result),
            Err(e) => Err(e),
        }
    }

    /// Reasons over the graph from start_index to stop_index.
    ///
    /// start_index: Node index to start reasoning from
    /// stop_index: Node index to end reasoning
    /// data: Observations to apply to nodes
    /// data_index: Optional index map if data indices differ from node indices
    ///
    /// Gets start node and verifies it. If false, returns false.
    ///
    /// Uses a stack to traverse nodes depth-first:
    /// - Get node's children and push to stack
    /// - Pop next node and get observations
    /// - Verify node and if false, return false
    /// - If node is stop_index, return true
    /// - Else, push node's children to stack
    ///
    /// Returns:
    /// - Ok(bool): True if all nodes verify, False if any node fails
    /// - Err(CausalityGraphError): On invalid indices or empty data
    ///
    /// Traverses nodes depth-first, verifying each one.
    /// If any node fails, returns false. If all pass, returns true.
    ///
    /// Algo inspired by simple path https://github.com/petgraph/petgraph/blob/master/src/algo/simple_paths.rs
    fn reason_from_to_cause(
        &self,
        start_index: usize,
        stop_index: usize,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    ) -> Result<bool, CausalityGraphError> {
        if self.is_empty() {
            return Err(CausalityGraphError("Graph is empty".to_string()));
        }

        if data.is_empty() {
            return Err(CausalityGraphError("Data are empty (len ==0).".into()));
        }

        if !self.contains_causaloid(start_index) {
            return Err(CausalityGraphError(
                "Graph does not contains start causaloid".into(),
            ));
        }

        let cause = self
            .get_causaloid(start_index)
            .expect("Failed to get causaloid");

        let obs = graph_reasoning_utils::get_obs(cause.id(), data, &data_index);

        let res = match cause.verify_single_cause(&obs) {
            Ok(res) => res,
            Err(e) => return Err(CausalityGraphError(e.0)),
        };

        if !res {
            return Ok(false);
        }

        let mut stack = Vec::with_capacity(self.size());
        stack.push(self.get_graph().outgoing_edges(start_index).unwrap());

        while let Some(children) = stack.last_mut() {
            if let Some(child) = children.next() {
                let cause = self.get_causaloid(child).expect("Failed to get causaloid");

                let obs = graph_reasoning_utils::get_obs(cause.id(), data, &data_index);

                let res = if cause.is_singleton() {
                    match cause.verify_single_cause(&obs) {
                        Ok(res) => res,
                        Err(e) => return Err(CausalityGraphError(e.0)),
                    }
                } else {
                    match cause.verify_all_causes(data, data_index) {
                        Ok(res) => res,
                        Err(e) => return Err(CausalityGraphError(e.0)),
                    }
                };

                if !res {
                    return Ok(false);
                }

                if child == stop_index {
                    return Ok(true);
                } else {
                    stack.push(self.get_graph().outgoing_edges(child).unwrap());
                }
            } else {
                stack.pop();
            }
        }

        // If all of the previous nodes evaluated to true,
        // then all nodes must be true, hence return true.
        Ok(true)
    }

    /// Reason over the shortest subgraph spanning between a start and stop cause.
    ///
    /// start_index: NodeIndex - index of the start cause
    /// stop_index: NodeIndex - index of the stop cause
    /// data: &[NumericalValue] - data applied to the subgraph
    /// Optional: data_index - provide when the data have a different index sorting than
    /// the causaloids.
    ///
    /// Conventionally, the index of the causaloid is matched to the
    /// index of the data so that data at index i get applied to causaloid i.
    /// If, for any reason, the data use a different index, the the optional data_index
    /// is used to match a causaloid i to its data at a (different) index n.
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

        for index in shortest_path {
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
