/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::*;

use crate::errors::{CausalGraphIndexError, CausalityGraphError};
use crate::traits::causable_graph::CausalGraph;
use crate::{Causable, CausalityError, NumericalValue};

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
    T: Clone + Causable + PartialEq,
{
    fn is_frozen(&self) -> bool;

    /// Ensures the graph is in the immutable, performance-optimized `Static` state.
    ///
    /// If the graph is already frozen, this operation is a no-op. Otherwise, it
    /// converts the graph from a `Dynamic` state in-place. This is an O(V + E)
    /// operation if a state change occurs.
    fn freeze(&mut self);

    /// Ensures the graph is in the mutable, `Dynamic` state.
    ///
    /// If the graph is already dynamic, this operation is a no-op. Otherwise, it
    /// converts the graph from a `Static` state in-place. This is an O(V + E)
    /// operation if a state change occurs and requires node and edge data to be `Clone`.
    fn unfreeze(&mut self);

    /// Returns a reference to the underlying `CausalGraph`.
    ///
    /// This method is primarily used to enable default implementations for
    /// other traits like `CausableGraphExplaining` and `CausableGraphReasoning`,
    /// allowing them to operate directly on the graph structure.
    fn get_graph(&self) -> &CausalGraph<T>;

    /// Adds a special "root" causaloid to the graph.
    ///
    /// The root node serves as the starting point for causal reasoning and traversals.
    /// There can typically only be one root node in the graph.
    ///
    /// # Arguments
    ///
    /// * `value` - The causaloid of type `T` to be added as the root.
    ///
    /// # Returns
    ///
    /// The `usize` index of the newly added root node.
    fn add_root_causaloid(&mut self, value: T) -> Result<usize, CausalityGraphError>;

    /// Checks if a root causaloid has been set in the graph.
    ///
    /// # Returns
    ///
    /// * `true` if a root node exists, `false` otherwise.
    fn contains_root_causaloid(&self) -> bool;

    /// Retrieves an immutable reference to the root causaloid, if it exists.
    ///
    /// # Returns
    ///
    /// * `Some(&T)` containing a reference to the root causaloid if it's present.
    /// * `None` if no root node has been added to the graph.
    fn get_root_causaloid(&self) -> Option<&T>;

    /// Retrieves the index of the root causaloid, if it exists.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` containing the index of the root node if it's present.
    /// * `None` if no root node has been added to the graph.
    fn get_root_index(&self) -> Option<usize>;

    /// Gets the index of the last causaloid added to the graph.
    ///
    /// This is useful for understanding the current size or for linking
    /// to a newly added node.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` containing the index of the last node.
    /// * `Err(CausalityGraphError)` if the graph is empty and has no nodes.
    fn get_last_index(&self) -> Result<usize, CausalityGraphError>;

    // Nodes
    /// Adds a new causaloid (node) to the graph.
    ///
    /// # Arguments
    ///
    /// * `value` - The causaloid of type `T` to be added to the graph.
    ///
    /// # Returns
    ///
    /// The `usize` index of the newly added causaloid.
    fn add_causaloid(&mut self, value: T) -> Result<usize, CausalityGraphError>;

    /// Checks if a causaloid exists at a specific index in the graph.
    ///
    /// # Arguments
    ///
    /// * `index` - The `usize` index to check for the existence of a causaloid.
    ///
    /// # Returns
    ///
    /// * `true` if a causaloid is present at the given index.
    /// * `false` if the index is out of bounds or no causaloid is at that index.
    fn contains_causaloid(&self, index: usize) -> bool;

    /// Retrieves an immutable reference to a causaloid at a given index.
    ///
    /// # Arguments
    ///
    /// * `index` - The `usize` index of the causaloid to retrieve.
    ///
    /// # Returns
    ///
    /// * `Some(&T)` containing a reference to the causaloid if it exists at the specified index.
    /// * `None` if the index is out of bounds.
    fn get_causaloid(&self, index: usize) -> Option<&T>;

    /// Removes a causaloid from the graph at the specified index.
    ///
    /// Note: Removing a causaloid will also remove all edges connected to it.
    ///
    /// # Arguments
    ///
    /// * `index` - The `usize` index of the causaloid to remove.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the causaloid was successfully removed.
    /// * `Err(CausalGraphIndexError)` if the provided `index` is invalid or out of bounds.
    fn remove_causaloid(&mut self, index: usize) -> Result<(), CausalGraphIndexError>;

    // Edges
    /// Adds a directed edge between two causaloids in the graph.
    ///
    /// This creates a causal link from the causaloid at index `a` to the one at index `b`.
    ///
    /// # Arguments
    ///
    /// * `a` - The `usize` index of the source causaloid (the cause).
    /// * `b` - The `usize` index of the target causaloid (the effect).
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the edge was successfully added.
    /// * `Err(CausalGraphIndexError)` if either `a` or `b` is an invalid index.
    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError>;

    /// Adds a weighted directed edge between two causaloids.
    ///
    /// The weight can represent the strength, probability, or intensity of the causal relationship.
    ///
    /// # Arguments
    ///
    /// * `a` - The `usize` index of the source causaloid.
    /// * `b` - The `usize` index of the target causaloid.
    /// * `weight` - The `u64` weight of the edge.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the weighted edge was successfully added.
    /// * `Err(CausalGraphIndexError)` if either `a` or `b` is an invalid index.
    fn add_edg_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), CausalGraphIndexError>;

    /// Checks if a directed edge exists from causaloid `a` to causaloid `b`.
    ///
    /// # Arguments
    ///
    /// * `a` - The `usize` index of the source causaloid.
    /// * `b` - The `usize` index of the target causaloid.
    ///
    /// # Returns
    ///
    /// * `true` if a directed edge from `a` to `b` exists.
    /// * `false` if no such edge exists or if either index is invalid.
    fn contains_edge(&self, a: usize, b: usize) -> bool;

    /// Removes a directed edge between two causaloids.
    ///
    /// If multiple edges exist between `a` and `b`, this will remove one of them.
    ///
    /// # Arguments
    ///
    /// * `a` - The `usize` index of the source causaloid.
    /// * `b` - The `usize` index of the target causaloid.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the edge was successfully removed.
    /// * `Err(CausalGraphIndexError)` if the edge does not exist or if either index is invalid.
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), CausalGraphIndexError>;

    // Utils
    /// Checks if all causaloids in the graph are currently active.
    ///
    /// This method iterates through all causaloids in the graph and calls
    /// the `is_active()` method on each one.
    ///
    /// # Returns
    ///
    /// * `true` if every causaloid in the graph is active.
    /// * `false` if at least one causaloid is not active.
    fn all_active(&self) -> Result<bool, CausalityError>;

    /// Counts the number of active causaloids in the graph.
    ///
    /// # Returns
    ///
    /// A `NumericalValue` representing the total count of active causaloids.
    fn number_active(&self) -> Result<NumericalValue, CausalityError>;

    /// Calculates the percentage of active causaloids relative to the total number of causaloids.
    ///
    /// # Returns
    ///
    /// A `NumericalValue` representing the percentage of active causaloids (e.g., from 0.0 to 100.0).
    fn percent_active(&self) -> Result<NumericalValue, CausalityError>;

    /// Returns the total number of causaloids (nodes) in the graph.
    ///
    /// # Returns
    ///
    /// A `usize` representing the total count of nodes in the graph.
    fn size(&self) -> usize;

    /// Checks if the graph contains no causaloids.
    ///
    /// # Returns
    ///
    /// * `true` if the graph has no nodes.
    /// * `false` if the graph has one or more nodes.
    fn is_empty(&self) -> bool;

    /// Removes all causaloids and edges from the graph.
    ///
    /// After calling this method, the graph will be empty.
    fn clear(&mut self);

    /// Returns the total number of edges in the graph.
    ///
    /// # Returns
    ///
    /// A `usize` representing the total count of edges.
    fn number_edges(&self) -> usize;

    /// Returns the total number of causaloids (nodes) in the graph.
    ///
    /// # Returns
    ///
    /// A `usize` representing the total count of nodes.
    fn number_nodes(&self) -> usize;

    /// Default implementation for shortest path algorithm.
    ///
    /// Finds the shortest path between two node indices in the graph.
    ///
    /// start_index: The start node index
    /// stop_index: The target node index
    ///
    /// Returns:
    /// - Ok(`Vec<usize>`): The node indices of the shortest path
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
            Ok(path) => match path {
                Some(path) => Ok(path),
                None => Err(CausalityGraphError("No path found".to_string())),
            },
            Err(e) => Err(CausalityGraphError(format!("{e}"))),
        }
    }
    /// Counts the number of nodes that are known to be active, ignoring unevaluated nodes.
    ///
    /// This is a lenient check useful for inspecting partially evaluated graphs.
    /// It treats any unevaluated node as "not active" for the purpose of the count.
    fn count_known_active(&self) -> NumericalValue;
}
