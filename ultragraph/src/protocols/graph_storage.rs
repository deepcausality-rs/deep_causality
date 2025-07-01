/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::protocols::graph_algorithms::GraphAlgorithms;
use crate::protocols::graph_like::GraphLike;
use crate::protocols::graph_root::GraphRoot;

pub trait GraphStorage<T>: GraphLike<T> + GraphRoot<T> + GraphAlgorithms<T> {
    /// Returns the total number of nodes and edges in the graph.
    ///
    /// This is equivalent to `self.number_nodes() + self.number_edges()`.
    fn size(&self) -> usize;

    /// Returns `true` if the graph contains no nodes or edges.
    ///
    /// This is equivalent to `self.size() == 0`.
    fn is_empty(&self) -> bool;

    /// Returns the number of nodes in the graph.
    fn number_nodes(&self) -> usize;

    /// Returns the number of edges in the graph.
    fn number_edges(&self) -> usize;

    /// Returns a vector containing references to all nodes in the graph.
    ///
    /// The order of nodes in the returned vector is not guaranteed.
    fn get_all_nodes(&self) -> Vec<&T>;

    /// Returns a vector of tuples representing all edges in the graph.
    ///
    /// Each tuple `(source_index, target_index)` represents an edge from the node at `source_index`
    /// to the node at `target_index`. The indices correspond to the internal storage of the graph.
    ///
    /// The order of edges in the returned vector is not guaranteed.
    fn get_all_edges(&self) -> Vec<(usize, usize)>;

    /// Clears all nodes and edges from the graph, making it empty.
    ///
    /// After calling this method, `self.is_empty()` will return `true`,
    /// and `self.size()`, `self.number_nodes()`, and `self.number_edges()`
    /// will all return `0`.
    fn clear(&mut self);
}
