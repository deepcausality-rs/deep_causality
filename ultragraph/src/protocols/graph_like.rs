/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::UltraGraphError;

pub trait GraphLike<T> {
    fn add_node(&mut self, value: T) -> usize;

    fn contains_node(&self, index: usize) -> bool;

    fn get_node(&self, index: usize) -> Option<&T>;

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

    fn add_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError>;

    fn add_edge_with_weight(
        &mut self,
        a: usize,
        b: usize,
        weight: u64,
    ) -> Result<(), UltraGraphError>;

    fn contains_edge(&self, a: usize, b: usize) -> bool;

    fn remove_edge(&mut self, a: usize, b: usize) -> Result<(), UltraGraphError>;
}
