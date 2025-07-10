/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{GraphError, GraphView};

pub trait StructuralGraphAlgorithms<N, W>: GraphView<N, W> {
    /// Finds all Strongly Connected Components in the graph using Tarjan's algorithm.
    ///
    /// # Returns
    /// A vector of vectors, where each inner vector is a list of node indices
    /// belonging to a single SCC.
    fn strongly_connected_components(&self) -> Result<Vec<Vec<usize>>, GraphError>;
}
