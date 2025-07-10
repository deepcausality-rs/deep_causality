/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{GraphAlgorithms, GraphError, GraphState, UltraGraphContainer};

impl<N, W> GraphAlgorithms<N, W> for UltraGraphContainer<N, W>
where
    N: Clone,
    W: Clone + Default,
{
    /// Finds a cycle in the graph.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    fn find_cycle(&self) -> Result<Option<Vec<usize>>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.find_cycle(),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Checks if the graph contains a cycle.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    fn has_cycle(&self) -> Result<bool, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.has_cycle(),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Performs a topological sort of the graph's nodes.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error from the underlying implementation if the graph contains a cycle.
    fn topological_sort(&self) -> Result<Option<Vec<usize>>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.topological_sort(),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Checks if a node `stop_index` is reachable from `start_index`.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error if either node index is invalid.
    fn is_reachable(&self, start_index: usize, stop_index: usize) -> Result<bool, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.is_reachable(start_index, stop_index),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Calculates the length of the shortest path between two nodes.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error if either node index is invalid.
    fn shortest_path_len(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<usize>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.shortest_path_len(start_index, stop_index),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Finds the shortest path (as a sequence of node indices) between two nodes.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error if either node index is invalid.
    fn shortest_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<Vec<usize>>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.shortest_path(start_index, stop_index),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Finds the shortest path (as a sequence of node indices) between two nodes, considering edge weights.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Type Parameters
    /// - `W`: The weight type, which must implement `Copy`, `Ord`, `Default`, and `std::ops::Add`.
    ///
    /// # Errors
    ///
    /// - Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    /// - Returns an error if either node index is invalid.
    /// - Returns an error if the graph contains negative cycles (for algorithms like Dijkstra's).
    ///
    fn shortest_weighted_path(
        &self,
        start_index: usize,
        stop_index: usize,
    ) -> Result<Option<(Vec<usize>, W)>, GraphError>
    where
        W: Copy + Ord + Default + std::ops::Add<Output = W>,
    {
        match &self.state {
            GraphState::Static(g) => g.shortest_weighted_path(start_index, stop_index),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }

    /// Finds the strongly connected components (SCCs) of the graph.
    ///
    /// # Preconditions
    /// This high-performance operation is only available when the graph is in a `Static` (frozen) state.
    ///
    /// # Returns
    /// A `Result` containing a `Vec<Vec<usize>>`, where each inner `Vec<usize>` represents
    /// a strongly connected component as a list of node indices.
    ///
    /// # Errors
    /// Returns `GraphError::GraphNotFrozen` if the graph is in a `Dynamic` state.
    fn strongly_connected_components(&self) -> Result<Vec<Vec<usize>>, GraphError> {
        match &self.state {
            GraphState::Static(g) => g.strongly_connected_components(),
            GraphState::Dynamic(_) => Err(GraphError::GraphNotFrozen),
        }
    }
}
