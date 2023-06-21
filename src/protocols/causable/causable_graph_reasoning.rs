/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */
use std::collections::HashMap;

use crate::prelude::{Causable, CausalityGraphError, IdentificationValue, NodeIndex, NumericalValue};

/// Describes signatures for causal reasoning and explaining
/// in causality hyper graph.
pub trait CausableGraphReasoning<T>
    where
        T: Causable + PartialEq,
{
    /// Explains the line of reasoning across the entire graph.
    ///
    /// Returns: String representing the explanation or an error
    fn explain_all_causes(
        &self
    )
        -> Result<String, CausalityGraphError>;

    /// Explains the line of reasoning across a subgraph starting from a given node index until
    /// the end of the graph.
    ///
    /// index: NodeIndex - index of the starting node
    ///
    /// Returns: String representing the explanation or an error
    fn explain_subgraph_from_cause(
        &self,
        start_index: NodeIndex,
    )
        -> Result<String, CausalityGraphError>;


    /// Explains the line of reasoning of the shortest sub-graph
    /// between a start and stop cause.
    ///
    /// start_index: NodeIndex - index of the start cause
    /// stop_index: NodeIndex - index of the stop cause
    ///
    /// Returns: String representing the explanation or an error
    fn explain_shortest_path_between_causes(
        &self,
        start_index: NodeIndex,
        stop_index: NodeIndex,
    )
        -> Result<String, CausalityGraphError>;

    /// Reason over the entire graph.
    ///
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
    )
        -> Result<bool, CausalityGraphError>;

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
        start_index: NodeIndex,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityGraphError>;

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
        start_index: NodeIndex,
        stop_index: NodeIndex,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityGraphError>;

    /// Reason over single node given by its index
    ///
    /// index: NodeIndex - index of the node
    ///
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_single_cause(
        &self,
        index: NodeIndex,
        data: &[NumericalValue],
    )
        -> Result<bool, CausalityGraphError>;
}
