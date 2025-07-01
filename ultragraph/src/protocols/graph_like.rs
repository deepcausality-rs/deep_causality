/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::UltraGraphError;

pub trait GraphLike<T> {
    fn add_node(&mut self, value: T) -> usize;

    /// Checks if a node exists at the given index.
    ///
    /// # Arguments
    /// * `index`: The `usize` index of the node to check.
    ///
    /// # Returns
    /// `true` if a node exists at the index, `false` otherwise.
    fn contains_node(&self, index: usize) -> bool;

    /// Retrieves a reference to the payload of a node at the given index.
    ///
    /// # Arguments
    /// * `index`: The `usize` index of the node to retrieve.
    ///
    /// # Returns
    /// An `Option` containing a reference to the node's payload if found, `None` otherwise.
    fn get_node(&self, index: usize) -> Option<&T>;

    /// Removes a node at the given index.
    ///
    /// This operation also removes all edges connected to the node.
    ///
    /// # Arguments
    /// * `index`: The `usize` index of the node to remove.
    ///
    /// # Errors
    /// Returns `UltraGraphError` if no node exists at the given index.
    fn remove_node(&mut self, index: usize) -> Result<(), UltraGraphError>;

    /// Updates the payload of an existing node at a given index.
    ///
    /// This operation preserves all edges connected to the node.
    ///
    /// # Arguments
    /// * `index`: The `usize` index of the node to update.
    /// * `value`: The new data payload for the node.
    ///
    /// # Errors
    /// Returns `UltraGraphError` if no node exists at the given index.
    fn update_node(&mut self, index: usize, value: T) -> Result<(), UltraGraphError>;

    /// Adds an unweighted edge between two nodes.
    ///
    /// # Arguments
    /// * `a`: The `usize` index of the first node.
    /// * `b`: The `usize` index of the second node.
    ///
    /// # Errors
    /// Returns `UltraGraphError` if either node `a` or node `b` does not exist,
    /// or if an edge already exists between them.
    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError>;

    /// Adds a weighted edge between two nodes.
    ///
    /// # Arguments
    /// * `a`: The `usize` index of the first node.
    /// * `b`: The `usize` index of the second node.
    /// * `weight`: The `u64` weight of the edge.
    ///
    /// # Errors
    /// Returns `UltraGraphError` if either node `a` or node `b` does not exist,
    /// or if an edge already exists between them.
    fn add_edge_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), UltraGraphError>;

    /// Checks if an edge exists between two nodes.
    ///
    /// # Arguments
    /// * `a`: The `usize` index of the first node.
    /// * `b`: The `usize` index of the second node.
    ///
    /// # Returns
    /// `true` if an edge exists between `a` and `b`, `false` otherwise.
    fn contains_edge(&self, a: usize, b: usize) -> bool;

    /// Removes an edge between two nodes.
    ///
    /// # Arguments
    /// * `a`: The `usize` index of the first node.
    /// * `b`: The `usize` index of the second node.
    ///
    /// # Errors
    /// Returns `UltraGraphError` if either node `a` or node `b` does not exist,
    /// or if no edge exists between them.
    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError>;
}
