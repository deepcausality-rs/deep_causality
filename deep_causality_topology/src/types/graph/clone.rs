/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Graph;

impl<T> Graph<T> {
    /// Creates a shallow clone of the Graph.
    pub fn clone_shallow(&self) -> Self
    where
        T: Clone,
    {
        Graph {
            num_vertices: self.num_vertices,
            adjacencies: self.adjacencies.clone(),
            num_edges: self.num_edges,
            data: self.data.clone(),
            cursor: 0,
        }
    }
}
