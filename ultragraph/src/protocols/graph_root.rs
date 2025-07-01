/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::UltraGraphError;
use crate::protocols::graph_like::GraphLike;

/// The `GraphRoot` trait defines the behavior for graph-like structures that have a designated root node.
///
/// This trait extends `GraphLike` and adds methods specifically for managing a root node within the graph.
///
/// # Type Parameters
///
/// * `T`: The type of the data stored in the graph nodes.
///
pub trait GraphRoot<T>: GraphLike<T> {
    /// * `add_root_node(&mut self, value: T) -> usize`:
    ///   Adds a new node with the given `value` and designates it as the root of the graph.
    ///   Returns the index of the newly added root node.
    fn add_root_node(&mut self, value: T) -> usize;

    ///  `contains_root_node(&self) -> bool`:
    ///   Checks if a root node currently exists in the graph.
    ///   Returns `true` if a root node is present, `false` otherwise.
    fn contains_root_node(&self) -> bool;

    ///   `get_root_node(&self) -> Option<&T>`:
    ///   Retrieves an immutable reference to the data stored in the root node.
    ///   Returns `Some(&T)` if a root node exists, otherwise returns `None`.
    fn get_root_node(&self) -> Option<&T>;

    ///  `get_root_index(&self) -> Option<usize>`:
    ///   Retrieves the index of the root node.
    ///   Returns `Some(usize)` if a root node exists, otherwise returns `None`.
    fn get_root_index(&self) -> Option<usize>;

    ///  `get_last_index(&self) -> Result<usize, UltraGraphError>`:
    ///   Retrieves the index of the last node added to the graph.
    ///   This is useful for understanding the current size or extent of the graph.
    ///   Returns `Ok(usize)` with the index if successful, or an `UltraGraphError`
    ///   if no nodes have been added yet (e.g., an empty graph).
    fn get_last_index(&self) -> Result<usize, UltraGraphError>;
}
