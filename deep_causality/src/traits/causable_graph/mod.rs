// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::collections::HashMap;

use ultragraph::prelude::UltraGraph;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{Causable, IdentificationValue, NumericalValue};

pub mod graph;
pub mod graph_explaining;
pub mod graph_reasoning;
mod graph_reasoning_utils;

// Type alias is shared between trait and implementation
pub(crate) type CausalGraph<T> = UltraGraph<T>;

/// The CausableGraph trait defines the interface for a causal graph data structure.
///
/// It operates on generic type T which must implement the Causable trait.
///
/// Provides methods for:
///
/// - Adding a root node
/// - Adding/removing nodes
/// - Adding/removing edges
/// - Accessing nodes/edges
/// - Getting graph metrics like size and active nodes
///
/// The root node is a special "start" node for causal reasoning.
///
/// Nodes are indexed by usize.
///
/// Edges are added by specifying the node indices.
///
/// Nodes must be unique. Edges can be duplicated.
///
/// Errors on invalid node/edge indices.
///
pub trait CausableGraph<T>
where
    T: Causable + PartialEq,
{
    // Root Node
    fn add_root_causaloid(&mut self, value: T) -> usize;
    fn contains_root_causaloid(&self) -> bool;
    fn get_root_causaloid(&self) -> Option<&T>;
    fn get_root_index(&self) -> Option<usize>;
    fn get_last_index(&self) -> Result<usize, CausalityGraphError>;

    // Nodes
    fn add_causaloid(&mut self, value: T) -> usize;
    fn contains_causaloid(&self, index: usize) -> bool;
    fn get_causaloid(&self, index: usize) -> Option<&T>;
    fn remove_causaloid(&mut self, index: usize) -> Result<(), CausalGraphIndexError>;

    // Edges
    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError>;
    fn add_edg_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), CausalGraphIndexError>;
    fn contains_edge(&self, a: usize, b: usize) -> bool;
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError>;

    // Utils
    fn all_active(&self) -> bool;
    fn number_active(&self) -> NumericalValue;
    fn percent_active(&self) -> NumericalValue;
    fn size(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&mut self);
    fn number_edges(&self) -> usize;
    fn number_nodes(&self) -> usize;
}

/// The CausableGraphReasoning trait extends CausableGraph with reasoning methods.
///
/// Provides explain and reason methods for:
/// - The entire graph
/// - Subgraphs starting from a given node
/// - Shortest path between two nodes
/// - Single nodes
///
/// The explain methods return a string explanation.
///
/// The reason methods take input data and return a Result<bool> indicating
/// if reasoning succeeded or failed.
///
/// An optional data_index can be provided to map data to nodes when the indices
/// differ.
///
pub trait CausableGraphReasoning<T>: CausableGraph<T>
where
    T: Causable + PartialEq,
{
    /// Explains the line of reasoning across the entire graph.
    /// Returns: String representing the explanation or an error
    fn explain_all_causes(&self) -> Result<String, CausalityGraphError>;

    /// Explains the line of reasoning across a subgraph starting from a given node index until
    /// the end of the graph.
    /// index: NodeIndex - index of the starting node
    /// Returns: String representing the explanation or an error
    fn explain_subgraph_from_cause(
        &self,
        start_index: usize,
    ) -> Result<String, CausalityGraphError>;

    /// Explains the line of reasoning of the shortest sub-graph
    /// between a start and stop cause.
    /// start_index: NodeIndex - index of the start cause
    /// stop_index: NodeIndex - index of the stop cause
    /// Returns: String representing the explanation or an error
    fn explain_shortest_path_between_causes(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<String, CausalityGraphError>;

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
    ) -> Result<bool, CausalityGraphError>;

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
    ) -> Result<bool, CausalityGraphError>;

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
    ) -> Result<bool, CausalityGraphError>;

    /// Reason over single node given by its index
    /// index: NodeIndex - index of the node
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_single_cause(
        &self,
        index: usize,
        data: &[NumericalValue],
    ) -> Result<bool, CausalityGraphError>;
}
