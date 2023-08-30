// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::GraphStorage;

use super::UltraMatrixGraph;

impl<T> GraphStorage<T> for UltraMatrixGraph<T> {
    fn size(&self) -> usize {
        self.graph.node_count()
    }

    fn is_empty(&self) -> bool {
        self.graph.node_count() == 0
    }

    fn number_nodes(&self) -> usize {
        self.graph.node_count()
    }

    fn number_edges(&self) -> usize {
        self.graph.edge_count()
    }

    fn get_all_nodes(&self) -> Vec<&T> {
        let mut res = Vec::with_capacity(self.graph.node_count());

        for val in self.node_map.values() {
            res.push(val);
        }

        res
    }

    fn get_all_edges(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::with_capacity(self.node_map.len());

        for idx in self.node_map.keys() {
            for e in self.graph.neighbors(*idx) {
                edges.push((idx.index(), e.index()));
            }
        }

        edges
    }

    fn clear(&mut self) {
        self.graph.clear();
        self.node_map.clear();
        self.index_map.clear();
        self.root_index = None;
    }
}
