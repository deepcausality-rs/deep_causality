// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use ultragraph::prelude::*;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::prelude::{Causable, NumericalValue};
use crate::traits::causable_graph::CausalGraph;

/// The CausableGraph trait defines the core interface for a causal graph.
///
/// It builds on the CausalGraph data structure.
///
/// Provides methods for:
///
/// - Adding a root node
/// - Adding/removing nodes
/// - Adding/removing edges
/// - Accessing nodes/edges
/// - Getting graph metrics like size and active nodes
///
/// The get_graph() method returns the underlying CausalGraph instance.
/// This enables default implementations for reasoning and explaining.
///
/// Also includes a default implementation of shortest_path() using the
/// underlying CausalGraph.
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
    // The get_graph method enables the default implementation of the
    // CausableGraphExplaining and CausableGraphReasoning traits.
    fn get_graph(&self) -> &CausalGraph<T>;
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

    /// Default implementation for shortest path algorithm.
    ///
    /// Finds the shortest path between two node indices in the graph.
    ///
    /// start_index: The start node index
    /// stop_index: The target node index
    ///
    /// Returns:
    /// - Ok(Vec<usize>): The node indices of the shortest path
    /// - Err(CausalityGraphError): If no path exists
    ///
    /// Checks if start and stop nodes are identical and early returns error.
    /// Otherwise calls shortest_path() on the underlying CausalGraph.
    ///
    fn get_shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Vec<usize>, CausalityGraphError> {
        if start_index == stop_index {
            return Err(CausalityGraphError(
                "Start and Stop node identical: No shortest path possible".into(),
            ));
        }

        match self.get_graph().shortest_path(start_index, stop_index) {
            Some(path) => Ok(path),
            None => Err(CausalityGraphError("No path found".to_string())),
        }
    }
}
