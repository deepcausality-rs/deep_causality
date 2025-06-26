/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::vec::IntoIter;

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use petgraph::algo::astar;
use petgraph::prelude::EdgeRef;

use crate::errors::UltraGraphError;
use crate::prelude::{GraphAlgorithms, GraphLike, UltraMatrixGraph};
use crate::storage::matrix_graph::NodeIndex;

impl<T> GraphAlgorithms<T> for UltraMatrixGraph<T> {
    fn shortest_path(&self, start_index: usize, stop_index: usize) -> Option<Vec<usize>> {
        if !self.contains_node(start_index) {
            return None;
        };

        if !self.contains_node(stop_index) {
            return None;
        };

        let mut result: Vec<usize> = Vec::new();

        // A* algorithm https://docs.rs/petgraph/latest/petgraph/algo/astar/fn.astar.html
        if let Some((_, path)) = astar(
            &self.graph,
            NodeIndex::new(start_index),
            |finish| finish == NodeIndex::new(stop_index),
            |e| *e.weight(),
            |_| 0,
        ) {
            for node in path {
                result.push(node.index());
            }
            Some(result)
        } else {
            None
        }
    }

    fn outgoing_edges(&self, a: usize) -> Result<IntoIter<usize>, UltraGraphError> {
        if !self.contains_node(a) {
            return Err(UltraGraphError("index a not found".into()));
        };

        let mut result: Vec<usize> = Vec::new();

        let neighbors = self.graph.neighbors(NodeIndex::new(a));

        for node in neighbors {
            result.push(node.index());
        }

        Ok(result.into_iter())
    }
}
