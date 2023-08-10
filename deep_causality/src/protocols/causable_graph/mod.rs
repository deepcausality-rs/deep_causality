use petgraph::graph::NodeIndex as GraphNodeIndex;
use std::collections::HashMap;
use petgraph::matrix_graph::MatrixGraph;
use petgraph::Directed;

pub mod causable_graph_type;
pub mod causable_graph_reasoning;
pub mod causable_graph_explaining;

// Custom index type. See documentation in
// src/protocols/contextuable/csm_types
// for more details.
pub type DefaultIx = u32;
pub type NodeIndex<Ix = DefaultIx> = GraphNodeIndex<Ix>;
pub type IndexMap = HashMap<usize, NodeIndex>;

// CausalGraph type alias
// Edge weights need to be numerical (u64) to make shortest path algo work.
pub type CausalGraph<T> = MatrixGraph<T, u64, Directed, Option<u64>, u32>;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{Causable, IdentificationValue, NumericalValue};

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
    fn add_edg_with_weight(&mut self, a: usize, b: usize, weight: u64) -> Result<(), CausalGraphIndexError>;
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

/// Describes signatures for causal reasoning and explaining
/// in causality hyper graph.
pub trait CausableGraphReasoning<T> : CausableGraph<T>
    where
        T: Causable + PartialEq,
{
    /// Explains the line of reasoning across the entire graph.
    /// Returns: String representing the explanation or an error
    fn explain_all_causes(
        &self
    )
        -> Result<String, CausalityGraphError>;

    /// Explains the line of reasoning across a subgraph starting from a given node index until
    /// the end of the graph.
    /// index: NodeIndex - index of the starting node
    /// Returns: String representing the explanation or an error
    fn explain_subgraph_from_cause(
        &self,
        start_index: usize,
    )
        -> Result<String, CausalityGraphError>;


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
        -> Result<String, CausalityGraphError>;

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
        start_index: usize,
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
        start_index: usize,
        stop_index: usize,
        data: &[NumericalValue],
        data_index: Option<&HashMap<IdentificationValue, IdentificationValue>>,
    )
        -> Result<bool, CausalityGraphError>;

    /// Reason over single node given by its index
    /// index: NodeIndex - index of the node
    /// Returns Result either true or false in case of successful reasoning or
    /// a CausalityGraphError in case of failure.
    fn reason_single_cause(
        &self,
        index: usize,
        data: &[NumericalValue],
    )
        -> Result<bool, CausalityGraphError>;
}
