/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GraphError, GraphView};

pub trait TopologicalGraphAlgorithms<N, W>: GraphView<N, W> {
    /// Finds a single cycle in the graph and returns the path of nodes that form it.
    ///
    /// This is the most powerful cycle detection method, as it not only confirms the
    /// presence of a cycle but also identifies the specific nodes involved. This is
    /// invaluable for debugging dynamically generated graphs.
    ///
    /// # Returns
    /// `Some(Vec<usize>)` containing the sequence of node indices that form a cycle
    /// (e.g., `[0, 1, 0]`). Returns `None` if the graph is a DAG.
    fn find_cycle(&self) -> Result<Option<Vec<usize>>, GraphError>;

    /// Checks if the graph contains any directed cycles.
    ///
    /// This method should be implemented as a simple call to `self.find_cycle().is_some()`.
    fn has_cycle(&self) -> Result<bool, GraphError>;

    /// Computes a topological sort of the graph, if it is a Directed Acyclic Graph (DAG).
    /// Returns `None` if the graph contains a cycle.
    fn topological_sort(&self) -> Result<Option<Vec<usize>>, GraphError>;
}
