/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::vec::IntoIter;

use crate::errors::UltraGraphError;
use crate::prelude::GraphLike;

pub trait GraphAlgorithms<T>: GraphLike<T> {
    /// Returns the path of subsequent NodeId from start to finish, if one was found.
    fn shortest_path(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>>;
    ///
    /// # Arguments
    ///
    /// * `a` - The index of the node from which to find outgoing edges.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `IntoIter<usize>` of node indices that have an outgoing edge from `a`,
    /// or an `UltraGraphError` if an error occurs (e.g., the node `a` does not exist).
    fn outgoing_edges(&self, a: usize) -> Result<IntoIter<usize>, UltraGraphError>;
}
