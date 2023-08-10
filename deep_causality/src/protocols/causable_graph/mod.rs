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
